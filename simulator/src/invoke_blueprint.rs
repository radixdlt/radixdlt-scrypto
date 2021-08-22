use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use colored::*;
use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::types::rust::collections::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::*;

const ARG_PACKAGE: &'static str = "PACKAGE";
const ARG_BLUEPRINT: &'static str = "BLUEPRINT";
const ARG_FUNCTION: &'static str = "FUNCTION";
const ARG_ARGS: &'static str = "ARGS";

/// Constructs a `invoke-blueprint` subcommand.
pub fn make_invoke_blueprint_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_INVOKE_BLUEPRINT)
        .about("Invokes a blueprint function.")
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
        .arg(
            Arg::with_name(ARG_FUNCTION)
                .help("Specify the function name.")
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
    let package: Address = matches.value_of(ARG_PACKAGE).unwrap().into();
    let blueprint = matches.value_of(ARG_BLUEPRINT).unwrap();
    let function = matches.value_of(ARG_FUNCTION).unwrap();
    let mut args = Vec::new();
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(hex::decode(a).unwrap()));
    }

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_data_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);
    println!("----");
    println!("Package: {}", package);
    println!("Blueprint: {}", blueprint);
    println!("Function: {}", function);
    println!("Arguments: {:02x?}", args);
    println!("----");

    let (module, memory) = runtime.load_module(package).expect("Package not found");
    let mut process = Process::new(
        &mut runtime,
        package,
        format!("{}_main", blueprint),
        function.to_owned(),
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
