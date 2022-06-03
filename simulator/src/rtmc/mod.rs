use clap::Parser;
use scrypto::buffer::scrypto_encode;
use std::path::PathBuf;
use transaction::manifest::compile;

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
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    CompileError(transaction::manifest::CompileError),
}

pub fn run() -> Result<(), Error> {
    let args = Args::parse();

    let content = std::fs::read_to_string(args.input).map_err(Error::IoError)?;
    let transaction = compile(&content).map_err(Error::CompileError)?;
    std::fs::write(args.output, scrypto_encode(&transaction)).map_err(Error::IoError)?;

    Ok(())
}
