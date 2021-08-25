mod file;

pub use file::FileBasedLedger;

/// Returns the ROOT data directory.
pub fn get_data_dir() -> std::path::PathBuf {
    let mut path = dirs::home_dir().expect("Unable to retrieve user's home folder");
    path.push(".radix-engine-simulator");
    path
}
