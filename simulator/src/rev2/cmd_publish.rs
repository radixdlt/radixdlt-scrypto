use std::ffi::OsStr;
use std::fs;

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::ledger::*;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::rev2::*;
use crate::utils::*;

const ARG_TRACE: &str = "TRACE";
const ARG_PATH: &str = "PATH";
const ARG_ADDRESS: &str = "ADDRESS";

/// Constructs a `publish` subcommand.
pub fn make_publish<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_PUBLISH)
        .about("Publishes a package")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_PATH)
                .help("Specify the the path to a Scrypto package or a .wasm file.")
                .required(true),
        )
        // options
        .arg(
            Arg::with_name(ARG_TRACE)
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_ADDRESS)
                .long("address")
                .takes_value(true)
                .help("Specify the address to overwrite.")
                .required(false),
        )
}

/// Handles a `publish` request.
pub fn handle_publish(matches: &ArgMatches) -> Result<(), Error> {
    let path = match_path(matches, ARG_PATH)?;
    let trace = matches.is_present(ARG_TRACE);

    // Load wasm code
    let code = fs::read(if path.extension() != Some(OsStr::new("wasm")) {
        build_package(path, false).map_err(Error::CargoError)?
    } else {
        path
    })
    .map_err(Error::IOError)?;

    // Update existing package if `--address` is provided
    if let Some(a) = matches.value_of(ARG_ADDRESS) {
        let address: Address = a.parse().map_err(Error::InvalidAddress)?;
        let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
        ledger.put_package(address, Package::new(code));
        println!("Package updated!");
        Ok(())
    } else {
        let mut configs = get_configs()?;
        let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
        let mut executor =
            TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce);
        let transaction = TransactionBuilder::new(&executor)
            .publish_package(&code)
            .build()
            .map_err(Error::TransactionConstructionError)?;

        let receipt = executor.run(transaction, trace);

        println!("{:?}", receipt);
        if receipt.success {
            configs.nonce = executor.nonce();
            set_configs(configs)?;
            Ok(())
        } else {
            Err(Error::TransactionFailed)
        }
    }
}
