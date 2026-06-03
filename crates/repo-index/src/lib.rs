use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use quote::ToTokens;
use serde::{Deserialize, Serialize};
use syn::{
    Fields, File, FnArg, ImplItem, Item, Pat, ReturnType, Signature, TraitItem, Type, UseTree,
};

const MAX_INDEXED_CONTENT_BYTES: usize = 8 * 1024;
const TRUNCATED_INDEX_MARKER: &str = "\n// ... truncated indexed contents ...";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexedFile {
    path: String,
    contents: String,
    #[serde(default)]
    modified_at_epoch_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredIndex {
    indexed_at_epoch_ms: u128,
    files: Vec<IndexedFile>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LocalIgnoreConfig {
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    prefixes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct IgnoreRules {
    paths: Vec<String>,
    prefixes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInventory {
    pub indexed_at_epoch_ms: u128,
    pub indexed_files: usize,
    pub sample_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub indexed_files: usize,
    pub index_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub path: String,
    pub line_number: usize,
    pub line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexedDocument {
    pub path: String,
    pub contents: String,
}

pub fn sync_repo(root: &Path) -> io::Result<SyncResult> {
    let ignore_rules = load_ignore_rules(root)?;
    let previous_files = load_index(root)
        .map(|index| {
            index
                .files
                .into_iter()
                .map(|file| (file.path.clone(), file))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let mut discovered_paths = Vec::new();
    collect_file_paths(root, root, &ignore_rules, &mut discovered_paths)?;
    let mut files = discovered_paths
        .into_iter()
        .filter_map(|path| build_indexed_file(root, &path, &previous_files).transpose())
        .collect::<io::Result<Vec<_>>>()?;
    files.sort_by(|left, right| left.path.cmp(&right.path));

    let stored = StoredIndex {
        indexed_at_epoch_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_millis(),
        files,
    };
    let index_path = index_path(root);
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&index_path, serde_json::to_vec_pretty(&stored).map_err(io::Error::other)?)?;

    Ok(SyncResult {
        indexed_files: stored.files.len(),
        index_path: index_path.display().to_string(),
    })
}

pub fn search_code(root: &Path, query: &str, limit: usize) -> io::Result<Vec<SearchHit>> {
    let index = load_index(root)?;
    let normalized_query = query.to_ascii_lowercase();
    let capped_limit = limit.clamp(1, 10);
    let mut symbol_hits = Vec::new();
    let mut text_hits = Vec::new();

    for file in index.files {
        for (offset, line) in file.contents.lines().enumerate() {
            if line.to_ascii_lowercase().contains(&normalized_query) {
                let hit = SearchHit {
                    path: file.path.clone(),
                    line_number: offset + 1,
                    line: line.to_string(),
                };
                if is_symbol_line_match(line, query) {
                    symbol_hits.push(hit);
                } else {
                    text_hits.push(hit);
                }
                if symbol_hits.len() + text_hits.len() >= capped_limit {
                    break;
                }
            }
        }
        if symbol_hits.len() + text_hits.len() >= capped_limit {
            break;
        }
    }

    symbol_hits.extend(text_hits);
    symbol_hits.truncate(capped_limit);
    Ok(symbol_hits)
}

pub fn repo_inventory(root: &Path) -> io::Result<RepoInventory> {
    let index = load_index(root)?;
    let sample_paths = index
        .files
        .iter()
        .take(5)
        .map(|file| file.path.clone())
        .collect();

    Ok(RepoInventory {
        indexed_at_epoch_ms: index.indexed_at_epoch_ms,
        indexed_files: index.files.len(),
        sample_paths,
    })
}

pub fn indexed_documents(root: &Path, limit: usize) -> io::Result<Vec<IndexedDocument>> {
    let capped_limit = limit.clamp(1, 10);
    let index = load_index(root)?;

    Ok(index
        .files
        .into_iter()
        .take(capped_limit)
        .map(|file| IndexedDocument {
            path: file.path,
            contents: file.contents,
        })
        .collect())
}

pub fn matching_documents(root: &Path, query: &str, limit: usize) -> io::Result<Vec<IndexedDocument>> {
    let capped_limit = limit.clamp(1, 10);
    let normalized_query = query.to_ascii_lowercase();
    let index = load_index(root)?;

    let mut symbol_documents = Vec::new();
    let mut text_documents = Vec::new();

    for file in index.files {
        if !file.contents.to_ascii_lowercase().contains(&normalized_query) {
            continue;
        }

        let document = IndexedDocument {
            path: file.path,
            contents: file.contents,
        };

        if contains_symbol_match(&document.contents, query) {
            symbol_documents.push(document);
        } else {
            text_documents.push(document);
        }

        if symbol_documents.len() + text_documents.len() >= capped_limit {
            break;
        }
    }

    symbol_documents.extend(text_documents);
    symbol_documents.truncate(capped_limit);
    Ok(symbol_documents)
}

fn load_index(root: &Path) -> io::Result<StoredIndex> {
    let bytes = fs::read(index_path(root))?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}

fn collect_file_paths(
    root: &Path,
    current: &Path,
    ignore_rules: &IgnoreRules,
    files: &mut Vec<PathBuf>,
) -> io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let relative = relative_index_path(root, &path)?;

        if file_type.is_dir() {
            if should_skip_dir(&path) || should_ignore_path(&relative, ignore_rules) {
                continue;
            }
            collect_file_paths(root, &path, ignore_rules, files)?;
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        if should_ignore_path(&relative, ignore_rules) {
            continue;
        }

        files.push(path);
    }

    Ok(())
}

fn load_ignore_rules(root: &Path) -> io::Result<IgnoreRules> {
    let path = ignore_config_path(root);
    if !path.exists() {
        return Ok(IgnoreRules::default());
    }

    let bytes = fs::read(path)?;
    let config: LocalIgnoreConfig = serde_json::from_slice(&bytes).map_err(io::Error::other)?;

    Ok(IgnoreRules {
        paths: config
            .paths
            .into_iter()
            .filter_map(|entry| normalize_ignore_entry(&entry))
            .collect(),
        prefixes: config
            .prefixes
            .into_iter()
            .filter_map(|entry| normalize_ignore_entry(&entry))
            .collect(),
    })
}

fn normalize_ignore_entry(entry: &str) -> Option<String> {
    let normalized = entry
        .trim()
        .replace('\\', "/")
        .trim_matches('/')
        .to_string();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn should_ignore_path(relative: &str, rules: &IgnoreRules) -> bool {
    rules.paths.iter().any(|path| path == relative)
        || rules.prefixes.iter().any(|prefix| {
            relative == prefix || relative.strip_prefix(prefix).is_some_and(|rest| rest.starts_with('/'))
        })
}

fn build_indexed_file(
    root: &Path,
    path: &Path,
    previous_files: &HashMap<String, IndexedFile>,
) -> io::Result<Option<IndexedFile>> {
    let relative = relative_index_path(root, path)?;
    let modified_at_epoch_ms = modified_at_epoch_ms(path)?;

    if let Some(existing) = previous_files.get(&relative) {
        if existing.modified_at_epoch_ms == modified_at_epoch_ms {
            return Ok(Some(existing.clone()));
        }
    }

    let Ok(contents) = fs::read_to_string(path) else {
        return Ok(None);
    };
    let indexed_contents = build_indexed_contents(path, &contents);
    Ok(Some(IndexedFile {
        path: relative,
        contents: indexed_contents,
        modified_at_epoch_ms,
    }))
}

fn build_indexed_contents(path: &Path, contents: &str) -> String {
    let indexed = match path.extension().and_then(|extension| extension.to_str()) {
        Some("rs") => build_rust_capsule(contents).unwrap_or_else(|| contents.to_string()),
        _ => contents.to_string(),
    };
    bound_indexed_contents(indexed)
}

fn modified_at_epoch_ms(path: &Path) -> io::Result<u128> {
    fs::metadata(path)?
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)
        .map(|duration| duration.as_millis())
}

fn contains_symbol_match(input: &str, query: &str) -> bool {
    input.lines().any(|line| is_symbol_line_match(line, query))
}

fn is_symbol_line_match(line: &str, query: &str) -> bool {
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

fn build_rust_capsule(contents: &str) -> Option<String> {
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
                    field.ident.as_ref().map(|ident| {
                        format!("{}: {}", ident, format_type(&field.ty))
                    })
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
                        field.ident.as_ref().map(|ident| {
                            format!("{}: {}", ident, format_type(&field.ty))
                        })
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

fn bound_indexed_contents(contents: String) -> String {
    if contents.len() <= MAX_INDEXED_CONTENT_BYTES {
        return contents;
    }

    let cutoff = MAX_INDEXED_CONTENT_BYTES.saturating_sub(TRUNCATED_INDEX_MARKER.len());
    let mut boundary = cutoff;
    while boundary > 0 && !contents.is_char_boundary(boundary) {
        boundary -= 1;
    }

    let mut bounded = contents[..boundary].to_string();
    bounded.push_str(TRUNCATED_INDEX_MARKER);
    bounded
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | ".quotarelay" | "target" | "node_modules")
    )
}

fn relative_index_path(root: &Path, path: &Path) -> io::Result<String> {
    Ok(path
        .strip_prefix(root)
        .map_err(io::Error::other)?
        .to_string_lossy()
        .replace('\\', "/"))
}

fn index_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("index.json")
}

fn ignore_config_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("ignore.json")
}

#[cfg(test)]
mod tests {
    use super::{
        indexed_documents, load_index, matching_documents, repo_inventory, search_code, sync_repo,
        MAX_INDEXED_CONTENT_BYTES, TRUNCATED_INDEX_MARKER,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sync_persists_and_search_returns_bounded_hits() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle one\nneedle two\n").expect("alpha file should write");
        fs::write(root.join("beta.txt"), "needle three\n").expect("beta file should write");

        let sync = sync_repo(&root).expect("sync should succeed");
        assert_eq!(sync.indexed_files, 2);
        assert!(root.join(".quotarelay").join("index.json").exists());

        let hits = search_code(&root, "needle", 2).expect("search should succeed");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].path, "alpha.txt");
        assert_eq!(hits[0].line_number, 1);

        let inventory = repo_inventory(&root).expect("inventory should succeed");
        assert_eq!(inventory.indexed_files, 2);
        assert_eq!(inventory.sample_paths, vec!["alpha.txt", "beta.txt"]);
        assert!(inventory.indexed_at_epoch_ms > 0);
    }

    #[test]
    fn sync_rust_files_into_structural_capsules_with_bounded_output() {
        let root = temp_repo();
        let large_body = "println!(\"padding\");\n".repeat(900);
        fs::write(
            root.join("lib.rs"),
            format!(
                "use std::fmt;\n\nstruct Widget {{\n    id: usize,\n}}\n\nfn plan(input: &str) -> usize {{\n    let body_only_term = input.len();\n    {large_body}    body_only_term\n}}\n"
            ),
        )
        .expect("rust file should write");

        sync_repo(&root).expect("sync should succeed");

        let index = load_index(&root).expect("index should load");
        assert_eq!(index.files.len(), 1);
        assert!(index.files[0].contents.contains("use std::fmt;"));
        assert!(index.files[0].contents.contains("struct Widget { id: usize }"));
        assert!(index.files[0].contents.contains("fn plan(input: &str) -> usize;"));
        assert!(!index.files[0].contents.contains("body_only_term = input.len()"));
        assert!(index.files[0].contents.len() <= MAX_INDEXED_CONTENT_BYTES);
        assert!(
            index.files[0].contents.len() < MAX_INDEXED_CONTENT_BYTES
                || index.files[0].contents.ends_with(TRUNCATED_INDEX_MARKER)
        );
    }

    #[test]
    fn malformed_rust_files_fall_back_to_raw_text_search() {
        let root = temp_repo();
        fs::write(
            root.join("broken.rs"),
            "fn broken( {\n    let raw_fallback_term = 1;\n}\n",
        )
        .expect("broken rust file should write");

        sync_repo(&root).expect("sync should succeed");

        let hits = search_code(&root, "raw_fallback_term", 2).expect("search should succeed");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "broken.rs");
        assert_eq!(hits[0].line_number, 2);
    }

    #[test]
    fn document_queries_return_bounded_indexed_contents() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "alpha term\n").expect("alpha file should write");
        fs::write(root.join("beta.txt"), "beta term\n").expect("beta file should write");

        sync_repo(&root).expect("sync should succeed");

        let docs = indexed_documents(&root, 1).expect("documents should load");
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].path, "alpha.txt");

        let matches = matching_documents(&root, "beta", 2).expect("matching documents should load");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "beta.txt");
        assert_eq!(matches[0].contents, "beta term\n");
    }

    #[test]
    fn sync_repo_updates_incrementally_for_add_change_delete() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "alpha one\n").expect("alpha file should write");
        fs::write(root.join("beta.txt"), "beta one\n").expect("beta file should write");

        sync_repo(&root).expect("initial sync should succeed");
        let first_index = load_index(&root).expect("first index should load");
        let first_beta = first_index
            .files
            .iter()
            .find(|file| file.path == "beta.txt")
            .expect("beta should exist")
            .modified_at_epoch_ms;

        thread::sleep(Duration::from_millis(20));
        fs::remove_file(root.join("alpha.txt")).expect("alpha should delete");
        fs::write(root.join("beta.txt"), "beta two\n").expect("beta should update");
        fs::write(root.join("gamma.txt"), "gamma one\n").expect("gamma should write");

        sync_repo(&root).expect("incremental sync should succeed");
        let second_index = load_index(&root).expect("second index should load");

        assert_eq!(second_index.files.len(), 2);
        assert!(second_index.files.iter().all(|file| file.path != "alpha.txt"));
        assert!(second_index.files.iter().any(|file| file.path == "gamma.txt"));
        let second_beta = second_index
            .files
            .iter()
            .find(|file| file.path == "beta.txt")
            .expect("beta should still exist");
        assert_eq!(second_beta.contents, "beta two\n");
        assert!(second_beta.modified_at_epoch_ms >= first_beta);
    }

    #[test]
    fn sync_repo_respects_explicit_local_ignore_config_after_sync() {
        let root = temp_repo();
        let generated_dir = root.join("generated");
        fs::create_dir_all(&generated_dir).expect("generated dir should create");
        fs::write(root.join("alpha.txt"), "alpha keep\n").expect("alpha file should write");
        fs::write(root.join("secret.txt"), "secret needle\n").expect("secret file should write");
        fs::write(generated_dir.join("artifact.txt"), "artifact needle\n")
            .expect("artifact file should write");

        sync_repo(&root).expect("initial sync should succeed");
        assert_eq!(search_code(&root, "needle", 10).expect("initial search should succeed").len(), 2);

        let state_dir = root.join(".quotarelay");
        fs::create_dir_all(&state_dir).expect("state dir should create");
        fs::write(
            state_dir.join("ignore.json"),
            r#"{
  "paths": ["secret.txt"],
  "prefixes": ["generated"]
}"#,
        )
        .expect("ignore config should write");

        assert_eq!(
            search_code(&root, "needle", 10)
                .expect("search before explicit resync should still use old index")
                .len(),
            2
        );

        let resync = sync_repo(&root).expect("resync should honor ignore config");
        assert_eq!(resync.indexed_files, 1);

        let index = load_index(&root).expect("index should load");
        assert_eq!(index.files.len(), 1);
        assert_eq!(index.files[0].path, "alpha.txt");
        assert!(
            search_code(&root, "needle", 10)
                .expect("ignored search should succeed")
                .is_empty()
        );
    }

    #[test]
    fn sync_repo_normalizes_windows_style_ignore_entries() {
        let root = temp_repo();
        let generated_dir = root.join("generated");
        fs::create_dir_all(&generated_dir).expect("generated dir should create");
        fs::write(generated_dir.join("artifact.txt"), "artifact needle\n")
            .expect("artifact file should write");
        fs::write(root.join("keep.txt"), "keep needle\n").expect("keep file should write");
        let state_dir = root.join(".quotarelay");
        fs::create_dir_all(&state_dir).expect("state dir should create");
        fs::write(
            state_dir.join("ignore.json"),
            r#"{
  "paths": ["generated\\artifact.txt"]
}"#,
        )
        .expect("ignore config should write");

        let sync = sync_repo(&root).expect("sync should honor normalized ignore path");
        assert_eq!(sync.indexed_files, 1);
        let hits = search_code(&root, "needle", 10).expect("search should succeed");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "keep.txt");
    }

    #[test]
    fn many_file_repos_keep_index_and_query_outputs_bounded() {
        let root = temp_repo();
        let large_tail = "padding\n".repeat(2_000);
        for index in 0..40 {
            fs::write(
                root.join(format!("file-{index:02}.txt")),
                format!("needle line {index}\n{large_tail}"),
            )
            .expect("repo file should write");
        }

        let sync = sync_repo(&root).expect("sync should succeed");
        assert_eq!(sync.indexed_files, 40);

        let stored = load_index(&root).expect("index should load");
        assert_eq!(stored.files.len(), 40);
        assert!(stored.files.iter().all(|file| file.contents.len() <= MAX_INDEXED_CONTENT_BYTES));

        let hits = search_code(&root, "needle", 3).expect("search should succeed");
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].path, "file-00.txt");

        let documents = indexed_documents(&root, 4).expect("documents should load");
        assert_eq!(documents.len(), 4);
        assert!(documents.iter().all(|document| document.contents.len() <= MAX_INDEXED_CONTENT_BYTES));

        let matches = matching_documents(&root, "needle", 5).expect("matching documents should load");
        assert_eq!(matches.len(), 5);
    }

    #[test]
    fn search_and_document_matching_prefer_symbol_hits() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "Widget appears in notes only\n")
            .expect("alpha file should write");
        fs::write(
            root.join("lib.rs"),
            "struct Widget {\n    id: usize,\n}\n",
        )
        .expect("rust file should write");

        sync_repo(&root).expect("sync should succeed");

        let hits = search_code(&root, "Widget", 2).expect("search should succeed");
        assert_eq!(hits[0].path, "lib.rs");

        let documents = matching_documents(&root, "Widget", 2).expect("matching documents should succeed");
        assert_eq!(documents[0].path, "lib.rs");
    }

    fn temp_repo() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("quotarelay-index-{unique}"));
        fs::create_dir_all(&root).expect("temp repo should create");
        root
    }
}
