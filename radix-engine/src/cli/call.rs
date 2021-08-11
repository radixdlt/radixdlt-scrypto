use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::cli::*;
use crate::execution::*;
use crate::ledger::*;

const ARG_BLUEPRINT: &'static str = "BLUEPRINT";
const ARG_COMPONENT: &'static str = "COMPONENT";
const ARG_METHOD: &'static str = "METHOD";
const ARG_ARGS: &'static str = "ARGS";

/// Prepares a subcommand that handles `call`.
pub fn prepare_call<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("call")
        .about("Call into a blueprint.")
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

/// Processes a `call` command.
pub fn handle_call<'a>(matches: &ArgMatches<'a>) {
    let blueprint: Address = matches.value_of(ARG_BLUEPRINT).unwrap().into();
    let component = matches.value_of(ARG_COMPONENT).unwrap();
    let method = matches.value_of(ARG_METHOD).unwrap();
    let args = if let Some(x) = matches.values_of(ARG_ARGS) {
        x.map(|a| hex::decode(a).unwrap()).collect()
    } else {
        Vec::new()
    };
    println!("----");
    println!("Blueprint: {:?}", blueprint);
    println!("Component: {}", component);
    println!("Method: {}", method);
    println!("Arguments: {:?}", args);
    println!("----");

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_root_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);
    let (module, memory) = runtime.load_module(blueprint).expect("Blueprint not found");

    let mut process = Process::new(
        &mut runtime,
        blueprint,
        component.to_string(),
        method.to_string(),
        args,
        0,
        &module,
        &memory,
    );
    let output = process.run();
    if output.is_ok() {
        runtime.flush();
    }

    println!("----");
    println!("Output: {:?}", output);
    println!("----");
}
