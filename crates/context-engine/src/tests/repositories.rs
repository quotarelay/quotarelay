use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use super::common::temp_repo;
use crate::*;

#[test]
fn register_and_list_repositories_are_persistent_and_deduplicated() {
    let state_root = temp_repo();
    let repo_one = temp_repo();
    let repo_two = temp_repo();

    let first =
        register_repository(&state_root, &repo_one).expect("first registration should succeed");
    let second =
        register_repository(&state_root, &repo_two).expect("second registration should succeed");
    let duplicate =
        register_repository(&state_root, &repo_one).expect("duplicate registration should succeed");
    let listed = list_registered_repositories(&state_root, 10).expect("listing should succeed");

    assert_eq!(listed.len(), 2);
    assert_eq!(duplicate.repository, first.repository);
    assert_eq!(listed[0].root, first.repository.root);
    assert_eq!(listed[1].root, second.repository.root);
    assert!(state_root
        .join(".quotarelay")
        .join("registered_repositories.json")
        .exists());
}

#[test]
fn register_repository_missing_root_returns_not_found() {
    let state_root = temp_repo();
    let missing_root = state_root.join("missing-repo");

    let error = register_repository(&state_root, &missing_root)
        .expect_err("missing repo root should fail registration");

    assert_eq!(error.kind(), ErrorKind::NotFound);
}

#[test]
fn list_registered_repositories_is_bounded() {
    let state_root = temp_repo();
    let repo_roots = (0..6).map(|_| temp_repo()).collect::<Vec<_>>();
    for repo_root in &repo_roots {
        register_repository(&state_root, repo_root).expect("registration should succeed");
    }

    let listed = list_registered_repositories(&state_root, 3).expect("listing should succeed");

    assert_eq!(listed.len(), 3);
}

#[cfg(windows)]
#[test]
fn register_repository_normalizes_windows_path_variants() {
    let state_root = temp_repo();
    let repo_root = temp_repo();
    let forward_slash_root = PathBuf::from(repo_root.to_string_lossy().replace('\\', "/"));
    let dotted_root = PathBuf::from(format!("{}\\.", repo_root.to_string_lossy()));

    let first = register_repository(&state_root, &forward_slash_root)
        .expect("forward slash registration should succeed");
    let duplicate =
        register_repository(&state_root, &dotted_root).expect("dotted registration should succeed");
    let listed = list_registered_repositories(&state_root, 10).expect("listing should succeed");
    let detail = registered_repository_detail(&state_root, &dotted_root)
        .expect("detail lookup should succeed")
        .expect("registered repository should exist");

    assert_eq!(duplicate.repository, first.repository);
    assert_eq!(listed.len(), 1);
    assert_eq!(detail.repository.id, first.repository.id);
    assert_eq!(detail.repository.root, first.repository.root);
}

#[test]
fn remove_registered_repository_updates_persistent_state() {
    let state_root = temp_repo();
    let repo_root = temp_repo();

    let registered =
        register_repository(&state_root, &repo_root).expect("registration should succeed");
    let removed =
        remove_registered_repository(&state_root, &repo_root).expect("removal should succeed");
    let listed = list_registered_repositories(&state_root, 10).expect("listing should succeed");

    assert_eq!(removed.repository, Some(registered.repository));
    assert!(listed.is_empty());
}

#[test]
fn remove_unregistered_existing_repository_is_explicit_noop() {
    let state_root = temp_repo();
    let registered_root = temp_repo();
    let unregistered_root = temp_repo();

    register_repository(&state_root, &registered_root).expect("registration should succeed");
    let removed = remove_registered_repository(&state_root, &unregistered_root)
        .expect("missing removal should succeed");
    let listed = list_registered_repositories(&state_root, 10).expect("listing should succeed");

    assert!(removed.repository.is_none());
    assert_eq!(listed.len(), 1);
    assert_eq!(
        listed[0].root,
        registered_root
            .canonicalize()
            .expect("registered root should canonicalize")
            .to_string_lossy()
    );
}

