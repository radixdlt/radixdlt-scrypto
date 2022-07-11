#![allow(unused_must_use)]
use colored::*;
use radix_engine::ledger::*;
use radix_engine::model::*;
use sbor::rust::collections::HashSet;
use scrypto::address::Bech32Encoder;
use scrypto::buffer::{scrypto_decode, scrypto_encode};
use scrypto::core::Network;
use scrypto::engine::types::*;
use scrypto::values::*;
use std::collections::VecDeque;

use crate::utils::*;

/// Represents an error when displaying an entity.
#[derive(Debug, Clone)]
pub enum DisplayError {
    PackageNotFound,
    ComponentNotFound,
    ResourceManagerNotFound,
}

/// Dump a package into console.
pub fn dump_package<T: ReadableSubstateStore, O: std::io::Write>(
    package_address: PackageAddress,
    substate_store: &T,
    output: &mut O,
) -> Result<(), DisplayError> {
    let bech32_encoder = Bech32Encoder::new_from_network(&Network::LocalSimulator);

    let package: Option<ValidatedPackage> = substate_store
        .get_decoded_substate(&package_address)
        .map(|(package, _)| package);
    match package {
        Some(b) => {
            writeln!(
                output,
                "{}: {}",
                "Package".green().bold(),
                bech32_encoder
                    .encode_package_address(&package_address)
                    .unwrap()
            );
            writeln!(
                output,
                "{}: {} bytes",
                "Code size".green().bold(),
                b.code().len()
            );
            Ok(())
        }
        None => Err(DisplayError::PackageNotFound),
    }
}

/// Dump a component into console.
pub fn dump_component<T: ReadableSubstateStore + QueryableSubstateStore, O: std::io::Write>(
    component_address: ComponentAddress,
    substate_store: &T,
    output: &mut O,
) -> Result<(), DisplayError> {
    let bech32_encoder = Bech32Encoder::new_from_network(&Network::LocalSimulator);

    let component: Option<Component> = substate_store
        .get_decoded_substate(&component_address)
        .map(|(component, _)| component);
    match component {
        Some(c) => {
            writeln!(
                output,
                "{}: {}",
                "Component".green().bold(),
                bech32_encoder
                    .encode_component_address(&component_address)
                    .unwrap()
            );

            writeln!(
                output,
                "{}: {{ package_address: {}, blueprint_name: \"{}\" }}",
                "Blueprint".green().bold(),
                c.package_address(),
                c.blueprint_name()
            );

            writeln!(output, "{}", "Authorization".green().bold());
            for (_, auth) in c.authorization().iter().identify_last() {
                for (last, (k, v)) in auth.iter().identify_last() {
                    writeln!(output, "{} {:?} => {:?}", list_item_prefix(last), k, v);
                }
            }

            let state = c.state();
            let state_data = ScryptoValue::from_slice(state).unwrap();
            writeln!(output, "{}: {}", "State".green().bold(), state_data);

            // Find all vaults owned by the component, assuming a tree structure.
            let mut vaults_found: HashSet<VaultId> = state_data.vault_ids.iter().cloned().collect();
            let mut queue: VecDeque<KeyValueStoreId> =
                state_data.kv_store_ids.iter().cloned().collect();
            while !queue.is_empty() {
                let kv_store_id = queue.pop_front().unwrap();
                let (maps, vaults) =
                    dump_kv_store(component_address, &kv_store_id, substate_store, output)?;
                queue.extend(maps);
                vaults_found.extend(vaults);
            }

            // Dump resources
            dump_resources(component_address, &vaults_found, substate_store, output)
        }
        None => Err(DisplayError::ComponentNotFound),
    }
}

fn dump_kv_store<T: ReadableSubstateStore + QueryableSubstateStore, O: std::io::Write>(
    component_address: ComponentAddress,
    kv_store_id: &KeyValueStoreId,
    substate_store: &T,
    output: &mut O,
) -> Result<(Vec<KeyValueStoreId>, Vec<VaultId>), DisplayError> {
    let mut referenced_maps = Vec::new();
    let mut referenced_vaults = Vec::new();
    let map = substate_store.get_kv_store_entries(component_address, kv_store_id);
    writeln!(
        output,
        "{}: {:?}{:?}",
        "Key Value Store".green().bold(),
        component_address,
        kv_store_id
    );
    for (last, (k, v)) in map.iter().identify_last() {
        let key = ScryptoValue::from_slice(k).unwrap();
        let value_wrapper: Option<Vec<u8>> = scrypto_decode(v).unwrap();
        if let Some(v) = value_wrapper {
            let value = ScryptoValue::from_slice(&v).unwrap();
            writeln!(output, "{} {} => {}", list_item_prefix(last), key, value);
            referenced_maps.extend(value.kv_store_ids);
            referenced_vaults.extend(value.vault_ids);
        }
    }
    Ok((referenced_maps, referenced_vaults))
}

