use std::io;
use std::path::Path;

use super::{
    contains_symbol_match, is_symbol_line_match, load_index, IndexedDocument, RepoInventory,
    SearchHit,
};

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

pub fn matching_documents(
    root: &Path,
    query: &str,
    limit: usize,
) -> io::Result<Vec<IndexedDocument>> {
    let capped_limit = limit.clamp(1, 10);
    let normalized_query = query.to_ascii_lowercase();
    let index = load_index(root)?;

    let mut symbol_documents = Vec::new();
    let mut text_documents = Vec::new();

    for file in index.files {
        if !file
            .contents
            .to_ascii_lowercase()
            .contains(&normalized_query)
        {
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
