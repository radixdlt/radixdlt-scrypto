use std::ffi::OsStr;
use std::fs;

use clap::{crate_version, App, Arg, ArgMatches};
use radix_engine::ledger::*;
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::resim::*;
use crate::utils::*;

const ARG_TRACE: &str = "TRACE";
const ARG_SIGNERS: &str = "SIGNERS";
const ARG_PATH: &str = "PATH";
const ARG_ADDRESS: &str = "ADDRESS";

/// Constructs a `publish` subcommand.
pub fn make_publish<'a>() -> App<'a> {
    App::new(CMD_PUBLISH)
        .about("Publishes a package")
        .version(crate_version!())
        .arg(
            Arg::new(ARG_PATH)
                .help("Specify the the path to a Scrypto package or a .wasm file.")
                .required(true),
        )
        // options
        .arg(Arg::new(ARG_TRACE).long("trace").help("Turn on tracing."))
        .arg(
            Arg::new(ARG_SIGNERS)
                .long("signers")
                .takes_value(true)
                .help("Specify the transaction signers, separated by comma."),
        )
        .arg(
            Arg::new(ARG_ADDRESS)
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
    let signers = match_signers(matches, ARG_SIGNERS)?;

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
            TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce, trace);
        let transaction = TransactionBuilder::new(&executor)
            .publish_package(&code)
            .build(signers)
            .map_err(Error::TransactionConstructionError)?;

        let receipt = executor.run(transaction).unwrap();

        println!("{:?}", receipt);
        if receipt.error.is_none() {
            configs.nonce = executor.nonce();
            set_configs(configs)?;
            Ok(())
        } else {
            Err(Error::TransactionFailed)
        }
    }
}
