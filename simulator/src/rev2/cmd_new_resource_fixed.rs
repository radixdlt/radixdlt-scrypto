use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::engine::*;
use radix_engine::utils::*;
use scrypto::args;
use scrypto::rust::collections::HashMap;
use scrypto::rust::str::FromStr;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::ledger::*;
use crate::rev2::*;

const ARG_TRACE: &str = "TRACE";
const ARG_SUPPLY: &str = "SUPPLY";
const ARG_SYMBOL: &str = "SYMBOL";
const ARG_NAME: &str = "NAME";
const ARG_DESCRIPTION: &str = "DESCRIPTION";
const ARG_URL: &str = "URL";
const ARG_ICON_URL: &str = "ICON_URL";

/// Constructs a `new-resource-fixed` subcommand.
pub fn make_new_resource_fixed<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_NEW_RESOURCE_FIXED)
        .about("Creates token with fixed supply")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_TRACE)
                .short("t")
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_SUPPLY)
                .help("Specify the total supply.")
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

/// Handles a `new-resource-fixed` request.
pub fn handle_new_resource_fixed(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);

    let supply = Amount::from_str(
        matches
            .value_of(ARG_SUPPLY)
            .ok_or_else(|| Error::MissingArgument(ARG_SUPPLY.to_owned()))?,
    )
    .map_err(|_| Error::InvalidAmount)?;

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

    match get_config(CONF_DEFAULT_ACCOUNT)? {
        Some(a) => {
            let account: Address = a.as_str().parse().map_err(Error::InvalidAddress)?;

            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            let mut track = Track::new(sha256(Uuid::new_v4().to_string()), &mut ledger);
            let mut process = track.start_process(trace);
            let resource_address: Address = process
                .call_method(account, "new_resource_fixed", args!(metadata, supply))
                .and_then(decode_return)
                .map_err(Error::TxnExecutionError)?;
            process.finalize().map_err(Error::TxnExecutionError)?;
            track.commit();

            println!("New resource: {}", resource_address);
            Ok(())
        }
        None => Err(Error::NoDefaultAccount),
    }
}
