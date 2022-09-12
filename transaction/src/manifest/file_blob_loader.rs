use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::manifest::BlobLoader;

/// A blob loader that loads from local file system.
///
/// **WARNING:** This is intended for local development only as it attempts to read "any" file
/// based on user input, which may introduce security issues.
pub struct FileBlobLoader {
    root: PathBuf,
}

impl FileBlobLoader {
    pub fn new<T: AsRef<Path>>(root: T) -> Self {
        Self {
            root: PathBuf::from(root.as_ref()),
        }
    }

    pub fn with_current_dir() -> Self {
        Self::new(env::current_dir().expect("Unable to get current dir"))
    }
}

impl BlobLoader for FileBlobLoader {
    fn load(&self, key: &str) -> Option<Vec<u8>> {
        let mut path = self.root.clone();
        path.push(key);

        if let Ok(contents) = fs::read(path) {
            Some(contents)
        } else {
            None
        }
    }
}