fn dump_resources<T: ReadableSubstateStore, O: std::io::Write>(
    component_address: ComponentAddress,
    vaults: &HashSet<VaultId>,
    substate_store: &T,
    output: &mut O,
) -> Result<(), DisplayError> {
    let bech32_encoder = Bech32Encoder::new_from_network(&Network::LocalSimulator);

    writeln!(output, "{}:", "Resources".green().bold());
    for (last, vault_id) in vaults.iter().identify_last() {
        let mut vault_address = scrypto_encode(&component_address);
        vault_address.extend(scrypto_encode(vault_id));
        let substate = substate_store.get_substate(&vault_address).unwrap();
        let vault: Vault = scrypto_decode(&substate.value).unwrap();
        let amount = vault.total_amount();
        let resource_address = vault.resource_address();
        let resource_manager: ResourceManager = substate_store
            .get_decoded_substate(&resource_address)
            .map(|(resource, _)| resource)
            .unwrap();
        writeln!(
            output,
            "{} {{ amount: {}, resource address: {}{}{} }}",
            list_item_prefix(last),
            amount,
            bech32_encoder
                .encode_resource_address(&resource_address)
                .unwrap(),
            resource_manager
                .metadata()
                .get("name")
                .map(|name| format!(", name: \"{}\"", name))
                .unwrap_or(String::new()),
            resource_manager
                .metadata()
                .get("symbol")
                .map(|symbol| format!(", symbol: \"{}\"", symbol))
                .unwrap_or(String::new()),
        );
        if matches!(resource_manager.resource_type(), ResourceType::NonFungible) {
            let ids = vault.total_ids().unwrap();
            for (inner_last, id) in ids.iter().identify_last() {
                let mut nf_address = scrypto_encode(&resource_address);
                nf_address.push(0u8);
                nf_address.extend(id.to_vec());

                let non_fungible: Option<NonFungible> =
                    scrypto_decode(&substate_store.get_substate(&nf_address).unwrap().value)
                        .unwrap();

                let id = ScryptoValue::from_slice(&id.to_vec()).unwrap();

                if let Some(non_fungible) = non_fungible {
                    let immutable_data =
                        ScryptoValue::from_slice(&non_fungible.immutable_data()).unwrap();
                    let mutable_data =
                        ScryptoValue::from_slice(&non_fungible.mutable_data()).unwrap();
                    writeln!(
                        output,
                        "{}  {} NonFungible {{ id: {}, immutable_data: {}, mutable_data: {} }}",
                        if last { " " } else { "│" },
                        list_item_prefix(inner_last),
                        id,
                        immutable_data,
                        mutable_data
                    );
                }
            }
        }
    }
    Ok(())
}

/// Dump a resource into console.
pub fn dump_resource_manager<T: ReadableSubstateStore, O: std::io::Write>(
    resource_address: ResourceAddress,
    substate_store: &T,
    output: &mut O,
) -> Result<(), DisplayError> {
    let resource_manager: Option<ResourceManager> = substate_store
        .get_decoded_substate(&resource_address)
        .map(|(resource, _)| resource);
    match resource_manager {
        Some(r) => {
            writeln!(
                output,
                "{}: {:?}",
                "Resource Type".green().bold(),
                r.resource_type()
            );
            writeln!(
                output,
                "{}: {}",
                "Metadata".green().bold(),
                r.metadata().len()
            );
            for (last, e) in r.metadata().iter().identify_last() {
                writeln!(
                    output,
                    "{} {}: {}",
                    list_item_prefix(last),
                    e.0.green().bold(),
                    e.1
                );
            }
            writeln!(
                output,
                "{}: {}",
                "Total Supply".green().bold(),
                r.total_supply()
            );
            Ok(())
        }
        None => Err(DisplayError::ResourceManagerNotFound),
    }
}
