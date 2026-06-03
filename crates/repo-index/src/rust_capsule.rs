use quote::ToTokens;
use syn::{
    Fields, File, FnArg, ImplItem, Item, Pat, ReturnType, Signature, TraitItem, Type, UseTree,
};

pub(crate) fn contains_symbol_match(input: &str, query: &str) -> bool {
    input.lines().any(|line| is_symbol_line_match(line, query))
}

pub(crate) fn is_symbol_line_match(line: &str, query: &str) -> bool {
    looks_like_symbol_line(line) && contains_symbol_token(line, query)
}

fn contains_symbol_token(input: &str, query: &str) -> bool {
    let normalized_query = query.to_ascii_lowercase();
    input
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| !token.is_empty())
        .any(|token| token.to_ascii_lowercase() == normalized_query)
}

fn looks_like_symbol_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    matches!(
        trimmed,
        line if line.starts_with("fn ")
            || line.starts_with("struct ")
            || line.starts_with("enum ")
            || line.starts_with("trait ")
            || line.starts_with("impl")
            || line.starts_with("use ")
            || line.starts_with("type ")
            || line.starts_with("const ")
            || line.starts_with("static ")
            || line.starts_with("mod ")
    )
}

pub(crate) fn build_rust_capsule(contents: &str) -> Option<String> {
    let file = syn::parse_file(contents).ok()?;
    let mut lines = Vec::new();
    collect_rust_capsule_lines(&file, &mut lines);

    if lines.is_empty() {
        return Some(String::new());
    }

    Some(lines.join("\n"))
}

fn collect_rust_capsule_lines(file: &File, lines: &mut Vec<String>) {
    for item in &file.items {
        match item {
            Item::Use(item_use) => lines.push(format!("use {};", format_use_tree(&item_use.tree))),
            Item::Struct(item_struct) => lines.push(format_struct(item_struct)),
            Item::Enum(item_enum) => lines.push(format_enum(item_enum)),
            Item::Trait(item_trait) => lines.push(format_trait(item_trait)),
            Item::Fn(item_fn) => lines.push(format_fn_signature(&item_fn.sig, false)),
            Item::Impl(item_impl) => lines.extend(format_impl(item_impl)),
            _ => {}
        }
    }
}

fn format_struct(item_struct: &syn::ItemStruct) -> String {
    let mut line = format!(
        "struct {}{}",
        item_struct.ident,
        format_generics(&item_struct.generics)
    );

    match &item_struct.fields {
        Fields::Named(fields) => {
            let field_list = fields
                .named
                .iter()
                .filter_map(|field| {
                    field
                        .ident
                        .as_ref()
                        .map(|ident| format!("{}: {}", ident, format_type(&field.ty)))
                })
                .collect::<Vec<_>>()
                .join(", ");
            line.push_str(&format!(" {{ {} }}", field_list));
        }
        Fields::Unnamed(fields) => {
            let field_list = fields
                .unnamed
                .iter()
                .map(|field| format_type(&field.ty))
                .collect::<Vec<_>>()
                .join(", ");
            line.push_str(&format!("({});", field_list));
        }
        Fields::Unit => line.push(';'),
    }

    line
}

