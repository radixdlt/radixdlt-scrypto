use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::execution::*;
use radix_engine::ledger::*;
use radix_engine::model::*;
use scrypto::buffer::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::cli::*;
use crate::ledger::*;
use crate::utils::*;

const ARG_TRACE: &'static str = "TRACE";
const ARG_PATH: &'static str = "PATH";
const ARG_ADDRESS: &'static str = "ADDRESS";

/// Constructs a `publish` subcommand.
pub fn make_publish_cmd<'a, 'b>() -> App<'a, 'b> {
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
pub fn handle_publish<'a>(matches: &ArgMatches<'a>) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);
    let path = PathBuf::from(
        matches
            .value_of(ARG_PATH)
            .ok_or(Error::MissingArgument(ARG_PATH.to_owned()))?,
    );
    let file = if path.extension() != Some(OsStr::new("wasm")) {
        build_package(path).map_err(|e| Error::BuildError(e))?
    } else {
        path
    };
    let code = fs::read(&file).map_err(|e| Error::IOError(e))?;
    validate_module(&code).map_err(|e| Error::ExecutionError(e))?;

    if let Some(a) = matches.value_of(ARG_ADDRESS) {
        let address: Address = a.parse().map_err(|e| Error::InvalidAddress(e))?;
        let mut ledger = FileBasedLedger::new(get_data_dir()?);
        ledger.put_package(address, Package::new(code));
        println!("New package: {}", address);
        return Ok(());
    }

    match get_config(CONF_DEFAULT_ACCOUNT)? {
        Some(a) => {
            let account: Address = a.as_str().parse().map_err(|e| Error::InvalidAddress(e))?;
            let tx_hash = sha256(Uuid::new_v4().to_string());
            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            let mut runtime = Runtime::new(tx_hash, &mut ledger);

            let mut process = Process::new(0, trace, &mut runtime);
            let output = process
                .target_method(
                    account,
                    "publish_package".to_owned(),
                    vec![scrypto_encode(&code)],
                )
                .and_then(|target| process.run(target))
                .map_err(|e| Error::ExecutionError(e))?;
            process.finalize().map_err(|e| Error::ExecutionError(e))?;
            let package: Address = scrypto_decode(&output).map_err(|e| Error::DataError(e))?;

            runtime.flush();
            println!("New package: {}", package);

            Ok(())
        }
        None => Err(Error::NoDefaultAccount),
    }
}
