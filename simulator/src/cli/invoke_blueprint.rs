use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;

use crate::cli::*;
use crate::invoke::*;

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

    let result = invoke_blueprint(package, blueprint, function, &args, false);

    match result {
        Err(e) => {
            println!("Error: {}", e);
        }
        Ok(r) => {
            println!("Result: {:02x?}", r.result);
            print_logs(&r.logs);
        }
    }
}
