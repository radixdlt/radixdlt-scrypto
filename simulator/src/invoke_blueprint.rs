use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use colored::*;
use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::kernel::*;
use scrypto::types::rust::collections::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::*;

const ARG_COMPONENT_NAME: &'static str = "COMPONENT_NAME";
const ARG_METHOD: &'static str = "METHOD";
const ARG_ADDRESS: &'static str = "ADDRESS";
const ARG_ARGS: &'static str = "ARGS";

/// Constructs a `invoke-blueprint` subcommand.
pub fn make_invoke_blueprint_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_INVOKE_BLUEPRINT)
        .about("Invokes a blueprint method.")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_ADDRESS)
                .help("Specify the blueprint address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_COMPONENT_NAME)
                .help("Specify the component name.")
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

/// Handles a `invoke-blueprint` request.
pub fn handle_invoke_blueprint<'a>(matches: &ArgMatches<'a>) {
    let component_name = matches.value_of(ARG_COMPONENT_NAME).unwrap();
    let method = matches.value_of(ARG_METHOD).unwrap();
    let blueprint: Address = matches.value_of(ARG_ADDRESS).unwrap().into();
    let mut args = Vec::new();
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(hex::decode(a).unwrap()));
    }

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_data_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);
    println!("----");
    println!("Blueprint: {}", blueprint);
    println!("Component: {}", component_name);
    println!("Method: {}", method);
    println!("Arguments: {:02x?}", args);
    println!("----");

    let (module, memory) = runtime.load_module(blueprint).expect("Blueprint not found");
    let mut process = Process::new(
        &mut runtime,
        blueprint,
        format!("{}_main", component_name),
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
