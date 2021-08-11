use std::fs;
use std::path::PathBuf;

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::utils::*;
use uuid::Uuid;

use crate::cli::get_root_dir;
use crate::execution::*;
use crate::ledger::*;
use crate::model::*;

const ARG_FILE: &'static str = "FILE";

/// Prepares a subcommand that handles `publish`.
pub fn prepare_publish<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("publish")
        .about("Publish a new blueprint.")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_FILE)
                .help("Specify the .wasm file to publish.")
                .required(true),
        )
}

/// Processes a `publish` command.
pub fn handle_publish<'a>(args: &ArgMatches<'a>) {
    let file = args.value_of(ARG_FILE).unwrap();
    let code =
        fs::read(PathBuf::from(file)).expect(format!("Unable to load file: {}", file).as_str());

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_root_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    let address = runtime.new_blueprint_address(&code);
    if runtime.get_blueprint(address).is_some() {
        println!("{}", address.to_string());
    } else {
        load_module(&code).unwrap();
        runtime.put_blueprint(address, Blueprint::new(code));
        runtime.flush();
        println!("{}", address.to_string());
    }
}
