use colored::*;
use radix_engine::ledger::*;
use radix_engine::model::*;
use radix_engine::utils::*;
use scrypto::types::*;

use crate::utils::*;

/// Represents an error when displaying an entity.
#[derive(Debug, Clone)]
pub enum DisplayError {
    PackageNotFound,
    ComponentNotFound,
    ResourceDefNotFound,
}

/// Dump a package into console.
pub fn dump_package<T: Ledger>(address: Address, ledger: &T) -> Result<(), DisplayError> {
    let package = ledger.get_package(address);
    match package {
        Some(b) => {
            println!("{}: {}", "Package".green().bold(), address.to_string());
            println!("{}: {} bytes", "Code size".green().bold(), b.code().len());
            Ok(())
        }
        None => Err(DisplayError::PackageNotFound),
    }
}

/// Dump a component into console.
pub fn dump_component<T: Ledger>(address: Address, ledger: &T) -> Result<(), DisplayError> {
    let component = ledger.get_component(address);
    match component {
        Some(c) => {
            println!("{}: {}", "Component".green().bold(), address.to_string());

            println!(
                "{}: {{ package: {}, name: {} }}",
                "Blueprint".green().bold(),
                c.package(),
                c.name()
            );
            let mut vaults = vec![];
            println!(
                "{}: {}",
                "State".green().bold(),
                format_data_with_ledger(c.state(Auth::NoAuth).unwrap(), ledger, &mut vaults)
                    .unwrap()
            );

            println!("{}:", "Resources".green().bold());
            for (last, vid) in vaults.iter().identify_last() {
                let vault = ledger.get_vault(*vid).unwrap();
                let amount = vault.amount(Auth::NoAuth).unwrap();
                let resource_def_address = vault.resource_def(Auth::NoAuth).unwrap();
                let resource_def = ledger.get_resource_def(resource_def_address).unwrap();
                println!(
                    "{} {{ amount: {}, resource_def: {}{}{} }}",
                    list_item_prefix(last),
                    amount,
                    resource_def_address,
                    resource_def
                        .metadata()
                        .get("name")
                        .map(|name| format!(", name: {}", name))
                        .unwrap_or(String::new()),
                    resource_def
                        .metadata()
                        .get("symbol")
                        .map(|symbol| format!(", symbol: {}", symbol))
                        .unwrap_or(String::new()),
                );
                if let Supply::NonFungible { entries } = vault.total_supply(Auth::NoAuth).unwrap() {
                    // TODO how to deal with the case where a vault id is referenced in the NFT
                    let mut vaults = Vec::new();
                    for (inner_last, id) in entries.iter().identify_last() {
                        let data = []; // FIXME retrieve NFT data
                        println!(
                            "{}  {} NFT {{ id: {}, data: {} }}",
                            if last { " " } else { "│" },
                            list_item_prefix(inner_last),
                            id,
                            format_data_with_ledger(&data, ledger, &mut vaults).unwrap()
                        );
                    }
                }
            }
            Ok(())
        }
        None => Err(DisplayError::ComponentNotFound),
    }
}

/// Dump a resource definition into console.
pub fn dump_resource_def<T: Ledger>(address: Address, ledger: &T) -> Result<(), DisplayError> {
    let resource_def = ledger.get_resource_def(address);
    match resource_def {
        Some(r) => {
            println!(
                "{}: {:?}",
                "Resource Type".green().bold(),
                r.resource_type()
            );
            println!("{}: {}", "Metadata".green().bold(), r.metadata().len());
            for (last, e) in r.metadata().iter().identify_last() {
                println!("{} {}: {}", list_item_prefix(last), e.0.green().bold(), e.1);
            }
            println!("{}: {}", "Total Supply".green().bold(), r.total_supply());
            println!("{}: {:?}", "Mint Auth".green().bold(), r.minter());
            Ok(())
        }
        None => Err(DisplayError::ResourceDefNotFound),
    }
}
