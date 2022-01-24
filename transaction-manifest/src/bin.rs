extern crate transaction_manifest;

use clap::Parser;
use scrypto::buffer::scrypto_encode;
use std::fs::read_to_string;
use std::fs::write;
use std::path::PathBuf;
use transaction_manifest::compile;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the output file
    #[clap(short, long)]
    output: PathBuf,

    /// Input file
    #[clap(required = true)]
    input: PathBuf,
}

fn main() {
    let args = Args::parse();

    let content = read_to_string(args.input).unwrap();
    let transaction = compile(&content).unwrap();
    write(args.output, scrypto_encode(&transaction)).unwrap();
}
