use clap::{crate_version, App, Arg, ArgMatches};
use scrypto::types::*;

use crate::ledger::*;
use crate::resim::*;

const ARG_ADDRESS: &str = "ADDRESS";

/// Constructs a `show` subcommand.
pub fn make_show<'a>() -> App<'a> {
    App::new(CMD_SHOW)
        .about("Displays the content behind an address")
        .version(crate_version!())
        .arg(
            Arg::new(ARG_ADDRESS)
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

    let ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
    match address {
        Address::Package(_) => dump_package(address, &ledger).map_err(Error::LedgerDumpError),
        Address::Component(_) => dump_component(address, &ledger).map_err(Error::LedgerDumpError),
        Address::ResourceDef(_) => {
            dump_resource_def(address, &ledger).map_err(Error::LedgerDumpError)
        }
        Address::PublicKey(_) => Ok(println!("Public Key: {}", address)),
    }
}
