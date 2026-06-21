use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::recommend_validation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnboardingPack {
    pub repo_name: String,
    pub indexed_files: usize,
    pub sample_directories: Vec<String>,
    pub rust_entrypoints: Vec<String>,
    pub recommended_validation: Vec<String>,
    pub handoff_templates: Vec<String>,
    pub privacy_notes: Vec<String>,
    pub omitted_directory_count: usize,
    pub omitted_rust_file_count: usize,
}

pub fn onboarding_pack(root: &Path, touched_paths: &[String]) -> io::Result<OnboardingPack> {
    let map = repo_index::repo_map(root)?;
    let recommended_validation = recommend_validation(touched_paths)
        .recommendations
        .into_iter()
        .map(|recommendation| recommendation.command)
        .collect();

    Ok(OnboardingPack {
        repo_name: crate::repo_name(root),
        indexed_files: map.indexed_files,
        sample_directories: map
            .directories
            .iter()
            .take(8)
            .map(|directory| directory.path.clone())
            .collect(),
        rust_entrypoints: map
            .rust_files
            .iter()
            .take(8)
            .map(|file| {
                if file.symbols.is_empty() {
                    file.path.clone()
                } else {
                    format!("{}: {}", file.path, file.symbols.join(", "))
                }
            })
            .collect(),
        recommended_validation,
        handoff_templates: vec![
            "bug_fix".to_string(),
            "feature_slice".to_string(),
            "review".to_string(),
            "refactor".to_string(),
            "release".to_string(),
        ],
        privacy_notes: vec![
            "Local onboarding uses repository index metadata and validation commands only."
                .to_string(),
            "It does not upload source code, snippets, context packets, caches, or .quotarelay state."
                .to_string(),
            "Run sync_repo explicitly before onboarding when repository files change.".to_string(),
        ],
        omitted_directory_count: map.omitted_directory_count,
        omitted_rust_file_count: map.omitted_rust_file_count,
    })
}
