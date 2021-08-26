mod file;

pub use file::FileBasedLedger;

/// Returns the data directory.
pub fn get_data_dir() -> std::path::PathBuf {
    let mut path = dirs::home_dir().expect("Unable to retrieve user's home folder");
    path.push(".radix-engine-simulator");
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Unable to create folder");
    }
    path
}

/// Returns the config file.
pub fn get_config_json() -> std::path::PathBuf {
    let mut dir = get_data_dir();
    dir.push("config");
    dir.with_extension("json")
}
