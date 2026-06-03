use crate::recommend_validation;

#[test]
fn validation_recommendations_match_repo_owned_paths() {
    let result = recommend_validation(&[
        "crates/context-engine/src/lib.rs".to_string(),
        "apps/mcp-server/src/cli.rs".to_string(),
        "web/controlplane/src/truth.ts".to_string(),
    ]);
    let commands = result
        .recommendations
        .iter()
        .map(|recommendation| recommendation.command.as_str())
        .collect::<Vec<_>>();

    assert!(commands.contains(&"cargo test -p context-engine"));
    assert!(commands.contains(&"cargo test -p mcp-server"));
    assert!(commands.contains(&"npm --prefix web/controlplane run build"));
    assert!(commands.contains(
        &"powershell -NoProfile -ExecutionPolicy Bypass -File scripts\\check-line-counts.ps1"
    ));
    assert_eq!(result.omitted_path_count, 0);
}

#[test]
fn validation_recommendations_cover_docs_and_unknown_paths() {
    let docs = recommend_validation(&["docs/MVP_SCOPE.md".to_string()]);
    assert_eq!(docs.recommendations.len(), 1);
    assert_eq!(
        docs.recommendations[0].command,
        "powershell -NoProfile -ExecutionPolicy Bypass -File scripts\\check-line-counts.ps1"
    );

    let unknown = recommend_validation(&["scratch/outside.txt".to_string()]);
    assert_eq!(unknown.recommendations.len(), 1);
    assert_eq!(
        unknown.recommendations[0].command,
        "powershell -NoProfile -ExecutionPolicy Bypass -File scripts\\clean-check.ps1"
    );
}
