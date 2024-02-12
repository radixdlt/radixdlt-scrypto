use crate::internal_prelude::*;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[cfg(feature = "coverage")]
pub fn save_coverage_data(blueprint_name: &String, coverage_data: &Vec<u8>) {
    if let Some(dir) = env::var_os("COVERAGE_DIRECTORY") {
        let mut file_path = Path::new(&dir).to_path_buf();
        file_path.push(blueprint_name);

        // Check if the blueprint directory exists, if not create it
        if !file_path.exists() {
            // error is ignored because when multiple tests are running it may fail
            fs::create_dir(&file_path).ok();
        }

        // Write .profraw binary data
        let file_name = hash(&coverage_data);
        let file_name: String = file_name.0[..16]
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect();
        file_path.push(format!("{}.profraw", file_name));
        let mut file = File::create(file_path).unwrap();
        file.write_all(&coverage_data).unwrap();
    }
}