#[test]
fn update_registered_repository_metadata_changes_display_name_only() {
    let state_root = temp_repo();
    let repo_root = temp_repo();

    let registered =
        register_repository(&state_root, &repo_root).expect("registration should succeed");
    let updated =
        update_registered_repository_metadata(&state_root, &repo_root, Some("Workspace API"))
            .expect("metadata update should succeed")
            .repository
            .expect("registered repository should be updated");

    assert_eq!(updated.id, registered.repository.id);
    assert_eq!(updated.root, registered.repository.root);
    assert_eq!(updated.name, "Workspace API");

    let listed = list_registered_repositories(&state_root, 10).expect("listing should succeed");
    assert_eq!(listed[0].name, "Workspace API");
    assert_eq!(listed[0].root, registered.repository.root);

    let missing_root = temp_repo();
    assert!(
        update_registered_repository_metadata(&state_root, &missing_root, Some("Missing"))
            .expect("missing metadata update should succeed")
            .repository
            .is_none()
    );
}

#[test]
fn registered_repository_state_reports_sync_and_recent_run_truth() {
    let state_root = temp_repo();
    let repo_root = temp_repo();
    fs::create_dir(repo_root.join("src")).expect("src directory should write");
    fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");
    fs::write(
        repo_root.join("src").join("lib.rs"),
        "pub mod api;\npub struct Widget;\n",
    )
    .expect("rust file should write");

    register_repository(&state_root, &repo_root).expect("registration should succeed");
    repo_index::sync_repo(&repo_root).expect("sync should succeed");
    assemble_context(&repo_root, "needle", 2).expect("assembly should succeed");

    let state =
        registered_repository_state(&state_root, 5).expect("repository state should succeed");

    assert_eq!(state.len(), 1);
    assert_eq!(state[0].sync.status, RepositorySyncStatus::Indexed);
    assert_eq!(state[0].sync.indexed_files, 2);
    assert!(state[0].sync.indexed_at_epoch_ms.is_some());
    let repo_map = state[0]
        .repo_map
        .as_ref()
        .expect("indexed repository should include repo map");
    assert!(repo_map
        .directories
        .iter()
        .any(|directory| directory.path == "src" && directory.indexed_files == 1));
    assert!(repo_map.rust_files[0]
        .symbols
        .iter()
        .any(|symbol| symbol.contains("struct Widget")));
    assert_eq!(
        state[0]
            .recent_context_run
            .as_ref()
            .map(|run| run.query.as_str()),
        Some("needle")
    );
}

#[test]
fn registered_repository_detail_reports_one_repo_truth() {
    let state_root = temp_repo();
    let repo_root = temp_repo();
    let missing_root = temp_repo();
    fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");

    let registered =
        register_repository(&state_root, &repo_root).expect("registration should succeed");
    repo_index::sync_repo(&repo_root).expect("sync should succeed");
    assemble_context(&repo_root, "needle", 2).expect("assembly should succeed");

    let detail = registered_repository_detail(&state_root, &repo_root)
        .expect("repository detail should succeed")
        .expect("registered repository detail should exist");

    assert_eq!(detail.repository, registered.repository);
    assert_eq!(detail.sync.status, RepositorySyncStatus::Indexed);
    assert_eq!(detail.sync.indexed_files, 1);
    assert_eq!(
        detail
            .recent_context_run
            .as_ref()
            .map(|run| run.query.as_str()),
        Some("needle")
    );
    assert!(registered_repository_detail(&state_root, &missing_root)
        .expect("missing repository detail should succeed")
        .is_none());
}

#[test]
fn assemble_context_for_registered_repositories_is_bounded_per_repo() {
    let state_root = temp_repo();
    let repo_one = temp_repo();
    let repo_two = temp_repo();
    fs::write(repo_one.join("alpha.txt"), "needle one\nneedle two\n")
        .expect("repo one file should write");
    fs::write(repo_two.join("beta.txt"), "needle three\nneedle four\n")
        .expect("repo two file should write");

    register_repository(&state_root, &repo_one).expect("repo one registration should succeed");
    register_repository(&state_root, &repo_two).expect("repo two registration should succeed");
    repo_index::sync_repo(&repo_one).expect("repo one sync should succeed");
    repo_index::sync_repo(&repo_two).expect("repo two sync should succeed");

    let assembly = assemble_context_for_registered_repositories(
        &state_root,
        &[repo_one.clone(), repo_two.clone()],
        "needle",
        1,
    )
    .expect("multi repo assembly should succeed");

    assert_eq!(assembly.query, "needle");
    assert_eq!(assembly.repositories.len(), 2);
    assert_eq!(assembly.repositories[0].assembly.snippets.len(), 1);
    assert_eq!(assembly.repositories[1].assembly.snippets.len(), 1);
    assert!(assembly.repositories[0]
        .assembly
        .omissions
        .iter()
        .any(|omission| omission.kind == OmissionReasonKind::ItemLimitReached));
}

