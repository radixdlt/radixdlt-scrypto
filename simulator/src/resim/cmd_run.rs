use clap::Parser;
use radix_engine::utils::validate_call_arguments_to_native_components;
use regex::{Captures, Regex};
use std::env;
use std::path::PathBuf;
use transaction::manifest::{
    compile, compiler::compile_error_diagnostics, compiler::CompileErrorDiagnosticsStyle,
    BlobProvider,
};

use crate::resim::*;

/// Compiles, signs and runs a transaction manifest
#[derive(Parser, Debug)]
pub struct Run {
    /// The path to a transaction manifest file
    pub path: PathBuf,

    /// The network to use when outputting manifest, [simulator | adapanet | nebunet | mainnet]
    #[clap(short, long)]
    pub network: Option<String>,

    /// The paths to blobs
    #[clap(short, long, multiple = true)]
    pub blobs: Option<Vec<String>>,

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    pub signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    pub trace: bool,
}

impl Run {
    pub fn pre_process_manifest(manifest: &str) -> String {
        let re = Regex::new(r"\$\{(.+?)\}").unwrap();
        re.replace_all(manifest, |caps: &Captures| {
            env::var(&caps[1].trim()).unwrap_or_default()
        })
        .into()
    }

    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let manifest = std::fs::read_to_string(&self.path).map_err(Error::IOError)?;
        let pre_processed_manifest = Self::pre_process_manifest(&manifest);
        let network = match &self.network {
            Some(n) => NetworkDefinition::from_str(&n).map_err(Error::ParseNetworkError)?,
            None => NetworkDefinition::simulator(),
        };
        let mut blobs = Vec::new();
        if let Some(paths) = &self.blobs {
            for path in paths {
                blobs.push(std::fs::read(path).map_err(Error::IOError)?);
            }
        }
        let compiled_manifest = match compile(
            &pre_processed_manifest,
            &network,
            BlobProvider::new_with_blobs(blobs),
        ) {
            Ok(transaction) => transaction,
            Err(err) => {
                eprintln!(
                    "{}",
                    compile_error_diagnostics(
                        &pre_processed_manifest,
                        err,
                        CompileErrorDiagnosticsStyle::TextTerminalColors
                    )
                );

                // If the CompileError was returned here, then:
                // - the program exit code would be 1
                //   This is fine
                // - the error would be printed to stderr
                //   We don't want this, above diagnostics are just fine.
                std::process::exit(1);
            }
        };

        validate_call_arguments_to_native_components(&compiled_manifest.instructions)
            .map_err(Error::InstructionSchemaValidationError)?;

        handle_manifest(
            compiled_manifest,
            &self.signing_keys,
            &self.network,
            &None,
            self.trace,
            true,
            out,
        )
        .map(|_| ())
    }
}
