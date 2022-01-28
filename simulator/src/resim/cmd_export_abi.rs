use clap::{crate_version, App, Arg, ArgMatches};
use radix_engine::transaction::*;

use crate::ledger::*;
use crate::resim::*;

const ARG_PACKAGE: &str = "PACKAGE_ADDRESS";
const ARG_NAME: &str = "BLUEPRINT_NAME";

const ARG_TRACE: &str = "TRACE";

/// Constructs a `export-abi` subcommand.
pub fn make_export_abi<'a>() -> App<'a> {
    App::new(CMD_EXPORT_ABI)
        .about("Exports the ABI of a blueprint")
        .version(crate_version!())
        .arg(
            Arg::new(ARG_PACKAGE)
                .help("Specify the blueprint package address.")
                .required(true),
        )
        .arg(
            Arg::new(ARG_NAME)
                .help("Specify the blueprint name.")
                .required(true),
        )
        // options
        .arg(Arg::new(ARG_TRACE).long("trace").help("Turn on tracing."))
}

/// Handles a `export-abi` request.
pub fn handle_export_abi(matches: &ArgMatches) -> Result<(), Error> {
    let package = match_address(matches, ARG_PACKAGE)?;
    let name = match_string(matches, ARG_NAME)?;
    let trace = matches.is_present(ARG_TRACE);

    let configs = get_configs()?;
    let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    let executor =
        TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce, trace);
    let abi = executor.export_abi(package, name);

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
