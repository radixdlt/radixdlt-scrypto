use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;
use scrypto::rust::collections::HashMap;
use scrypto::types::*;

use crate::ledger::*;
use crate::rev2::*;

const ARG_TRACE: &str = "TRACE";
const ARG_MINTER_ADDRESS: &str = "MINTER_ADDRESS";
const ARG_SYMBOL: &str = "SYMBOL";
const ARG_NAME: &str = "NAME";
const ARG_DESCRIPTION: &str = "DESCRIPTION";
const ARG_URL: &str = "URL";
const ARG_ICON_URL: &str = "ICON_URL";

/// Constructs a `new-resource-mutable` subcommand.
pub fn make_new_resource_mutable<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_NEW_RESOURCE_MUTABLE)
        .about("Creates token with mutable supply")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_TRACE)
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_MINTER_ADDRESS)
                .help("Specify the minter address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_SYMBOL)
                .long("symbol")
                .takes_value(true)
                .help("Specify the symbol.")
                .required(false),
        )
        .arg(
            Arg::with_name(ARG_NAME)
                .long("name")
                .takes_value(true)
                .help("Specify the name.")
                .required(false),
        )
        .arg(
            Arg::with_name(ARG_DESCRIPTION)
                .long("description")
                .takes_value(true)
                .help("Specify the description.")
                .required(false),
        )
        .arg(
            Arg::with_name(ARG_URL)
                .long("url")
                .takes_value(true)
                .help("Specify the URL.")
                .required(false),
        )
        .arg(
            Arg::with_name(ARG_ICON_URL)
                .long("icon-url")
                .takes_value(true)
                .help("Specify the icon URL.")
                .required(false),
        )
}

/// Handles a `new-resource-mutable` request.
pub fn handle_new_resource_mutable(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);

    let minter_address: Address = matches
        .value_of(ARG_MINTER_ADDRESS)
        .ok_or_else(|| Error::MissingArgument(ARG_MINTER_ADDRESS.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;

    let mut metadata = HashMap::new();
    matches
        .value_of(ARG_SYMBOL)
        .and_then(|v| metadata.insert("symbol".to_owned(), v.to_owned()));
    matches
        .value_of(ARG_NAME)
        .and_then(|v| metadata.insert("name".to_owned(), v.to_owned()));
    matches
        .value_of(ARG_DESCRIPTION)
        .and_then(|v| metadata.insert("description".to_owned(), v.to_owned()));
    matches
        .value_of(ARG_URL)
        .and_then(|v| metadata.insert("url".to_owned(), v.to_owned()));
    matches
        .value_of(ARG_ICON_URL)
        .and_then(|v| metadata.insert("icon_url".to_owned(), v.to_owned()));

 
            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            let mut executor = TransactionExecutor::new(&mut ledger, 0, 0); // TODO: fix nonce and epoch.

            let resource_address = executor.new_resource_mutable(metadata, minter_address, trace);

            println!("New resource: {}", resource_address);
            Ok(()) 
}