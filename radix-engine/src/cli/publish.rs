use std::fs;
use std::path::PathBuf;

use clap::{App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;
use scrypto::utils::*;

use crate::cli::get_root_dir;
use crate::ledger::*;

const ARG_FILE: &'static str = "FILE";

pub fn prepare_publish<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("publish")
        .about("Publish a new blueprint.")
        .version("1.0")
        .arg(
            Arg::with_name(ARG_FILE)
                .help("Specify the .wasm file to publish.")
                .required(true),
        )
}

pub fn handle_publish<'a>(args: &ArgMatches<'a>) {
    let file = args.value_of(ARG_FILE).unwrap();
    let code =
        fs::read(PathBuf::from(file)).expect(format!("Unable to load file: {}", file).as_str());
    let address = Address::Blueprint(sha256_twice(&code).lower_26_bytes());

    let mut ledger = FileBasedLedger::new(get_root_dir());
    if ledger.get_blueprint(address).is_some() {
        println!("Blueprint already exists: {}", address.to_string());
    } else {
        // TODO check wasm file
        ledger.put_blueprint(address, code);
        println!("New blueprint: {}", address.to_string());
    }
}
