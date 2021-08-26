use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;

use crate::cli::*;
use crate::transaction::*;

const ARG_COMPONENT: &'static str = "COMPONENT";
const ARG_METHOD: &'static str = "METHOD";
const ARG_ARGS: &'static str = "ARGS";

/// Constructs a `call-method` subcommand.
pub fn make_call_method_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_CALL_METHOD)
        .about("Calls a component method")
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

/// Handles a `call-method` request.
pub fn handle_call_method<'a>(matches: &ArgMatches<'a>) {
    let component: Address = matches.value_of(ARG_COMPONENT).unwrap().into();
    let method = matches.value_of(ARG_METHOD).unwrap();
    let mut args = Vec::new();
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(a));
    }

    match construct_call_method_txn(component, method, &args, false) {
        Ok(txn) => {
            let receipt = execute(txn, true);
            print_receipt(receipt);
        }
        Err(e) => {
            println!("Failed to construct transaction: {:?}", e);
        }
    }
}
