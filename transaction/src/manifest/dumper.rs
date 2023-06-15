use crate::internal_prelude::*;
use radix_engine_interface::network::NetworkDefinition;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

pub fn dump_manifest_to_file_system<P>(
    manifest: &TransactionManifestV1,
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
        let manifest_string = decompile(&manifest.instructions, network_definition)?;
        let manifest_path = path.join(format!("{}.rtm", name.unwrap_or("transaction")));
        std::fs::write(manifest_path, manifest_string)?;
    }

    // Write all of the blobs to the specified path
    let blob_prefix = name.map(|n| format!("{n}-")).unwrap_or_default();
    for (hash, blob_content) in &manifest.blobs {
        let blob_path = path.join(format!("{blob_prefix}{hash}.blob"));
        std::fs::write(blob_path, blob_content)?;
    }

    // Validate the manifest
    NotarizedTransactionValidator::validate_instructions_v1(&manifest.instructions)?;

    Ok(())
}

#[derive(Debug)]
pub enum DumpManifestError {
    PathPointsToAFile(PathBuf),
    IoError(std::io::Error),
    DecompileError(DecompileError),
    TransactionValidationError(TransactionValidationError),
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

impl From<TransactionValidationError> for DumpManifestError {
    fn from(value: TransactionValidationError) -> Self {
        Self::TransactionValidationError(value)
    }
}
