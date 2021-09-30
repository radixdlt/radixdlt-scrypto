use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use colored::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::rev2::*;
use crate::utils::*;

const ARG_ADDRESS: &str = "ADDRESS";

/// Constructs a `show` subcommand.
pub fn make_show<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_SHOW)
        .about("Shows the content of an address")
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

    let ledger = FileBasedLedger::new(get_data_dir()?);
    match address {
        Address::System => {
            println!("Radix system address");
        }
        Address::PublicKey(_) => {
            println!("{}: {}", "Public key".green().bold(), address.to_string());
        }
        Address::Package(_) => {
            dump_package(address, &ledger);
        }
        Address::Component(_) => {
            dump_component(address, &ledger);
        }
        Address::ResourceDef(_) | Address::RadixToken => {
            dump_resource_def(address, &ledger);
        }
    }
    Ok(())
}
