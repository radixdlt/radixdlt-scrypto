use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;

use crate::*;
use radix_engine::ledger::*;

const ARG_ADDRESS: &'static str = "ADDRESS";

/// Constructs a `show` subcommand.
pub fn make_show_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_SHOW)
        .about("Shows the content of an address.")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_ADDRESS)
                .help("Specify the address.")
                .required(true),
        )
}

/// Handles a `show` request.
pub fn handle_show<'a>(matches: &ArgMatches<'a>) {
    let address: Address = matches.value_of(ARG_ADDRESS).unwrap().into();

    let ledger = FileBasedLedger::new(get_data_dir());
    match address {
        Address::System => {
            println!("Radix system address");
        }
        Address::Resource(_) | Address::RadixToken => {
            let resource = ledger.get_resource(address);
            match resource {
                Some(r) => {
                    let info = r.info();
                    println!("Resource: {}", address.to_string());
                    println!("Symbol: {}", info.symbol);
                    println!("Name: {}", info.name);
                    println!("Description: {}", info.description);
                    println!("URL: {}", info.url);
                    println!("Icon URL: {}", info.icon_url);
                    println!("Minter: {:?}", info.minter);
                    println!("supply: {:?}", info.supply);
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
        Address::Package(_) => {
            let package = ledger.get_package(address);
            match package {
                Some(b) => {
                    println!("Package: {}", address.to_string());
                    println!("Code size: {} bytes", b.code().len());
                    show_owning_resources(&ledger, address);
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
                    show_owning_resources(&ledger, address)
                }
                None => {
                    println!("Component not found");
                }
            }
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
