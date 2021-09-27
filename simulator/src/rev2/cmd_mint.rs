use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::execution::*;
use scrypto::args;
use scrypto::rust::str::FromStr;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::ledger::*;
use crate::rev2::*;

const ARG_TRACE: &str = "TRACE";
const ARG_AMOUNT: &str = "AMOUNT";
const ARG_RESOURCE: &str = "RESOURCE";

/// Constructs a `mint` subcommand.
pub fn make_mint<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_MINT)
        .about("Mints resources")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_TRACE)
                .short("t")
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_AMOUNT)
                .help("Specify the amount to mint.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_RESOURCE)
                .help("Specify the resource address.")
                .required(true),
        )
}

/// Handles a `mint` request.
pub fn handle_mint(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);
    let amount = Amount::from_str(
        matches
            .value_of(ARG_AMOUNT)
            .ok_or_else(|| Error::MissingArgument(ARG_AMOUNT.to_owned()))?,
    )
    .map_err(|_| Error::InvalidAmount)?;
    let resource: Address = matches
        .value_of(ARG_RESOURCE)
        .ok_or_else(|| Error::MissingArgument(ARG_RESOURCE.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;

    match get_config(CONF_DEFAULT_ACCOUNT)? {
        Some(a) => {
            let account: Address = a.as_str().parse().map_err(Error::InvalidAddress)?;

            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            let mut track = Track::new(sha256(Uuid::new_v4().to_string()), &mut ledger);
            let mut process = track.start_process(trace);
            process
                .call_method(account, "mint", args!(amount, resource))
                .and_then(|_| process.finalize())
                .map_err(Error::TxnExecutionError)?;
            track.commit();

            println!("Resource minted into the default account!");
            Ok(())
        }
        None => Err(Error::NoDefaultAccount),
    }
}
