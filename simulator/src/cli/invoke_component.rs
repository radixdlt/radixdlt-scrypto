use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;

use crate::cli::*;
use crate::transaction::*;

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
    let component: Address = matches.value_of(ARG_COMPONENT).unwrap().into();
    let method = matches.value_of(ARG_METHOD).unwrap().to_owned();
    let mut args = Vec::new();
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(hex::decode(a).unwrap()));
    }

    let transaction = Transaction {
        actions: vec![Action::InvokeComponent {
            component,
            method,
            args,
        }],
    };

    let result = execute(transaction, true);

    match result {
        Err(e) => {
            println!("Error: {:?}", e);
        }
        Ok(r) => {
            print_receipt(r);
        }
    }
}
