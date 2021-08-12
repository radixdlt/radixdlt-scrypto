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
                    println!("Resource: {}", address.to_string());
                    println!("Info: {:02x?}", r.info());
                }
                None => {
                    println!("Resource not found");
                }
            }
        }
        Address::PublicKey(_) => {
            let account = ledger.get_account(address);
            match account {
                Some(_) => {
                    println!("Account: {}", address.to_string());
                    show_owning_resources(&ledger, address)
                }
                None => {
                    println!("Account not found");
                }
            }
        }
        Address::Blueprint(_) => {
            let blueprint = ledger.get_blueprint(address);
            match blueprint {
                Some(b) => {
                    println!("Blueprint: {}", address.to_string());
                    println!("Code size: {} bytes", b.code().len());
                    show_owning_resources(&ledger, address);
                }
                None => {
                    println!("Blueprint not found");
                }
            }
        }
        Address::Component(_) => {
            let component = ledger.get_component(address);
            match component {
                Some(c) => {
                    println!("Component: {}", address.to_string());
                    println!("State: {:02x?}", c.state());
                    show_owning_resources(&ledger, address)
                }
                None => {
                    println!("Component not found");
                }
            }
        }
        _ => {
            println!("No info available");
        }
    }
}

fn show_owning_resources<T: Ledger>(ledger: &T, address: Address) {
    if let Some(account) = ledger.get_account(address) {
        for (resource, bid) in account.buckets() {
            println!(
                "Owns resource: address = {}, balance = {}",
                resource.to_string(),
                ledger
                    .get_bucket(*bid)
                    .map(|b| b.amount())
                    .unwrap_or(U256::zero())
            )
        }
    }
}
