#[cfg(not(debug_assertions))]
pub const BUNDLE_ID: &str = "org.destroy.dictation.community";
#[cfg(debug_assertions)]
pub const BUNDLE_ID: &str = "org.destroy.dictation.community.dev";
pub fn is_own_bundle_id(id: &str) -> bool {
    id == "org.destroy.dictation.community" || id == "org.destroy.dictation.community.dev"
}
pub fn config_path() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join("Library/Application Support")
        .join(BUNDLE_ID)
        .join("connection.json")
}