fn format_enum(item_enum: &syn::ItemEnum) -> String {
    let variants = item_enum
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            Fields::Named(fields) => {
                let field_list = fields
                    .named
                    .iter()
                    .filter_map(|field| {
                        field
                            .ident
                            .as_ref()
                            .map(|ident| format!("{}: {}", ident, format_type(&field.ty)))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", variant.ident, field_list)
            }
            Fields::Unnamed(fields) => {
                let field_list = fields
                    .unnamed
                    .iter()
                    .map(|field| format_type(&field.ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", variant.ident, field_list)
            }
            Fields::Unit => variant.ident.to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "enum {}{} {{ {} }}",
        item_enum.ident,
        format_generics(&item_enum.generics),
        variants
    )
}

fn format_trait(item_trait: &syn::ItemTrait) -> String {
    let items = item_trait
        .items
        .iter()
        .filter_map(|item| match item {
            TraitItem::Fn(method) => Some(format_fn_signature(&method.sig, false)),
            TraitItem::Type(assoc_type) => Some(format!("type {};", assoc_type.ident)),
            TraitItem::Const(assoc_const) => Some(format!(
                "const {}: {};",
                assoc_const.ident,
                format_type(&assoc_const.ty)
            )),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");

    format!(
        "trait {}{} {{ {} }}",
        item_trait.ident,
        format_generics(&item_trait.generics),
        items
    )
}

fn format_impl(item_impl: &syn::ItemImpl) -> Vec<String> {
    let mut lines = Vec::new();
    let mut header = String::from("impl");
    let generics = format_generics(&item_impl.generics);
    if !generics.is_empty() {
        header.push_str(&generics);
    }
    header.push(' ');
    if let Some((_, trait_path, _)) = &item_impl.trait_ {
        header.push_str(&format_tokens(&trait_path.to_token_stream()));
        header.push_str(" for ");
    }
    header.push_str(&format_type(&item_impl.self_ty));
    lines.push(format!("{} {{", header));

    for item in &item_impl.items {
        if let ImplItem::Fn(method) = item {
            lines.push(format!("  {}", format_fn_signature(&method.sig, false)));
        }
    }

    lines.push("}".to_string());
    lines
}

fn format_fn_signature(signature: &Signature, include_body_stub: bool) -> String {
    let mut line = String::new();

    if signature.constness.is_some() {
        line.push_str("const ");
    }
    if signature.asyncness.is_some() {
        line.push_str("async ");
    }
    if signature.unsafety.is_some() {
        line.push_str("unsafe ");
    }
    if let Some(abi) = &signature.abi {
        line.push_str(&format_tokens(&abi.to_token_stream()));
        line.push(' ');
    }

    line.push_str("fn ");
    line.push_str(&signature.ident.to_string());
    line.push_str(&format_generics(&signature.generics));
    line.push('(');
    line.push_str(
        &signature
            .inputs
            .iter()
            .map(format_fn_arg)
            .collect::<Vec<_>>()
            .join(", "),
    );
    line.push(')');

    match &signature.output {
        ReturnType::Default => {}
        ReturnType::Type(_, ty) => {
            line.push_str(" -> ");
            line.push_str(&format_type(ty));
        }
    }

    let where_clause = format_where_clause(&signature.generics.where_clause);
    if !where_clause.is_empty() {
        line.push(' ');
        line.push_str(&where_clause);
    }

    if include_body_stub {
        line.push_str(" { ... }");
    } else {
        line.push(';');
    }

    line
}

fn format_fn_arg(arg: &FnArg) -> String {
    match arg {
        FnArg::Receiver(receiver) => format_tokens(&receiver.to_token_stream()),
        FnArg::Typed(pat_type) => match &*pat_type.pat {
            Pat::Ident(pat_ident) => format!("{}: {}", pat_ident.ident, format_type(&pat_type.ty)),
            _ => format_tokens(&pat_type.to_token_stream()),
        },
    }
}

fn format_use_tree(tree: &UseTree) -> String {
    match tree {
        UseTree::Path(path) => format!("{}::{}", path.ident, format_use_tree(&path.tree)),
        UseTree::Name(name) => name.ident.to_string(),
        UseTree::Rename(rename) => format!("{} as {}", rename.ident, rename.rename),
        UseTree::Glob(_) => "*".to_string(),
        UseTree::Group(group) => format!(
            "{{{}}}",
            group
                .items
                .iter()
                .map(format_use_tree)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn format_type(ty: &Type) -> String {
    format_tokens(&ty.to_token_stream())
}

fn format_generics(generics: &syn::Generics) -> String {
    if generics.params.is_empty() {
        String::new()
    } else {
        format!("<{}>", format_tokens(&generics.params.to_token_stream()))
    }
}

fn format_where_clause(where_clause: &Option<syn::WhereClause>) -> String {
    where_clause
        .as_ref()
        .map(|clause| format_tokens(&clause.to_token_stream()))
        .unwrap_or_default()
}

fn format_tokens(tokens: &proc_macro2::TokenStream) -> String {
    normalize_spacing(&tokens.to_string())
}

fn normalize_spacing(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" ,", ",")
        .replace(" ;", ";")
        .replace(" ::", "::")
        .replace(" (", "(")
        .replace(" )", ")")
        .replace(" [", "[")
        .replace(" ]", "]")
        .replace(" <", "<")
        .replace(" >", ">")
        .replace("& ", "&")
}
