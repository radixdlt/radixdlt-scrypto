use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;

use crate::ledger::*;
use crate::rev2::*;

const ARG_ADDRESS: &str = "ADDRESS";

/// Constructs a `show` subcommand.
pub fn make_show<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_SHOW)
        .about("Displays the content behind an address")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_ADDRESS)
                .help("Specify the address.")
                .required(true),
        )
}

/// Handles a `show` request.
pub fn handle_show(matches: &ArgMatches) -> Result<(), Error> {
    let address: Address = matches
        .value_of(ARG_ADDRESS)
        .ok_or_else(|| Error::MissingArgument(ARG_ADDRESS.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;

    let ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    match address {
        Address::Package(_) => dump_package(address, &ledger).map_err(Error::LedgerDumpError),
        Address::Component(_) => dump_component(address, &ledger).map_err(Error::LedgerDumpError),
        Address::ResourceDef(_) | Address::RadixToken => {
            dump_resource_def(address, &ledger).map_err(Error::LedgerDumpError)
        }
    }
}
