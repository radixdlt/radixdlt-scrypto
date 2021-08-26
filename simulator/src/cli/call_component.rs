use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;

use crate::cli::*;
use crate::transaction::*;

const ARG_COMPONENT: &'static str = "COMPONENT";
const ARG_METHOD: &'static str = "METHOD";
const ARG_ARGS: &'static str = "ARGS";

/// Constructs a `call-component` subcommand.
pub fn make_call_component_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_CALL_COMPONENT)
        .about("Calls a component method.")
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

/// Handles a `call-component` request.
pub fn handle_call_component<'a>(matches: &ArgMatches<'a>) {
    let component: Address = matches.value_of(ARG_COMPONENT).unwrap().into();
    let method = matches.value_of(ARG_METHOD).unwrap().to_owned();
    let mut args = Vec::new();
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(hex::decode(a).unwrap()));
    }

    let transaction = Transaction {
        instructions: vec![Instruction::InvokeMethod {
            component,
            method,
            args,
        }],
    };

    let receipt = execute(transaction, true);
    print_receipt(receipt);
}
