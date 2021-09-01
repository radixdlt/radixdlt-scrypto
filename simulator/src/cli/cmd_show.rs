use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use colored::*;
use radix_engine::ledger::*;
use scrypto::types::*;

use crate::cli::*;
use crate::utils::*;

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
                    println!("{}: {}", "Resource".green().bold(), address.to_string());
                    println!("{}: {}", "Symbol".green().bold(), r.symbol);
                    println!("{}: {}", "Name".green().bold(), r.name);
                    println!("{}: {}", "Description".green().bold(), r.description);
                    println!("{}: {}", "URL".green().bold(), r.url);
                    println!("{}: {}", "Icon URL".green().bold(), r.icon_url);
                    println!("{}: {:?}", "Minter".green().bold(), r.minter);
                    println!("{}: {:?}", "supply".green().bold(), r.supply);
                }
                None => {
                    println!("{}", "Resource not found".red());
                }
            }
        }
        Address::PublicKey(_) => {
            println!("{}: {}", "Public key".green().bold(), address.to_string());
        }
        Address::Package(_) => {
            let package = ledger.get_package(address);
            match package {
                Some(b) => {
                    println!("{}: {}", "Package".green().bold(), address.to_string());
                    println!("{}: {} bytes", "Code size".green().bold(), b.code().len());
                }
                None => {
                    println!("{}", "Package not found".red());
                }
            }
        }
        Address::Component(_) => {
            let component = ledger.get_component(address);
            match component {
                Some(c) => {
                    println!("{}: {}", "Component".green().bold(), address.to_string());
                    println!(
                        "{}: {}, {}",
                        "Blueprint".green().bold(),
                        c.package(),
                        c.blueprint()
                    );
                    println!("{}: {:02x?}", "State".green().bold(), c.state());
                    let (fmt, bids) = format_sbor(c.state()).map_err(|e| Error::DataError(e))?;
                    println!("{}: {}", "State parsed".green().bold(), fmt);
                    for bid in bids {
                        let bucket = ledger.get_bucket(bid).unwrap();
                        println!(
                            "{}: id = {}, amount = {}, resource = {}",
                            "Bucket".green().bold(),
                            bid,
                            bucket.amount(),
                            bucket.resource()
                        )
                    }
                }
                None => {
                    println!("{}", "Component not found".red());
                }
            }
        }
    }
    Ok(())
}
