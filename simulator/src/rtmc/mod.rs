use clap::Parser;
use radix_engine::types::*;
use std::path::PathBuf;
use std::str::FromStr;
use transaction::manifest::compile;

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
    CompileError(transaction::manifest::CompileError),
    ParseNetworkError(ParseNetworkError),
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
    let transaction = compile(&content, &network, blobs).map_err(Error::CompileError)?;
    std::fs::write(
        args.output,
        manifest_encode(&transaction).map_err(Error::EncodeError)?,
    )
    .map_err(Error::IoError)?;

    Ok(())
}
