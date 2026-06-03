use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn temp_repo() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("quotarelay-context-{unique}"));
    std::fs::create_dir_all(&root).expect("temp repo should create");
    root
}
