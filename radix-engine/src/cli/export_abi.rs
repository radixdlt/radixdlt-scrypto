use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use sbor::collections::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::types::*;
use scrypto::utils::*;

use crate::cli::*;
use crate::execution::*;
use crate::ledger::*;

const ARG_BLUEPRINT: &'static str = "BLUEPRINT";
const ARG_COMPONENT: &'static str = "COMPONENT";

/// Constructs a `export_abi` subcommand.
pub fn make_export_abi_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_EXPORT_ABI)
        .about("Exports the ABI of a component.")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_BLUEPRINT)
                .help("Specify the blueprint address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_COMPONENT)
                .help("Specify the component name.")
                .required(true),
        )
}

/// Handles a `export_abi` request.
pub fn handle_export_abi<'a>(matches: &ArgMatches<'a>) {
    let blueprint: Address = matches.value_of(ARG_BLUEPRINT).unwrap().into();
    let component = matches.value_of(ARG_COMPONENT).unwrap();
    println!("----");
    println!("Blueprint: {}", blueprint.to_string());
    println!("Component: {}", component);
    println!("----");

    let tx_hash = sha256("");
    let mut ledger = InMemoryLedger::new();
    ledger.put_blueprint(
        blueprint,
        FileBasedLedger::new(get_root_dir())
            .get_blueprint(blueprint)
            .expect("Blueprint not found"),
    );
    let mut runtime = Runtime::new(tx_hash, &mut ledger);
    let (module, memory) = runtime.load_module(blueprint).expect("Blueprint not found");

    let mut process = Process::new(
        &mut runtime,
        blueprint,
        format!("{}_abi", component),
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
            let abi: abi::Component = scrypto_decode(&bytes).unwrap();
            let json = serde_json::to_string_pretty(&abi).unwrap();
            println!("{}", json);
        }
        Err(error) => {
            println!("Failed to export ABI: {}", error);
        }
    }
    println!("----");
}