#[test]
fn workspace_profiles_persist_repo_groups_and_default_limits() {
    let state_root = temp_repo();
    let repo_one = temp_repo();
    let repo_two = temp_repo();

    let saved = save_workspace_profile(
        &state_root,
        "local-api",
        &[repo_one.clone(), repo_two.clone()],
        RetrievalMode::TaskCapsule,
        9,
        8,
    )
    .expect("workspace profile should save");

    assert_eq!(saved.profile.name, "local-api");
    assert_eq!(saved.profile.repo_roots.len(), 2);
    assert_eq!(saved.profile.default_mode, RetrievalMode::TaskCapsule);
    assert_eq!(saved.profile.default_limit, 5);
    assert_eq!(saved.profile.per_repo_limit, 5);

    let replacement = save_workspace_profile(
        &state_root,
        "local-api",
        &[repo_one],
        RetrievalMode::ExactSearch,
        2,
        1,
    )
    .expect("workspace profile replacement should save");
    let profiles =
        list_workspace_profiles(&state_root, 10).expect("workspace profiles should list");

    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0], replacement.profile);
    assert_eq!(profiles[0].default_mode, RetrievalMode::ExactSearch);
    assert_eq!(profiles[0].default_limit, 2);
    assert_eq!(profiles[0].per_repo_limit, 1);
    assert!(state_root
        .join(".quotarelay")
        .join("workspace_profiles.json")
        .exists());
}

#[test]
fn team_policy_profiles_persist_replace_and_bound_public_team_config() {
    let state_root = temp_repo();
    let many_guardrails = (0..25)
        .map(|index| format!("guardrail {index}"))
        .collect::<Vec<_>>();

    let saved = save_team_policy_profile(
        &state_root,
        "backend-team",
        &many_guardrails,
        &[
            "cargo test -p context-engine".to_string(),
            " ".to_string(),
            "npm --prefix web/controlplane run build".to_string(),
        ],
        &["examples/mcp-client-presets/generic-stdio.json".to_string()],
        false,
    )
    .expect("team policy profile should save");

    assert_eq!(saved.profile.name, "backend-team");
    assert_eq!(saved.profile.guardrails.len(), 20);
    assert_eq!(saved.profile.validation_recipes.len(), 2);
    assert_eq!(saved.profile.mcp_client_presets.len(), 1);
    assert!(!saved.profile.allow_source_upload);

    let replacement = save_team_policy_profile(
        &state_root,
        "backend-team",
        &["Keep source local by default.".to_string()],
        &[
            "powershell -NoProfile -ExecutionPolicy Bypass -File scripts\\release-check.ps1"
                .to_string(),
        ],
        &[],
        true,
    )
    .expect("team policy profile replacement should save");
    let profiles =
        list_team_policy_profiles(&state_root, 10).expect("team policy profiles should list");

    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0], replacement.profile);
    assert!(profiles[0].allow_source_upload);
    assert!(state_root
        .join(".quotarelay")
        .join("team_policy_profiles.json")
        .exists());
}

#[test]
fn local_state_inspection_counts_team_policy_without_dumping_contents() {
    let state_root = temp_repo();
    save_team_policy_profile(
        &state_root,
        "privacy-team",
        &["never upload repository contents by default".to_string()],
        &[],
        &[],
        false,
    )
    .expect("team policy profile should save");

    let inspection = inspect_local_state(&state_root).expect("local state should inspect");

    assert!(inspection.team_policy_profiles.present);
    assert_eq!(inspection.team_policy_profiles.item_count, 1);
}
