#![allow(unused_must_use)]
use colored::*;
use radix_engine::blueprints::resource::*;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::types::*;
use radix_engine_interface::address::AddressDisplayContext;
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataEntry, MetadataValue};
use radix_engine_interface::types::IndexedScryptoValue;
use radix_engine_common::types::NodeId;
use radix_engine_interface::blueprints::package::PackageCodeSubstate;
use radix_engine_interface::blueprints::resource::{
    AccessRulesConfig, LiquidFungibleResource, LiquidNonFungibleResource,
};
use radix_engine_interface::network::NetworkDefinition;
use std::collections::VecDeque;
use utils::ContextualDisplay;

use crate::utils::*;

/// Represents an error when displaying an entity.
#[derive(Debug, Clone)]
pub enum DisplayError {
    PackageNotFound,
    ComponentNotFound,
    ResourceManagerNotFound,
    AddressError(AddressError),
}

/// Dump a package into console.
pub fn dump_package<T: ReadableSubstateStore, O: std::io::Write>(
    package_address: PackageAddress,
    substate_db: &T,
    output: &mut O,
) -> Result<(), DisplayError> {
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let package: Option<PackageCodeSubstate> = substate_db
        .get_substate(&SubstateId(
            package_address.as_node_id(),
            TypedModuleId::ObjectState,
            PackageOffset::Package.into(),
        ))
        .map(|s| s.substate)
        .map(|s| s.to_runtime().into());
    let package = package.ok_or(DisplayError::PackageNotFound)?;

    writeln!(
        output,
        "{}: {}",
        "Package".green().bold(),
        package_address.display(&bech32_encoder)
    );
    writeln!(
        output,
        "{}: {} bytes",
        "Code size".green().bold(),
        package.code.len()
    );
    Ok(())
}

struct ComponentStateDump {
    pub raw_state: Option<IndexedScryptoValue>,
    pub owned_vaults: Option<IndexSet<NodeId>>,
    pub package_address: Option<PackageAddress>, // Native components have no package address.
    pub blueprint_name: String,                  // All components have a blueprint, native or not.
    pub access_rules: Option<AccessRulesConfig>, // Virtual Components don't have access rules.
}

