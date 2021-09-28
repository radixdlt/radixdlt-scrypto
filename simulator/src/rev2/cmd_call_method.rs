use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::ledger::*;
use crate::rev2::*;
use crate::utils::*;

const ARG_TRACE: &str = "TRACE";
const ARG_COMPONENT_ADDRESS: &str = "COMPONENT_ADDRESS";
const ARG_METHOD: &str = "METHOD";
const ARG_ARGS: &str = "ARGS";

/// Constructs a `call-method` subcommand.
pub fn make_call_method<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_CALL_METHOD)
        .about("Calls a component method")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_TRACE)
                .short("t")
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_COMPONENT_ADDRESS)
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
            .help("Specify the arguments, e.g. \"5\", \"hello\" or \"amount,resource_address\" (bucket).")
                .multiple(true),
        )
}

/// Handles a `call-method` request.
pub fn handle_call_method(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);
    let component_address: Address = matches
        .value_of(ARG_COMPONENT_ADDRESS)
        .ok_or_else(|| Error::MissingArgument(ARG_COMPONENT_ADDRESS.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;
    let method = matches
        .value_of(ARG_METHOD)
        .ok_or_else(|| Error::MissingArgument(ARG_METHOD.to_owned()))?;
    let mut args = Vec::new();
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(a));
    }

    match get_config(CONF_DEFAULT_ACCOUNT)? {
        Some(a) => {
            let account: Address = a.as_str().parse().map_err(Error::InvalidAddress)?;
            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            match build_call_method(
                &mut ledger,
                account,
                component_address,
                method,
                &args,
                trace,
            ) {
                Ok(txn) => {
                    let receipt = execute_transaction(
                        &mut ledger,
                        sha256(Uuid::new_v4().to_string()),
                        txn,
                        trace,
                    );
                    dump_receipt(&receipt);
                    if receipt.success {
                        Ok(())
                    } else {
                        Err(Error::TransactionFailed)
                    }
                }
                Err(e) => Err(Error::TxnConstructionErr(e)),
            }
        }
        None => Err(Error::NoDefaultAccount),
    }
}
