use clap::Parser;
use scrypto::buffer::scrypto_encode;
use scrypto::core::{NetworkDefinition, NetworkError};
use std::path::PathBuf;
use std::str::FromStr;
use transaction::manifest::compile;

use crate::utils::FileBlobLoader;

/// Radix transaction manifest compiler
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, name = "rtmc")]
pub struct Args {
    /// Path to the output file
    #[clap(short, long)]
    output: PathBuf,

    /// Input file
    #[clap(required = true)]
    input: PathBuf,

    /// Network to Use [LocalSimulator | InternalTestnet]
    #[clap(required = true)]
    network: String,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    CompileError(transaction::manifest::CompileError),
    NetworkError(NetworkError),
}

pub fn run() -> Result<(), Error> {
    let args = Args::parse();

    let content = std::fs::read_to_string(args.input).map_err(Error::IoError)?;
    let network = NetworkDefinition::from_str(&args.network).map_err(Error::NetworkError)?;
    let transaction = compile(&content, &network, &mut FileBlobLoader::with_current_dir())
        .map_err(Error::CompileError)?;
    std::fs::write(args.output, scrypto_encode(&transaction)).map_err(Error::IoError)?;

    Ok(())
}
