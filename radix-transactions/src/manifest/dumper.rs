use crate::internal_prelude::*;
use radix_common::network::NetworkDefinition;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use super::decompiler::decompile;

pub fn dump_manifest_to_file_system<P>(
    manifest: &impl TypedReadableManifest,
    directory_path: P,
    name: Option<&str>,
    network_definition: &NetworkDefinition,
) -> Result<(), DumpManifestError>
where
    P: AsRef<Path>,
{
    let path = directory_path.as_ref().to_owned();

    // Check that the path is a directory and not a file
    if path.is_file() {
        return Err(DumpManifestError::PathPointsToAFile(path));
    }

    // If the directory does not exist, then create it.
    create_dir_all(&path)?;

    // Decompile the transaction manifest to the manifest string and then write it to the
    // directory
    {
        let manifest_string = decompile(manifest, network_definition)?;
        let manifest_path = path.join(format!("{}.rtm", name.unwrap_or("transaction")));
        std::fs::write(manifest_path, manifest_string)?;
    }

    // Write all of the blobs to the specified path
    let blob_prefix = name.map(|n| format!("{n}-")).unwrap_or_default();
    for (hash, blob_content) in manifest.get_blobs() {
        let blob_path = path.join(format!("{blob_prefix}{hash}.blob"));
        std::fs::write(blob_path, blob_content)?;
    }

    manifest.validate(ValidationRuleset::all())?;

    Ok(())
}

#[derive(Debug)]
pub enum DumpManifestError {
    PathPointsToAFile(PathBuf),
    IoError(std::io::Error),
    DecompileError(DecompileError),
    ManifestValidationError(ManifestValidationError),
}

impl From<std::io::Error> for DumpManifestError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<DecompileError> for DumpManifestError {
    fn from(value: DecompileError) -> Self {
        Self::DecompileError(value)
    }
}

impl From<ManifestValidationError> for DumpManifestError {
    fn from(value: ManifestValidationError) -> Self {
        Self::ManifestValidationError(value)
    }
}
