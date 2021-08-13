use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use sbor::collections::*;
use scrypto::buffer::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::cli::*;
use crate::execution::*;
use crate::ledger::*;

const ARG_ADDRESS: &'static str = "ADDRESS";
const ARG_METHOD: &'static str = "METHOD";
const ARG_ARGS: &'static str = "ARGS";

/// Constructs a `invoke_component` subcommand.
pub fn make_invoke_component_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_INVOKE_COMPONENT)
        .about("Invokes a component.")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_ADDRESS)
                .help("Specify the component address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_METHOD)
                .help("Specify the component method.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_ARGS)
                .help("Specify the arguments.")
                .multiple(true),
        )
}

/// Handles a `invoke_component` request.
pub fn handle_invoke_component<'a>(matches: &ArgMatches<'a>) {
    let address: Address = matches.value_of(ARG_ADDRESS).unwrap().into();
    let method = matches.value_of(ARG_METHOD).unwrap();
    let mut args = if let Some(x) = matches.values_of(ARG_ARGS) {
        x.map(|a| hex::decode(a).unwrap()).collect()
    } else {
        Vec::new()
    };
    println!("----");
    println!("Component: {}", address.to_string());
    println!("Method: {}", method);
    println!("Arguments: {:02x?}", args);
    println!("----");

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_root_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);
    let component = runtime.get_component(address).expect("Component not found");
    let blueprint = component.blueprint();
    let name = component.name().to_owned();
    let (module, memory) = runtime.load_module(blueprint).expect("Blueprint not found");

    // use component address as the first argument
    args.insert(0, scrypto_encode(&address));

    let mut process = Process::new(
        &mut runtime,
        blueprint,
        format!("{}_main", name),
        method.to_owned(),
        args,
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
    println!("Output: {:02x?}", output);
    println!("----");
}
