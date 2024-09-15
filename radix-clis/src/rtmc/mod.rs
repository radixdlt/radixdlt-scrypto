use clap::Parser;
use radix_common::prelude::*;
use radix_engine::utils::*;
use radix_transactions::manifest::*;
use radix_transactions::prelude::*;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

/// Radix transaction manifest compiler
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, name = "rtmc")]
pub struct Args {
    /// Path to the output file
    #[clap(short, long)]
    output: PathBuf,

    /// Network to Use [Simulator | Alphanet | Mainnet]
    #[clap(short, long)]
    network: Option<String>,

    /// The paths to blobs
    #[clap(short, long, multiple = true)]
    blobs: Option<Vec<String>>,

    /// Input file
    #[clap(required = true)]
    input: PathBuf,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    EncodeError(sbor::EncodeError),
    ParseNetworkError(ParseNetworkError),
    InstructionSchemaValidationError(radix_engine::utils::LocatedInstructionSchemaValidationError),
}

impl fmt::Display for Error {
    // TODO Implement pretty error printing
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Error> for String {
    fn from(err: Error) -> String {
        err.to_string()
    }
}

pub fn run() -> Result<(), String> {
    let args = Args::parse();

    let content = std::fs::read_to_string(&args.input).map_err(Error::IoError)?;
    let network = match args.network {
        Some(n) => NetworkDefinition::from_str(&n).map_err(Error::ParseNetworkError)?,
        None => NetworkDefinition::simulator(),
    };
    let mut blobs = Vec::new();
    if let Some(paths) = args.blobs {
        for path in paths {
            blobs.push(std::fs::read(path).map_err(Error::IoError)?);
        }
    }

    let manifest = compile_manifest_with_pretty_error::<TransactionManifestV1>(
        &content,
        &network,
        BlobProvider::new_with_blobs(blobs),
        CompileErrorDiagnosticsStyle::TextTerminalColors,
    )?;

    validate_call_arguments_to_native_components(&manifest)
        .map_err(Error::InstructionSchemaValidationError)?;
    std::fs::write(
        args.output,
        manifest_encode(&manifest).map_err(Error::EncodeError)?,
    )
    .map_err(Error::IoError)?;

    Ok(())
}
