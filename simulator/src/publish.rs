use std::ffi::OsStr;
use std::fs;
use std::fs::read_dir;
use std::path::PathBuf;

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::utils::*;
use uuid::Uuid;

use crate::*;
use radix_engine::execution::*;
use radix_engine::ledger::*;
use radix_engine::model::*;

const ARG_PATH: &'static str = "PATH";

/// Constructs a `publish` subcommand.
pub fn make_publish_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_PUBLISH)
        .about("Publishes a package.")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_PATH)
                .help("Specify the the path to a Scrypto package or a .wasm file.")
                .required(true),
        )
}

/// Handles a `publish` request.
pub fn handle_publish<'a>(matches: &ArgMatches<'a>) {
    let mut path = PathBuf::from(matches.value_of(ARG_PATH).unwrap());
    let file = if path.extension() == Some(OsStr::new("wasm")) {
        path
    } else {
        path.push("target");
        path.push("wasm32-unknown-unknown");
        path.push("release");
        read_dir(path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension() == Some(OsStr::new("wasm")))
            .next()
            .unwrap()
    };
    println!("Publishing: {}", file.to_str().unwrap());
    let code = fs::read(&file).expect(format!("Unable to load file: {:?}", file).as_str());

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_data_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    let address = runtime.new_package_address();
    if runtime.get_package(address).is_none() {
        load_module(&code).unwrap();
        runtime.put_package(address, Package::new(code));
        runtime.flush();
    }
    println!("New package: {}", address.to_string());
}
