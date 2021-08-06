use std::path::PathBuf;

pub fn get_root_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("Unable to determine your home folder");
    path.push("radix-engine");
    path
}
