use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use colored::*;
use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::buffer::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::cli::*;
use crate::invoke::*;
use crate::ledger::*;

const ARG_COMPONENT: &'static str = "COMPONENT";
const ARG_METHOD: &'static str = "METHOD";
const ARG_ARGS: &'static str = "ARGS";

/// Constructs a `invoke-component` subcommand.
pub fn make_invoke_component_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_INVOKE_COMPONENT)
        .about("Invokes a component method.")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_COMPONENT)
                .help("Specify the component address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_METHOD)
                .help("Specify the method name.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_ARGS)
                .help("Specify the arguments, in hex.")
                .multiple(true),
        )
}

/// Handles a `invoke-component` request.
pub fn handle_invoke_component<'a>(matches: &ArgMatches<'a>) {
    let address: Address = matches.value_of(ARG_COMPONENT).unwrap().into();
    let method = matches.value_of(ARG_METHOD).unwrap();
    let package;
    let blueprint;
    let mut args = Vec::new();

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_data_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);
    let component = runtime.get_component(address).expect("Component not found");
    package = component.package();
    blueprint = component.name().to_owned();
    args.push(scrypto_encode(&address)); // self
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(hex::decode(a).unwrap()));
    }

    let (output, logs) = invoke(package, blueprint.as_str(), method, args, true);

    println!("----");
    println!("Number of Logs: {}", logs.len());
    for (level, msg) in logs {
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
