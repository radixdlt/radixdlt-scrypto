use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;
use scrypto::rust::str::FromStr;
use scrypto::types::*;

use crate::ledger::*;
use crate::rev2::*;

const ARG_TRACE: &str = "TRACE";
const ARG_AMOUNT: &str = "AMOUNT";
const ARG_RESOURCE_ADDRESS: &str = "RESOURCE_ADDRESS";

/// Constructs a `mint` subcommand.
pub fn make_mint<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_MINT)
        .about("Mints resource")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_TRACE)
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_AMOUNT)
                .help("Specify the amount to mint.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_RESOURCE_ADDRESS)
                .help("Specify the resource definition address.")
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
    let resource_address: Address = matches
        .value_of(ARG_RESOURCE_ADDRESS)
        .ok_or_else(|| Error::MissingArgument(ARG_RESOURCE_ADDRESS.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;

    match get_config(CONF_DEFAULT_ACCOUNT)? {
        Some(a) => {
            let account: Address = a.as_str().parse().map_err(Error::InvalidAddress)?;

            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            let mut executor = TransactionExecutor::new(&mut ledger, 0, 0); // TODO: fix nonce and epoch.

            executor.mint_resource(amount, resource_address, account, trace);

            println!("Resource minted!");
            Ok(())
        }
        None => Err(Error::NoDefaultAccount),
    }
}
