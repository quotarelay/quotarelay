use serde_json::Value;

#[test]
fn local_cli_usage_returns_console_snapshot() {
    let mut output = Vec::new();
    super::run_cli(["usage".to_string()], &mut output).expect("cli usage should succeed");

    let usage: Value = serde_json::from_slice(&output).expect("usage payload should parse");
    assert_eq!(usage["ok"], true);
    assert_eq!(usage["command"], "usage");
    assert_eq!(usage["result"]["title"], "Quotarelay console usage");
    assert!(usage["result"]["status"]
        .as_str()
        .expect("status should be a string")
        .contains("Backend: Ready | Repo: Not configured"));
    assert_eq!(
        usage["result"]["provider_calls"]["sparkline"]
            .as_array()
            .map(|items| items.len()),
        Some(8)
    );
    assert_eq!(usage["result"]["system_health"]["provider_calls"], "none");
    assert_eq!(
        usage["result"]["tool_usage"]
            .as_array()
            .map(|items| items.len()),
        Some(5)
    );
    assert_eq!(
        usage["result"]["collapsed"][3]["value"],
        "Provider calls: none"
    );
}
