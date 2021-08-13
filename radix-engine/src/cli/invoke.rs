use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use colored::*;
use sbor::collections::*;
use scrypto::buffer::*;
use scrypto::kernel::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::cli::*;
use crate::execution::*;
use crate::ledger::*;

const ARG_COMPONENT_NAME: &'static str = "COMPONENT_NAME";
const ARG_METHOD_NAME: &'static str = "METHOD_NAME";
const ARG_ADDRESS: &'static str = "ADDRESS";
const ARG_ARGS: &'static str = "ARGS";

/// Constructs a `invoke` subcommand.
pub fn make_invoke_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_INVOKE)
        .about("Invokes a method.")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_COMPONENT_NAME)
                .help("Specify the component name.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_METHOD_NAME)
                .help("Specify the method name.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_ADDRESS)
                .help(
                    "Specify the blueprint address if functional, otherwise the component address.",
                )
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_ARGS)
                .help("Specify the arguments, in hex.")
                .multiple(true),
        )
}

/// Handles a `invoke` request.
pub fn handle_invoke<'a>(matches: &ArgMatches<'a>) {
    let component_name = matches.value_of(ARG_COMPONENT_NAME).unwrap();
    let method_name = matches.value_of(ARG_METHOD_NAME).unwrap();
    let address: Address = matches.value_of(ARG_ADDRESS).unwrap().into();
    let blueprint;
    let mut args = Vec::new();

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_root_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    // Check whether it's functional or stateful.
    match address {
        Address::Component(_) => {
            let component = runtime.get_component(address).expect("Component not found");
            blueprint = component.blueprint();
            args.push(scrypto_encode(&address)); // self
        }
        Address::Blueprint(_) => {
            blueprint = address;
        }
        _ => {
            panic!("Invalid address: {}", address);
        }
    }
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(hex::decode(a).unwrap()));
    }

    println!("----");
    println!("Blueprint: {}", blueprint);
    println!("Component: {}", component_name);
    println!("Method: {}", method_name);
    println!("Arguments: {:02x?}", args);
    println!("----");

    let (module, memory) = runtime.load_module(blueprint).expect("Blueprint not found");
    let mut process = Process::new(
        &mut runtime,
        blueprint,
        format!("{}_main", component_name),
        method_name.to_owned(),
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
    println!("Number of Logs: {}", runtime.logs().len());
    for (level, msg) in runtime.logs() {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), msg.red()),
            Level::Warn => ("WARN".yellow(), msg.yellow()),
            Level::Info => ("INFO".green(), msg.green()),
            Level::Debug => ("DEBUG".cyan(), msg.cyan()),
            Level::Trace => ("TRACE".normal(), msg.normal()),
        };

        println!("[{:5}] {}", l, m);
    }
    println!("Output: {:02x?}", output);
    println!("----");
}
