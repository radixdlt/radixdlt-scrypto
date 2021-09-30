use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::rev2::*;

const ARG_TRACE: &str = "TRACE";
const ARG_PACKAGE_ADDRESS: &str = "PACKAGE_ADDRESS";
const ARG_BLUEPRINT_NAME: &str = "BLUEPRINT_NAME";

/// Constructs a `export-abi` subcommand.
pub fn make_export_abi<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_EXPORT_ABI)
        .about("Exports the ABI of a blueprint")
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
}

/// Handles a `export-abi` request.
pub fn handle_export_abi(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);
    let package_address: Address = matches
        .value_of(ARG_PACKAGE_ADDRESS)
        .ok_or_else(|| Error::MissingArgument(ARG_PACKAGE_ADDRESS.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;
    let blueprint_name = matches
        .value_of(ARG_BLUEPRINT_NAME)
        .ok_or_else(|| Error::MissingArgument(ARG_BLUEPRINT_NAME.to_owned()))?;

    let mut ledger = FileBasedLedger::new(get_data_dir()?);
    let   executor = TransactionExecutor::new(&mut ledger, 0, 0);

    let result = executor.export_abi(package_address, blueprint_name, trace);

    match result {
        Err(e) => Err(Error::TxnExecutionError(e)),
        Ok(abi) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&abi).map_err(Error::JSONError)?
            );
            Ok(())
        }
    }
}