/// Dump a component into console.
pub fn dump_component<T: ReadableSubstateStore, O: std::io::Write>(
    component_address: ComponentAddress,
    substate_db: &T,
    output: &mut O,
) -> Result<(), DisplayError> {
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());

    // Some branching logic is needed here to deal well with native components. Only `Normal`
    // components have a `TypeInfoSubstate`. Other components require some special handling.
    let component_state_dump = match component_address {
        ComponentAddress::Normal(..) => {
            let type_info_substate: TypeInfoSubstate = substate_db
                .get_substate(&SubstateId(
                    component_address.as_node_id(),
                    TypedModuleId::TypeInfo,
                    TypeInfoOffset::TypeInfo.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().into())
                .ok_or(DisplayError::ComponentNotFound)?;
            let access_rules_chain_substate = substate_db
                .get_substate(&SubstateId(
                    component_address.as_node_id(),
                    TypedModuleId::AccessRules,
                    &AccessRulesOffset::AccessRules.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().method_access_rules().clone())
                .ok_or(DisplayError::ComponentNotFound)?;
            let state: ComponentStateSubstate = substate_db
                .get_substate(&SubstateId(
                    component_address.as_node_id(),
                    TypedModuleId::ObjectState,
                    ComponentOffset::Component.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().into())
                .unwrap();

            let raw_state = IndexedScryptoValue::from_scrypto_value(state.0);
            let blueprint = match type_info_substate {
                TypeInfoSubstate::Object { blueprint, .. } => blueprint,
                _ => panic!("Unexpected"),
            };
            let access_rules = access_rules_chain_substate.access_rules;

            // Find all vaults owned by the component, assuming a tree structure.
            let mut vaults_found: IndexSet<NodeId> = raw_state
                .owned_node_ids()
                .iter()
                .cloned()
                .filter_map(|node_id| match node_id {
                    vault_id => Some(vault_id),
                    _ => None,
                })
                .collect();
            let mut queue: VecDeque<NodeId> = raw_state
                .owned_node_ids()
                .iter()
                .cloned()
                .filter_map(|node_id| match node_id {
                    NodeId::KeyValueStore(kv_store_id) => Some(kv_store_id),
                    _ => None,
                })
                .collect();

            while !queue.is_empty() {
                let kv_store_id = queue.pop_front().unwrap();
                let (maps, vaults) =
                    dump_kv_store(component_address, &kv_store_id, substate_db, output)?;
                queue.extend(maps);
                vaults_found.extend(vaults);
            }

            ComponentStateDump {
                raw_state: Some(raw_state),
                blueprint_name: blueprint.blueprint_name,
                package_address: Some(blueprint.package_address),
                access_rules: Some(access_rules),
                owned_vaults: Some(vaults_found),
            }
        }
        ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
        | ComponentAddress::EddsaEd25519VirtualAccount(..) => {
            // Just an account with no vaults.
            ComponentStateDump {
                raw_state: None,
                owned_vaults: Some(index_set_new()),
                package_address: None, // No package address for native components (yet).
                blueprint_name: "Account".into(),
                access_rules: None,
            }
        }
        ComponentAddress::Account(..) => {
            let account_substate = substate_db
                .get_substate(&SubstateId(
                    component_address.as_node_id(),
                    TypedModuleId::ObjectState,
                    AccountOffset::Account.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().account().clone())
                .ok_or(DisplayError::ComponentNotFound)?;
            let access_rules_chain_substate = substate_db
                .get_substate(&SubstateId(
                    component_address.as_node_id(),
                    TypedModuleId::AccessRules,
                    &AccessRulesOffset::AccessRules.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().method_access_rules().clone())
                .ok_or(DisplayError::ComponentNotFound)?;

            // Getting the vaults in the key-value store of the account
            let vaults = dump_kv_store(
                component_address,
                &account_substate.vaults.key_value_store_id(),
                substate_db,
                output,
            )
            .map(|(_, vault_ids)| vault_ids)?
            .into_iter()
            .collect();

            ComponentStateDump {
                raw_state: None,
                owned_vaults: Some(vaults),
                package_address: None, // No package address for native components (yet).
                blueprint_name: "Account".into(),
                access_rules: Some(access_rules_chain_substate.access_rules),
            }
        }
        ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
        | ComponentAddress::EddsaEd25519VirtualIdentity(..) => {
            ComponentStateDump {
                raw_state: None,
                owned_vaults: Some(index_set_new()),
                package_address: None, // No package address for native components (yet).
                blueprint_name: "Identity".into(),
                access_rules: None,
            }
        }
        ComponentAddress::Identity(..) => {
            let access_rules_chain_substate = substate_db
                .get_substate(&SubstateId(
                    component_address.as_node_id(),
                    TypedModuleId::AccessRules,
                    &AccessRulesOffset::AccessRules.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().method_access_rules().clone())
                .ok_or(DisplayError::ComponentNotFound)?;

            ComponentStateDump {
                raw_state: None,
                owned_vaults: None,
                package_address: None, // No package address for native components (yet).
                blueprint_name: "Identity".into(),
                access_rules: Some(access_rules_chain_substate.access_rules),
            }
        }
        ComponentAddress::AccessController(..) => {
            let access_controller_substate = substate_db
                .get_substate(&SubstateId(
                    component_address.as_node_id(),
                    TypedModuleId::Metadata,
                    AccessControllerOffset::AccessController.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().access_controller().clone())
                .ok_or(DisplayError::ComponentNotFound)?;
            let access_rules_chain_substate = substate_db
                .get_substate(&SubstateId(
                    component_address.as_node_id(),
                    TypedModuleId::AccessRules,
                    &AccessRulesOffset::AccessRules.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().method_access_rules().clone())
                .ok_or(DisplayError::ComponentNotFound)?;

            ComponentStateDump {
                raw_state: None,
                owned_vaults: Some([access_controller_substate.controlled_asset].into()),
                package_address: None, // No package address for native components (yet).
                blueprint_name: "AccessController".into(),
                access_rules: Some(access_rules_chain_substate.access_rules),
            }
        }
        // For the time being, the above component types are the only "dump-able" ones. We should
        // add more as we go.
        _ => Err(DisplayError::ComponentNotFound)?,
    };

    writeln!(
        output,
        "{}: {}",
        "Component".green().bold(),
        component_address.display(&bech32_encoder),
    );

    if let Some(package_address) = component_state_dump.package_address {
        writeln!(
            output,
            "{}: {{ package_address: {}, blueprint_name: \"{}\" }}",
            "Blueprint".green().bold(),
            package_address.display(&bech32_encoder),
            component_state_dump.blueprint_name
        );
    } else {
        writeln!(
            output,
            "{}: {{ Native Package, blueprint_name: \"{}\" }}",
            "Blueprint".green().bold(),
            component_state_dump.blueprint_name
        );
    }

    if let Some(access_rules) = component_state_dump.access_rules {
        writeln!(output, "{}", "Access Rules".green().bold());
        for (k, v) in access_rules.get_all_method_auth().iter() {
            writeln!(output, "{} {:?} => {:?}", list_item_prefix(false), k, v);
        }
        writeln!(
            output,
            "{} {} => {:?}",
            list_item_prefix(true),
            "Default",
            access_rules.get_default()
        );
    }

    if let Some(raw_state) = component_state_dump.raw_state {
        let value_display_context =
            ScryptoValueDisplayContext::with_optional_bech32(Some(&bech32_encoder));
        writeln!(
            output,
            "{}: {}",
            "State".green().bold(),
            raw_state.display(value_display_context)
        );
    }

    if let Some(vaults) = component_state_dump.owned_vaults {
        dump_resources(&vaults, substate_db, output);
    }

    Ok(())
}

fn dump_kv_store<T: ReadableSubstateStore, O: std::io::Write>(
    component_address: ComponentAddress,
    kv_store_id: &NodeId,
    substate_db: &T,
    output: &mut O,
) -> Result<(Vec<NodeId>, Vec<NodeId>), DisplayError> {
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let mut owned_kv_stores = Vec::new();
    let mut owned_vaults = Vec::new();
    let map = substate_db.get_kv_store_entries(kv_store_id);
    writeln!(
        output,
        "{}: {}, {}",
        "Key Value Store".green().bold(),
        component_address.to_string(AddressDisplayContext::with_encoder(&bech32_encoder)),
        hex::encode(kv_store_id)
    );
    for (last, (key, substate)) in map.iter().identify_last() {
        let substate = substate.clone().to_runtime();
        if let Option::Some(value) = &substate.kv_store_entry() {
            let key: ScryptoValue = scrypto_decode(&key).unwrap();
            let value_display_context =
                ScryptoValueDisplayContext::with_optional_bech32(Some(&bech32_encoder));
            writeln!(
                output,
                "{} {} => {}",
                list_item_prefix(last),
                key.display(value_display_context),
                value.display(value_display_context)
            );

            if let Some(substate) = substate.kv_store_entry() {
                let (_, own, _) =
                    IndexedScryptoValue::from_scrypto_value(substate.clone()).unpack();
                for owned_node in own {
                    match owned_node {
                        vault_id => {
                            owned_vaults.push(vault_id);
                        }
                        NodeId::KeyValueStore(kv_store_id) => {
                            owned_kv_stores.push(kv_store_id);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok((owned_kv_stores, owned_vaults))
}

fn dump_resources<T: ReadableSubstateStore, O: std::io::Write>(
    vaults: &IndexSet<NodeId>,
    substate_store: &T,
    output: &mut O,
) -> Result<(), DisplayError> {
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());

    writeln!(output, "{}:", "Resources".green().bold());
    for (last, vault_id) in vaults.iter().identify_last() {
        // READ vault info
        let vault_info: VaultInfoSubstate = substate_db
            .get_substate(&SubstateId(
                NodeId::Object(*vault_id),
                TypedModuleId::ObjectState,
                VaultOffset::Vault.into(),
            ))
            .map(|s| s.substate)
            .map(|s| s.into())
            .unwrap();

        // READ resource manager
        let resource_address = vault_info.resource_address;

        let name_metadata: Option<Option<ScryptoValue>> = substate_db
            .get_substate(&SubstateId(
                resource_address.as_node_id(),
                TypedModuleId::Metadata,
                SubstateKey::KeyValueStore(KeyValueStoreOffset::Entry(
                    scrypto_encode("name").unwrap(),
                )),
            ))
            .map(|s| s.substate)
            .map(|s| s.to_runtime().into());
        let name_metadata = match name_metadata {
            Some(Option::Some(scrypto_value)) => {
                let entry: MetadataEntry =
                    scrypto_decode(&scrypto_encode(&scrypto_value).unwrap()).unwrap();
                match entry {
                    MetadataEntry::Value(MetadataValue::String(value)) => Some(value),
                    _ => None,
                }
            }
            _ => None,
        }
        .map(|name| format!(", name: \"{}\"", name))
        .unwrap_or(String::new());

        let symbol_metadata: Option<Option<ScryptoValue>> = substate_db
            .get_substate(&SubstateId(
                resource_address.as_node_id(),
                TypedModuleId::Metadata,
                SubstateKey::KeyValueStore(KeyValueStoreOffset::Entry(
                    scrypto_encode("symbol").unwrap(),
                )),
            ))
            .map(|s| s.substate)
            .map(|s| s.to_runtime().into());
        let symbol_metadata = match symbol_metadata {
            Some(Option::Some(scrypto_value)) => {
                let entry: MetadataEntry =
                    scrypto_decode(&scrypto_encode(&scrypto_value).unwrap()).unwrap();
                match entry {
                    MetadataEntry::Value(MetadataValue::String(value)) => Some(value),
                    _ => None,
                }
            }
            _ => None,
        }
        .map(|name| format!(", symbol: \"{}\"", name))
        .unwrap_or(String::new());

        // DUMP resource
        let amount = if vault_info.resource_type.is_fungible() {
            let vault: LiquidFungibleResource = substate_db
                .get_substate(&SubstateId(
                    NodeId::Object(*vault_id),
                    TypedModuleId::ObjectState,
                    VaultOffset::Vault.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.into())
                .unwrap();
            vault.amount()
        } else {
            let vault: LiquidNonFungibleResource = substate_db
                .get_substate(&SubstateId(
                    NodeId::Object(*vault_id),
                    TypedModuleId::ObjectState,
                    VaultOffset::Vault.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.into())
                .unwrap();
            vault.amount()
        };
        writeln!(
            output,
            "{} {{ amount: {}, resource address: {}, {:?}{:?} }}",
            list_item_prefix(last),
            amount,
            resource_address.display(&bech32_encoder),
            name_metadata,
            symbol_metadata,
        );

        // DUMP non-fungibles
        if !vault_info.resource_type.is_fungible() {
            let resource_manager: Option<NonFungibleResourceManagerSubstate> = substate_db
                .get_substate(&SubstateId(
                    resource_address.as_node_id(),
                    TypedModuleId::ObjectState,
                    ResourceManagerOffset::ResourceManager.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().into());
            let resource_manager = resource_manager.ok_or(DisplayError::ResourceManagerNotFound)?;

            let vault: LiquidNonFungibleResource = substate_db
                .get_substate(&SubstateId(
                    NodeId::Object(*vault_id),
                    TypedModuleId::ObjectState,
                    VaultOffset::Vault.into(),
                ))
                .map(|s| s.substate)
                .map(|s| s.into())
                .unwrap();

            let ids = vault.ids();
            let non_fungible_id = resource_manager.non_fungible_table;
            for (inner_last, id) in ids.iter().identify_last() {
                let non_fungible: Option<ScryptoValue> = substate_db
                    .get_substate(&SubstateId(
                        NodeId::KeyValueStore(non_fungible_id),
                        TypedModuleId::ObjectState,
                        SubstateKey::KeyValueStore(KeyValueStoreOffset::Entry(
                            scrypto_encode(id).unwrap(),
                        )),
                    ))
                    .map(|s| s.substate.to_runtime())
                    .map(|s| s.into())
                    .unwrap();
                if let Option::Some(value) = non_fungible {
                    let id = IndexedScryptoValue::from_typed(id);
                    let value_display_context =
                        ScryptoValueDisplayContext::with_optional_bech32(Some(&bech32_encoder));
                    writeln!(
                        output,
                        "{}  {} NonFungible {{ id: {}, data: {} }}",
                        if last { " " } else { "│" },
                        list_item_prefix(inner_last),
                        id.display(value_display_context),
                        value.display(value_display_context),
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
    substate_db: &T,
    output: &mut O,
) -> Result<(), DisplayError> {
    let resource_manager: Option<FungibleResourceManagerSubstate> = substate_db
        .get_substate(&SubstateId(
            resource_address.as_node_id(),
            TypedModuleId::ObjectState,
            ResourceManagerOffset::ResourceManager.into(),
        ))
        .map(|s| s.substate)
        .map(|s| s.to_runtime().into());
    let resource_manager = resource_manager.ok_or(DisplayError::ResourceManagerNotFound)?;

    writeln!(
        output,
        "{}: {}",
        "Total Supply".green().bold(),
        resource_manager.total_supply
    );
    Ok(())
}
