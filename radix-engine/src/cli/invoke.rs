use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use sbor::collections::*;
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

/// Constructs a `invoke` subcommand.
pub fn make_invoke_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_INVOKE)
        .about("Invokes a blueprint.")
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

/// Handles a `invoke` request.
pub fn handle_invoke<'a>(matches: &ArgMatches<'a>) {
    let blueprint: Address = matches.value_of(ARG_BLUEPRINT).unwrap().into();
    let component = matches.value_of(ARG_COMPONENT).unwrap();
    let method = matches.value_of(ARG_METHOD).unwrap();
    let args = if let Some(x) = matches.values_of(ARG_ARGS) {
        x.map(|a| hex::decode(a).unwrap()).collect()
    } else {
        Vec::new()
    };
    println!("----");
    println!("Blueprint: {}", blueprint.to_string());
    println!("Component: {}", component);
    println!("Method: {}", method);
    println!("Arguments: {:02x?}", args);
    println!("----");

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_root_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);
    let (module, memory) = runtime.load_module(blueprint).expect("Blueprint not found");

    let mut process = Process::new(
        &mut runtime,
        blueprint,
        format!("{}_main", component),
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
