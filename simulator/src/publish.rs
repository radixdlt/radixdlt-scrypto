use std::fs;
use std::path::PathBuf;

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::utils::*;
use uuid::Uuid;

use crate::*;
use radix_engine::execution::*;
use radix_engine::ledger::*;
use radix_engine::model::*;

const ARG_FILE: &'static str = "FILE";

/// Constructs a `publish` subcommand.
pub fn make_publish_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_PUBLISH)
        .about("Publishes a new blueprint.")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_FILE)
                .help("Specify the .wasm file to publish.")
                .required(true),
        )
}

/// Handles a `publish` request.
pub fn handle_publish<'a>(matches: &ArgMatches<'a>) {
    let file = matches.value_of(ARG_FILE).unwrap();
    let code =
        fs::read(PathBuf::from(file)).expect(format!("Unable to load file: {}", file).as_str());

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_data_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    let address = runtime.new_blueprint_address();
    if runtime.get_blueprint(address).is_none() {
        load_module(&code).unwrap();
        runtime.put_blueprint(address, Blueprint::new(code));
        runtime.flush();
    }
    println!("Blueprint: {}", address.to_string());
}
