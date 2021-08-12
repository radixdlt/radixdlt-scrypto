use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;

use crate::cli::utils::*;
use crate::ledger::*;

const ARG_ADDRESS: &'static str = "ADDRESS";

/// Prepares a subcommand that handles `show`.
pub fn prepare_show<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("show")
        .about("Show info about an address.")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_ADDRESS)
                .help("Specify the address.")
                .required(true),
        )
}

/// Processes a `show` command.
pub fn handle_show<'a>(matches: &ArgMatches<'a>) {
    let address: Address = matches.value_of(ARG_ADDRESS).unwrap().into();

    let ledger = FileBasedLedger::new(get_root_dir());
    match address {
        Address::Resource(_) => {
            let resource = ledger.get_resource(address);
            match resource {
                Some(r) => {
                    println!("Blueprint: {}", address.to_string());
                    println!("Info: {:02x?}", r.info());
                }
                None => {
                    println!("Resource not found");
                }
            }
        }
        Address::Account(_) => {
            let resource = ledger.get_account(address);
            match resource {
                Some(a) => {
                    println!("Account: {}", address.to_string());
                    for (resource, bid) in a.buckets() {
                        println!(
                            "Owned resource: address = {:?}, balance = {}",
                            resource,
                            ledger
                                .get_bucket(*bid)
                                .map(|b| b.amount())
                                .unwrap_or(U256::zero())
                        )
                    }
                }
                None => {
                    println!("Account not found");
                }
            }
        }
        Address::Blueprint(_) => {
            let resource = ledger.get_blueprint(address);
            match resource {
                Some(r) => {
                    println!("Blueprint: {}", address.to_string());
                    println!("Code size: {} bytes", r.code().len());
                }
                None => {
                    println!("Blueprint not found");
                }
            }
        }
        Address::Component(_) => {}
        _ => {
            println!("No info available");
        }
    }
}
