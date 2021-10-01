use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;

use crate::ledger::*;
use crate::rev2::*;

const ARG_PACKAGE: &str = "PACKAGE";
const ARG_NAME: &str = "NAME";

const ARG_TRACE: &str = "TRACE";

/// Constructs a `export-abi` subcommand.
pub fn make_export_abi<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_EXPORT_ABI)
        .about("Exports the ABI of a blueprint")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_PACKAGE)
                .help("Specify the blueprint package address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_NAME)
                .help("Specify the blueprint name.")
                .required(true),
        )
        // options
        .arg(
            Arg::with_name(ARG_TRACE)
                .long("trace")
                .help("Turns on tracing."),
        )
}

/// Handles a `export-abi` request.
pub fn handle_export_abi(matches: &ArgMatches) -> Result<(), Error> {
    let package = match_address(matches, ARG_PACKAGE)?;
    let name = match_string(matches, ARG_NAME)?;
    let trace = matches.is_present(ARG_TRACE);

    let configs = get_configs()?;
    let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    let executor = TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce);
    let abi = executor.export_abi(package, name, trace);

    match abi {
        Err(e) => Err(Error::TransactionExecutionError(e)),
        Ok(a) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&a).map_err(Error::JSONError)?
            );
            Ok(())
        }
    }
}
