use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::ledger::*;
use scrypto::types::*;

use crate::cli::*;

const ARG_ADDRESS: &'static str = "ADDRESS";

/// Constructs a `show` subcommand.
pub fn make_show_cmd<'a, 'b>() -> App<'a, 'b> {
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
pub fn handle_show<'a>(matches: &ArgMatches<'a>) -> Result<(), Error> {
    let address: Address = matches
        .value_of(ARG_ADDRESS)
        .ok_or(Error::MissingArgument(ARG_ADDRESS.to_owned()))?
        .into();

    let ledger = FileBasedLedger::new(get_data_dir()?);
    match address {
        Address::System => {
            println!("Radix system address");
        }
        Address::Resource(_) | Address::RadixToken => {
            let resource = ledger.get_resource(address);
            match resource {
                Some(r) => {
                    println!("Resource: {}", address.to_string());
                    println!("Symbol: {}", r.symbol);
                    println!("Name: {}", r.name);
                    println!("Description: {}", r.description);
                    println!("URL: {}", r.url);
                    println!("Icon URL: {}", r.icon_url);
                    println!("Minter: {:?}", r.minter);
                    println!("supply: {:?}", r.supply);
                }
                None => {
                    println!("Resource not found");
                }
            }
        }
        Address::PublicKey(_) => {
            println!("Public key: {}", address.to_string());
        }
        Address::Package(_) => {
            let package = ledger.get_package(address);
            match package {
                Some(b) => {
                    println!("Package: {}", address.to_string());
                    println!("Code size: {} bytes", b.code().len());
                }
                None => {
                    println!("Package not found");
                }
            }
        }
        Address::Component(_) => {
            let component = ledger.get_component(address);
            match component {
                Some(c) => {
                    println!("Component: {}", address.to_string());
                    println!("State: {:02x?}", c.state());
                }
                None => {
                    println!("Component not found");
                }
            }
        }
    }
    Ok(())
}
