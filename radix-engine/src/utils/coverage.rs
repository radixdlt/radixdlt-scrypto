use crate::internal_prelude::*;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

#[cfg(feature = "coverage")]
pub fn save_coverage_data(blueprint_name: &String, coverage_data: &Vec<u8>) {
    let Some(coverage_data_directory) = env::var_os("COVERAGE_DIRECTORY") else {
        return;
    };
    let blueprint_coverage_data_directory =
        PathBuf::from(coverage_data_directory).join(blueprint_name);

    // Check if the blueprint directory exists, if not create it. Ignore the error since we could
    // have a race condition when multiple tests are running and trying to create the same dir.
    if !blueprint_coverage_data_directory.exists() {
        let _ = fs::create_dir(&blueprint_coverage_data_directory);
    }

    let coverage_hash = hash(&coverage_data);
    let profraw_file_name = coverage_hash
        .0
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();
    let profraw_file_path = blueprint_coverage_data_directory
        .join(PathBuf::from(profraw_file_name).with_extension("profraw"));
    std::fs::write(profraw_file_path, coverage_data).expect("Failed to write the coverage data");
}
