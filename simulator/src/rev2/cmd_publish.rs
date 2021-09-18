use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::engine::*;
use radix_engine::execution::*;
use radix_engine::ledger::*;
use radix_engine::model::*;
use scrypto::args;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

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
            Arg::with_name(ARG_TRACE)
                .short("t")
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_PATH)
                .help("Specify the the path to a Scrypto package or a .wasm file.")
                .required(true),
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
    let trace = matches.is_present(ARG_TRACE);
    let path = PathBuf::from(
        matches
            .value_of(ARG_PATH)
            .ok_or_else(|| Error::MissingArgument(ARG_PATH.to_owned()))?,
    );
    let file = if path.extension() != Some(OsStr::new("wasm")) {
        build_package(path).map_err(Error::CargoError)?
    } else {
        path
    };
    let code = fs::read(&file).map_err(Error::IOError)?;
    validate_module(&code).map_err(Error::TxnExecutionError)?;

    if let Some(a) = matches.value_of(ARG_ADDRESS) {
        let address: Address = a.parse().map_err(Error::InvalidAddress)?;
        let mut ledger = FileBasedLedger::new(get_data_dir()?);
        ledger.put_package(address, Package::new(code));
        println!("New package: {}", address);
        return Ok(());
    }

    match get_config(CONF_DEFAULT_ACCOUNT)? {
        Some(a) => {
            let account: Address = a.as_str().parse().map_err(Error::InvalidAddress)?;

            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            let mut runtime = Runtime::new(sha256(Uuid::new_v4().to_string()), &mut ledger);
            let mut process = runtime.start_process(trace);
            let package: Address = process
                .call_method(account, "publish_package", args!(code))
                .and_then(decode_return)
                .map_err(Error::TxnExecutionError)?;
            process.finalize().map_err(Error::TxnExecutionError)?;
            runtime.flush();

            println!("New package: {}", package);
            Ok(())
        }
        None => Err(Error::NoDefaultAccount),
    }
}
