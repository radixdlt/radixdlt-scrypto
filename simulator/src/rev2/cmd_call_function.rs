use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;
use scrypto::types::*;

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
            // TODO: fix nonce and epoch.
            let account: Address = a.as_str().parse().map_err(Error::InvalidAddress)?;

            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            let mut executor = TransactionExecutor::new(&mut ledger, 0, 0); // TODO: fix nonce and epoch.

            let abi = executor
                .export_abi(package_address, blueprint_name, trace)
                .map_err(Error::TxnExecutionError)?;

            let transaction = TransactionBuilder::new()
                .call_function(&abi, function, args)
                .build_with(Some(account))
                .map_err(Error::TxnConstructionErr)?;

            let receipt = executor.execute(&transaction, trace);
            dump_receipt(&receipt);

            if receipt.success {
                Ok(())
            } else {
                Err(Error::TransactionFailed)
            }
        }
        None => Err(Error::NoDefaultAccount),
    }
}
