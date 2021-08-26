use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;

use crate::cli::*;
use crate::transaction::*;

const ARG_PACKAGE: &'static str = "PACKAGE";
const ARG_BLUEPRINT: &'static str = "BLUEPRINT";
const ARG_FUNCTION: &'static str = "FUNCTION";
const ARG_ARGS: &'static str = "ARGS";

/// Constructs a `call-function` subcommand.
pub fn make_call_function_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_CALL_FUNCTION)
        .about("Calls a blueprint function")
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

/// Handles a `call-function` request.
pub fn handle_call_function<'a>(matches: &ArgMatches<'a>) {
    let package: Address = matches.value_of(ARG_PACKAGE).unwrap().into();
    let blueprint = matches.value_of(ARG_BLUEPRINT).unwrap();
    let function = matches.value_of(ARG_FUNCTION).unwrap();
    let mut args = Vec::new();
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(a));
    }

    match construct_call_function_txn(package, blueprint, function, &args, false) {
        Ok(txn) => {
            let receipt = execute(txn, true);
            print_receipt(receipt);
        }
        Err(e) => {
            println!("Failed to construct transaction: {:?}", e);
        }
    }
}
