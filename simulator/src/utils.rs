use std::path::PathBuf;

/// Returns the ROOT data directory.
pub fn get_data_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("Unable to retrieve user's home folder");
    path.push(".scrypto-simulator");
    path
}
