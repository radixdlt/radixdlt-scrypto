use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::types::rust::collections::*;
use scrypto::types::*;
use scrypto::utils::*;

use crate::*;

const ARG_PACKAGE: &'static str = "PACKAGE";
const ARG_BLUEPRINT: &'static str = "BLUEPRINT";

/// Constructs a `export-abi` subcommand.
pub fn make_export_abi_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_EXPORT_ABI)
        .about("Exports the ABI of a blueprint.")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_PACKAGE)
                .help("Specify the package address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_BLUEPRINT)
                .help("Specify the blueprint name.")
                .required(true),
        )
}

/// Handles a `export-abi` request.
pub fn handle_export_abi<'a>(matches: &ArgMatches<'a>) {
    let package: Address = matches.value_of(ARG_PACKAGE).unwrap().into();
    let blueprint = matches.value_of(ARG_BLUEPRINT).unwrap();
    println!("----");
    println!("Package: {}", package.to_string());
    println!("Blueprint: {}", blueprint);
    println!("----");

    let tx_hash = sha256("");
    let mut ledger = InMemoryLedger::new();
    ledger.put_package(
        package,
        FileBasedLedger::new(get_data_dir())
            .get_package(package)
            .expect("Package not found"),
    );
    let mut runtime = Runtime::new(tx_hash, &mut ledger);
    let (module, memory) = runtime.load_module(package).unwrap();

    let mut process = Process::new(
        &mut runtime,
        package,
        format!("{}_abi", blueprint),
        String::new(),
        vec![],
        0,
        &module,
        &memory,
        HashMap::new(),
        HashMap::new(),
    );
    let output = process.run();
    if output.is_ok() {
        runtime.flush();
    }

    println!("----");
    match output {
        Ok(bytes) => {
            let abi: abi::Blueprint = scrypto_decode(&bytes).unwrap();
            let json = serde_json::to_string_pretty(&abi).unwrap();
            println!("{}", json);
        }
        Err(error) => {
            println!("Failed to export ABI: {}", error);
        }
    }
    println!("----");
}
