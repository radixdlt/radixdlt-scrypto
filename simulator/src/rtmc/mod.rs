use clap::Parser;
use radix_engine::utils::*;
use radix_engine_common::{
    data::manifest::manifest_encode,
    network::{NetworkDefinition, ParseNetworkError},
};
use std::path::PathBuf;
use std::str::FromStr;
use transaction::manifest::{compile, compiler::compile_error_diagnostics, BlobProvider};

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

pub fn run() -> Result<(), Error> {
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

    let transaction = match compile(&content, &network, BlobProvider::new_with_blobs(blobs)) {
        Ok(transaction) => transaction,
        Err(err) => {
            eprintln!("{}", compile_error_diagnostics(&content, err));
            // If the CompileError was returned here, then:
            // - the program exit code would be 1
            //   This is fine
            // - the error would be printed to stderr
            //   We don't want this, above diagnostics are just fine.
            std::process::exit(1);
        }
    };

    validate_call_arguments_to_native_components(&transaction.instructions)
        .map_err(Error::InstructionSchemaValidationError)?;
    std::fs::write(
        args.output,
        manifest_encode(&transaction).map_err(Error::EncodeError)?,
    )
    .map_err(Error::IoError)?;

    Ok(())
}
