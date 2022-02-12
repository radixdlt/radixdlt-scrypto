use colored::*;
use radix_engine::engine::*;
use radix_engine::ledger::*;
use radix_engine::model::*;
use scrypto::rust::collections::HashSet;
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
pub fn dump_package<T: SubstateStore>(address: Address, ledger: &T) -> Result<(), DisplayError> {
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
pub fn dump_component<T: SubstateStore>(address: Address, ledger: &T) -> Result<(), DisplayError> {
    let component = ledger.get_component(address);
    match component {
        Some(c) => {
            println!("{}: {}", "Component".green().bold(), address.to_string());

            println!(
                "{}: {{ package_address: {}, blueprint_name: \"{}\" }}",
                "Blueprint".green().bold(),
                c.package_address(),
                c.blueprint_name()
            );
            let state = c.state();
            let state_validated = validate_data(state).unwrap();
            println!("{}: {}", "State".green().bold(), state_validated);

            // TODO: check authorization
            // The current implementation recursively displays all referenced maps and vaults which
            // the component may not have access to.
            let vaults_found: HashSet<Vid> = state_validated.vaults.iter().cloned().collect();

            // Dump resources
            dump_resources(address, &vaults_found, ledger)
        }
        None => Err(DisplayError::ComponentNotFound),
    }
}

fn dump_resources<T: SubstateStore>(
    address: Address,
    vaults: &HashSet<Vid>,
    ledger: &T,
) -> Result<(), DisplayError> {
    println!("{}:", "Resources".green().bold());
    for (last, vid) in vaults.iter().identify_last() {
        let vault = ledger.get_vault(&address, vid);
        let amount = vault.amount();
        let resource_address = vault.resource_address();
        let resource_def = ledger.get_resource_def(resource_address).unwrap();
        println!(
            "{} {{ amount: {}, resource_def: {}{}{} }}",
            list_item_prefix(last),
            amount,
            resource_address,
            resource_def
                .metadata()
                .get("name")
                .map(|name| format!(", name: \"{}\"", name))
                .unwrap_or(String::new()),
            resource_def
                .metadata()
                .get("symbol")
                .map(|symbol| format!(", symbol: \"{}\"", symbol))
                .unwrap_or(String::new()),
        );
        if let Supply::NonFungible { keys } = vault.total_supply() {
            for (inner_last, key) in keys.iter().identify_last() {
                let non_fungible = ledger.get_non_fungible(resource_address, key).unwrap();
                let immutable_data = validate_data(&non_fungible.immutable_data()).unwrap();
                let mutable_data = validate_data(&non_fungible.mutable_data()).unwrap();
                println!(
                    "{}  {} NON_FUNGIBLE {{ id: {}, immutable_data: {}, mutable_data: {} }}",
                    if last { " " } else { "│" },
                    list_item_prefix(inner_last),
                    key,
                    immutable_data,
                    mutable_data
                );
            }
        }
    }
    Ok(())
}

/// Dump a resource definition into console.
pub fn dump_resource_def<T: SubstateStore>(
    address: Address,
    ledger: &T,
) -> Result<(), DisplayError> {
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
            println!("{}: {}", "Flags".green().bold(), r.flags());
            println!("{}: {}", "Mutable Flags".green().bold(), r.mutable_flags());
            println!("{}: {:?}", "Authorities".green().bold(), r.authorities());
            println!("{}: {}", "Total Supply".green().bold(), r.total_supply());
            Ok(())
        }
        None => Err(DisplayError::ResourceDefNotFound),
    }
}
