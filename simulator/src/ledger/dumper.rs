use colored::*;
use radix_engine::ledger::*;
use radix_engine::model::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::HashSet;
use std::collections::VecDeque;

use crate::utils::*;

/// Represents an error when displaying an entity.
#[derive(Debug, Clone)]
pub enum DisplayError {
    PackageNotFound,
    ComponentNotFound,
    ResourceDefNotFound,
}

/// Dump a package into console.
pub fn dump_package<T: SubstateStore>(
    package_address: PackageAddress,
    substate_store: &T,
) -> Result<(), DisplayError> {
    let package: Option<Package> = substate_store
        .get_decoded_substate(&package_address)
        .map(|(package, _)| package);
    match package {
        Some(b) => {
            println!(
                "{}: {}",
                "Package".green().bold(),
                package_address.to_string()
            );
            println!("{}: {} bytes", "Code size".green().bold(), b.code().len());
            Ok(())
        }
        None => Err(DisplayError::PackageNotFound),
    }
}

/// Dump a component into console.
pub fn dump_component<T: SubstateStore + QueryableSubstateStore>(
    component_address: ComponentAddress,
    substate_store: &T,
) -> Result<(), DisplayError> {
    let component: Option<Component> = substate_store
        .get_decoded_substate(&component_address)
        .map(|(component, _)| component);
    match component {
        Some(c) => {
            println!(
                "{}: {}",
                "Component".green().bold(),
                component_address.to_string()
            );

            println!(
                "{}: {{ package_address: {}, blueprint_name: \"{}\" }}",
                "Blueprint".green().bold(),
                c.package_address(),
                c.blueprint_name()
            );

            println!("{}", "Authorization".green().bold());
            for (last, (k, v)) in c.auth_rules().iter().identify_last() {
                println!("{} {:?} => {:?}", list_item_prefix(last), k, v);
            }

            let state = c.state();
            let state_data = ValidatedData::from_slice(state).unwrap();
            println!("{}: {}", "State".green().bold(), state_data);

            // Find all vaults owned by the component, assuming a tree structure.
            let mut vaults_found: HashSet<VaultId> = state_data.vault_ids.iter().cloned().collect();
            let mut queue: VecDeque<LazyMapId> = state_data.lazy_map_ids.iter().cloned().collect();
            while !queue.is_empty() {
                let lazy_map_id = queue.pop_front().unwrap();
                let (maps, vaults) =
                    dump_lazy_map(component_address, &lazy_map_id, substate_store)?;
                queue.extend(maps);
                vaults_found.extend(vaults);
            }

            // Dump resources
            dump_resources(component_address, &vaults_found, substate_store)
        }
        None => Err(DisplayError::ComponentNotFound),
    }
}

fn dump_lazy_map<T: SubstateStore + QueryableSubstateStore>(
    component_address: ComponentAddress,
    lazy_map_id: &LazyMapId,
    substate_store: &T,
) -> Result<(Vec<LazyMapId>, Vec<VaultId>), DisplayError> {
    let mut referenced_maps = Vec::new();
    let mut referenced_vaults = Vec::new();
    let map = substate_store.get_lazy_map_entries(component_address, lazy_map_id);
    println!(
        "{}: {:?}{:?}",
        "Lazy Map".green().bold(),
        component_address,
        lazy_map_id
    );
    for (last, (k, v)) in map.iter().identify_last() {
        let k_validated = ValidatedData::from_slice(k).unwrap();
        let v_validated = ValidatedData::from_slice(v).unwrap();
        println!(
            "{} {} => {}",
            list_item_prefix(last),
            k_validated,
            v_validated
        );
        referenced_maps.extend(v_validated.lazy_map_ids);
        referenced_vaults.extend(v_validated.vault_ids);
    }
    Ok((referenced_maps, referenced_vaults))
}

fn dump_resources<T: SubstateStore>(
    component_address: ComponentAddress,
    vaults: &HashSet<VaultId>,
    substate_store: &T,
) -> Result<(), DisplayError> {
    println!("{}:", "Resources".green().bold());
    for (last, vault_id) in vaults.iter().identify_last() {
        let vault: Vault = substate_store
            .get_decoded_child_substate(&component_address, vault_id)
            .unwrap()
            .0;

        let amount = vault.total_amount();
        let resource_address = vault.resource_address();
        let resource_def: ResourceDef = substate_store
            .get_decoded_substate(&resource_address)
            .map(|(resource, _)| resource)
            .unwrap();
        println!(
            "{} {{ amount: {}, resource definition: {}{}{} }}",
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
        if matches!(resource_def.resource_type(), ResourceType::NonFungible) {
            let ids = vault.total_ids().unwrap();
            for (inner_last, id) in ids.iter().identify_last() {
                let non_fungible: NonFungible = substate_store
                    .get_decoded_child_substate(&resource_address, id)
                    .unwrap()
                    .0;

                let immutable_data =
                    ValidatedData::from_slice(&non_fungible.immutable_data()).unwrap();
                let mutable_data = ValidatedData::from_slice(&non_fungible.mutable_data()).unwrap();
                println!(
                    "{}  {} NON_FUNGIBLE {{ id: {}, immutable_data: {}, mutable_data: {} }}",
                    if last { " " } else { "│" },
                    list_item_prefix(inner_last),
                    id,
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
    resource_address: ResourceAddress,
    substate_store: &T,
) -> Result<(), DisplayError> {
    let resource_def: Option<ResourceDef> = substate_store
        .get_decoded_substate(&resource_address)
        .map(|(resource, _)| resource);
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
            println!("{}: {}", "Total Supply".green().bold(), r.total_supply());
            Ok(())
        }
        None => Err(DisplayError::ResourceDefNotFound),
    }
}
