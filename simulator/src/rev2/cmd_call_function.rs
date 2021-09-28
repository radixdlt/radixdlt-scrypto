use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::ledger::*;
use crate::rev2::*;
use crate::utils::*;

const ARG_TRACE: &str = "TRACE";
const ARG_PACKAGE_ADDRESS: &str = "PACKAGE_ADDRESS";
const ARG_BLUEPRINT_NAME: &str = "BLUEPRINT_NAME";
const ARG_FUNCTION: &str = "FUNCTION";
const ARG_ARGS: &str = "ARGS";

/// Constructs a `call-function` subcommand.
pub fn make_call_function<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_CALL_FUNCTION)
        .about("Calls a blueprint function")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_TRACE)
                .short("t")
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_PACKAGE_ADDRESS)
                .help("Specify the package address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_BLUEPRINT_NAME)
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
                .help("Specify the arguments, e.g. \"5\", \"hello\" or \"amount,resource_address\" (bucket).")
                .multiple(true),
        )
}

/// Handles a `call-function` request.
pub fn handle_call_function(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);
    let package_address: Address = matches
        .value_of(ARG_PACKAGE_ADDRESS)
        .ok_or_else(|| Error::MissingArgument(ARG_PACKAGE_ADDRESS.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;
    let blueprint_name = matches
        .value_of(ARG_BLUEPRINT_NAME)
        .ok_or_else(|| Error::MissingArgument(ARG_BLUEPRINT_NAME.to_owned()))?;
    let function = matches
        .value_of(ARG_FUNCTION)
        .ok_or_else(|| Error::MissingArgument(ARG_FUNCTION.to_owned()))?;
    let mut args = Vec::new();
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(a));
    }

    match get_config(CONF_DEFAULT_ACCOUNT)? {
        Some(a) => {
            let account: Address = a.as_str().parse().map_err(Error::InvalidAddress)?;
            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            match build_call_function(
                &mut ledger,
                account,
                (package_address, blueprint_name.to_owned()),
                function,
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
