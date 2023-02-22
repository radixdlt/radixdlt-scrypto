#![allow(unused_must_use)]
use colored::*;
use radix_engine::blueprints::resource::{
    NonFungibleSubstate, ResourceManagerSubstate, VaultSubstate,
};
use radix_engine::ledger::*;
use radix_engine::system::global::GlobalSubstate;
use radix_engine::system::node_modules::metadata::MetadataSubstate;
use radix_engine::types::*;
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::package::PackageCodeSubstate;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::blueprints::resource::{AccessRules, ResourceType};
use radix_engine_interface::data::{IndexedScryptoValue, ScryptoValueDisplayContext};
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
    substate_store: &T,
    output: &mut O,
) -> Result<(), DisplayError> {
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());

    let global: Option<GlobalSubstate> = substate_store
        .get_substate(&SubstateId(
            RENodeId::Global(Address::Package(package_address)),
            NodeModuleId::SELF,
            SubstateOffset::Global(GlobalOffset::Global),
        ))
        .map(|s| s.substate)
        .map(|s| s.to_runtime().into());
    let package: Option<PackageCodeSubstate> = global.and_then(|global| {
        substate_store
            .get_substate(&SubstateId(
                global.node_deref(),
                NodeModuleId::SELF,
                SubstateOffset::Package(PackageOffset::Code),
            ))
            .map(|s| s.substate)
            .map(|s| s.to_runtime().into())
    });
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
    pub owned_vaults: Option<HashSet<VaultId>>,
    pub package_address: Option<PackageAddress>, // Native components have no package address.
    pub blueprint_name: String,                  // All components have a blueprint, native or not.
    pub access_rules: Option<Vec<AccessRules>>,  // Virtual Components don't have access rules.
    pub metadata: Option<BTreeMap<String, String>>,
}

/// Dump a component into console.
pub fn dump_component<T: ReadableSubstateStore + QueryableSubstateStore, O: std::io::Write>(
    component_address: ComponentAddress,
    substate_store: &T,
    output: &mut O,
) -> Result<(), DisplayError> {
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());

    // Dereference the global component address to get the component id
    let component_id = {
        let node_id = RENodeId::Global(Address::Component(component_address));
        substate_store
            .get_substate(&SubstateId(
                node_id,
                NodeModuleId::SELF,
                SubstateOffset::Global(GlobalOffset::Global),
            ))
            .map(|s| s.substate)
            .map(|s| s.to_runtime().global().node_deref())
            .ok_or(DisplayError::ComponentNotFound)?
    };

    // Some branching logic is needed here to deal well with native components. Only `Normal`
    // components have a `ComponentInfoSubstate`. Other components require some special handling.
    let component_state_dump = match component_address {
        ComponentAddress::Normal(..) => {
            let component_info_substate: TypeInfoSubstate = substate_store
                .get_substate(&SubstateId(
                    component_id,
                    NodeModuleId::TypeInfo,
                    SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().into())
                .ok_or(DisplayError::ComponentNotFound)?;
            let access_rules_chain_substate = substate_store
                .get_substate(&SubstateId(
                    component_id,
                    NodeModuleId::AccessRules,
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().access_rules_chain().clone())
                .ok_or(DisplayError::ComponentNotFound)?;
            let state: ComponentStateSubstate = substate_store
                .get_substate(&SubstateId(
                    component_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Component(ComponentOffset::State0),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().into())
                .unwrap();
            let metadata_substate = substate_store
                .get_substate(&SubstateId(
                    component_id,
                    NodeModuleId::Metadata,
                    SubstateOffset::Metadata(MetadataOffset::Metadata),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().metadata().clone())
                .ok_or(DisplayError::ComponentNotFound)?;

            let raw_state = IndexedScryptoValue::from_slice(&state.raw).unwrap();
            let package_address = component_info_substate.package_address;
            let blueprint_name = component_info_substate.blueprint_name;
            let access_rules = access_rules_chain_substate.access_rules_chain;
            let metadata = metadata_substate.metadata;

            // Find all vaults owned by the component, assuming a tree structure.
            let mut vaults_found: HashSet<VaultId> = raw_state
                .owned_node_ids()
                .iter()
                .cloned()
                .filter_map(|node_id| match node_id {
                    RENodeId::Vault(vault_id) => Some(vault_id),
                    _ => None,
                })
                .collect();
            let mut queue: VecDeque<KeyValueStoreId> = raw_state
                .owned_node_ids()
                .iter()
                .cloned()
                .filter_map(|node_id| match node_id {
                    RENodeId::KeyValueStore(kv_store_id) => Some(kv_store_id),
                    _ => None,
                })
                .collect();
            while !queue.is_empty() {
                let kv_store_id = queue.pop_front().unwrap();
                let (maps, vaults) =
                    dump_kv_store(component_address, &kv_store_id, substate_store, output)?;
                queue.extend(maps);
                vaults_found.extend(vaults);
            }

            ComponentStateDump {
                raw_state: Some(raw_state),
                blueprint_name,
                package_address: Some(package_address),
                metadata: Some(metadata),
                access_rules: Some(access_rules),
                owned_vaults: Some(vaults_found),
            }
        }
        ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
        | ComponentAddress::EddsaEd25519VirtualAccount(..) => {
            // Just an account with no vaults.
            ComponentStateDump {
                raw_state: None,
                owned_vaults: Some(HashSet::new()),
                package_address: None, // No package address for native components (yet).
                blueprint_name: "Account".into(),
                access_rules: None,
                metadata: None,
            }
        }
        ComponentAddress::Account(..) => {
            let account_substate = substate_store
                .get_substate(&SubstateId(
                    component_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Account(AccountOffset::Account),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().account().clone())
                .ok_or(DisplayError::ComponentNotFound)?;
            let access_rules_chain_substate = substate_store
                .get_substate(&SubstateId(
                    component_id,
                    NodeModuleId::AccessRules,
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().access_rules_chain().clone())
                .ok_or(DisplayError::ComponentNotFound)?;

            // Getting the vaults in the key-value store of the account
            let vaults = dump_kv_store(
                component_address,
                &account_substate.vaults.key_value_store_id(),
                substate_store,
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
                access_rules: Some(access_rules_chain_substate.access_rules_chain),
                metadata: None,
            }
        }
        ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
        | ComponentAddress::EddsaEd25519VirtualIdentity(..) => {
            ComponentStateDump {
                raw_state: None,
                owned_vaults: Some(HashSet::new()),
                package_address: None, // No package address for native components (yet).
                blueprint_name: "Identity".into(),
                access_rules: None,
                metadata: None,
            }
        }
        ComponentAddress::Identity(..) => {
            let metadata_substate = substate_store
                .get_substate(&SubstateId(
                    component_id,
                    NodeModuleId::Metadata,
                    SubstateOffset::Metadata(MetadataOffset::Metadata),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().metadata().clone())
                .ok_or(DisplayError::ComponentNotFound)?;
            let access_rules_chain_substate = substate_store
                .get_substate(&SubstateId(
                    component_id,
                    NodeModuleId::AccessRules,
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().access_rules_chain().clone())
                .ok_or(DisplayError::ComponentNotFound)?;

            ComponentStateDump {
                raw_state: None,
                owned_vaults: None,
                package_address: None, // No package address for native components (yet).
                blueprint_name: "Identity".into(),
                access_rules: Some(access_rules_chain_substate.access_rules_chain),
                metadata: Some(metadata_substate.metadata),
            }
        }
        ComponentAddress::AccessController(..) => {
            let access_controller_substate = substate_store
                .get_substate(&SubstateId(
                    component_id,
                    NodeModuleId::Metadata,
                    SubstateOffset::Metadata(MetadataOffset::Metadata),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().access_controller().clone())
                .ok_or(DisplayError::ComponentNotFound)?;
            let access_rules_chain_substate = substate_store
                .get_substate(&SubstateId(
                    component_id,
                    NodeModuleId::AccessRules,
                    SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().access_rules_chain().clone())
                .ok_or(DisplayError::ComponentNotFound)?;

            ComponentStateDump {
                raw_state: None,
                owned_vaults: Some([access_controller_substate.controlled_asset].into()),
                package_address: None, // No package address for native components (yet).
                blueprint_name: "AccessController".into(),
                access_rules: Some(access_rules_chain_substate.access_rules_chain),
                metadata: None,
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
        for (_, auth) in access_rules.iter().identify_last() {
            for (last, (k, v)) in auth.get_all_method_auth().iter().identify_last() {
                writeln!(output, "{} {:?} => {:?}", list_item_prefix(last), k, v);
            }
            writeln!(output, "Default: {:?}", auth.get_default());
        }
    }

    if let Some(raw_state) = component_state_dump.raw_state {
        let value_display_context =
            ScryptoValueDisplayContext::with_optional_bench32(Some(&bech32_encoder));
        writeln!(
            output,
            "{}: {}",
            "State".green().bold(),
            raw_state.display(value_display_context)
        );
    }

    if let Some(metadata) = component_state_dump.metadata {
        if !metadata.is_empty() {
            writeln!(output, "{}", "Metadata".green().bold());
            for (last, (key, value)) in metadata.iter().identify_last() {
                writeln!(
                    output,
                    "{} {:?} => {:?}",
                    list_item_prefix(last),
                    key,
                    value
                );
            }
        }
    }

    if let Some(vaults) = component_state_dump.owned_vaults {
        dump_resources(&vaults, substate_store, output);
    }

    Ok(())
}

fn dump_kv_store<T: ReadableSubstateStore + QueryableSubstateStore, O: std::io::Write>(
    component_address: ComponentAddress,
    kv_store_id: &KeyValueStoreId,
    substate_store: &T,
    output: &mut O,
) -> Result<(Vec<KeyValueStoreId>, Vec<VaultId>), DisplayError> {
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let mut owned_kv_stores = Vec::new();
    let mut owned_vaults = Vec::new();
    let map = substate_store.get_kv_store_entries(kv_store_id);
    writeln!(
        output,
        "{}: {:?}{:?}",
        "Key Value Store".green().bold(),
        component_address,
        kv_store_id
    );
    for (last, (_hash, substate)) in map.iter().identify_last() {
        let substate = substate.clone().to_runtime();
        if let KeyValueStoreEntrySubstate::Some(key, value) = &substate.kv_store_entry() {
            let value_display_context =
                ScryptoValueDisplayContext::with_optional_bench32(Some(&bech32_encoder));
            writeln!(
                output,
                "{} {} => {}",
                list_item_prefix(last),
                key.display(value_display_context),
                value.display(value_display_context)
            );
            for owned_node in substate.kv_store_entry().owned_node_ids() {
                match owned_node {
                    RENodeId::Vault(vault_id) => {
                        owned_vaults.push(vault_id);
                    }
                    RENodeId::KeyValueStore(kv_store_id) => {
                        owned_kv_stores.push(kv_store_id);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok((owned_kv_stores, owned_vaults))
}

fn dump_resources<T: ReadableSubstateStore, O: std::io::Write>(
    vaults: &HashSet<VaultId>,
    substate_store: &T,
    output: &mut O,
) -> Result<(), DisplayError> {
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());

    writeln!(output, "{}:", "Resources".green().bold());
    for (last, vault_id) in vaults.iter().identify_last() {
        let vault: VaultSubstate = substate_store
            .get_substate(&SubstateId(
                RENodeId::Vault(*vault_id),
                NodeModuleId::SELF,
                SubstateOffset::Vault(VaultOffset::Vault),
            ))
            .map(|s| s.substate)
            .map(|s| s.into())
            .unwrap();
        let amount = vault.0.amount();
        let resource_address = vault.0.resource_address();
        let global: Option<GlobalSubstate> = substate_store
            .get_substate(&SubstateId(
                RENodeId::Global(Address::Resource(resource_address)),
                NodeModuleId::SELF,
                SubstateOffset::Global(GlobalOffset::Global),
            ))
            .map(|s| s.substate)
            .map(|s| s.to_runtime().into());
        let resource_manager: Option<ResourceManagerSubstate> =
            global.as_ref().and_then(|global| {
                substate_store
                    .get_substate(&SubstateId(
                        global.node_deref(),
                        NodeModuleId::SELF,
                        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
                    ))
                    .map(|s| s.substate)
                    .map(|s| s.to_runtime().into())
            });
        let resource_manager = resource_manager.ok_or(DisplayError::ResourceManagerNotFound)?;
        let metadata: Option<MetadataSubstate> = global.and_then(|global| {
            substate_store
                .get_substate(&SubstateId(
                    global.node_deref(),
                    NodeModuleId::Metadata,
                    SubstateOffset::Metadata(MetadataOffset::Metadata),
                ))
                .map(|s| s.substate)
                .map(|s| s.to_runtime().into())
        });
        let metadata = metadata.ok_or(DisplayError::ResourceManagerNotFound)?;
        writeln!(
            output,
            "{} {{ amount: {}, resource address: {}{}{} }}",
            list_item_prefix(last),
            amount,
            resource_address.display(&bech32_encoder),
            metadata
                .metadata
                .get("name")
                .map(|name| format!(", name: \"{}\"", name))
                .unwrap_or(String::new()),
            metadata
                .metadata
                .get("symbol")
                .map(|symbol| format!(", symbol: \"{}\"", symbol))
                .unwrap_or(String::new()),
        );
        if matches!(
            resource_manager.resource_type,
            ResourceType::NonFungible { .. }
        ) {
            let ids = vault.0.ids();
            for (inner_last, id) in ids.iter().identify_last() {
                let non_fungible: NonFungibleSubstate = substate_store
                    .get_substate(&SubstateId(
                        RENodeId::NonFungibleStore(resource_manager.nf_store_id.unwrap()),
                        NodeModuleId::SELF,
                        SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(id.clone())),
                    ))
                    .map(|s| s.substate.to_runtime())
                    .map(|s| s.into())
                    .unwrap();
                if let Some(non_fungible) = non_fungible.0 {
                    let id = IndexedScryptoValue::from_typed(id);
                    let immutable_data =
                        IndexedScryptoValue::from_slice(&non_fungible.immutable_data()).unwrap();
                    let mutable_data =
                        IndexedScryptoValue::from_slice(&non_fungible.mutable_data()).unwrap();
                    let value_display_context =
                        ScryptoValueDisplayContext::with_optional_bench32(Some(&bech32_encoder));
                    writeln!(
                        output,
                        "{}  {} NonFungible {{ id: {}, immutable_data: {}, mutable_data: {} }}",
                        if last { " " } else { "│" },
                        list_item_prefix(inner_last),
                        id.display(value_display_context),
                        immutable_data.display(value_display_context),
                        mutable_data.display(value_display_context)
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
    let global: Option<GlobalSubstate> = substate_store
        .get_substate(&SubstateId(
            RENodeId::Global(Address::Resource(resource_address)),
            NodeModuleId::SELF,
            SubstateOffset::Global(GlobalOffset::Global),
        ))
        .map(|s| s.substate)
        .map(|s| s.to_runtime().into());
    let resource_manager: Option<ResourceManagerSubstate> = global.as_ref().and_then(|global| {
        substate_store
            .get_substate(&SubstateId(
                global.node_deref(),
                NodeModuleId::SELF,
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            ))
            .map(|s| s.substate)
            .map(|s| s.to_runtime().into())
    });
    let resource_manager = resource_manager.ok_or(DisplayError::ResourceManagerNotFound)?;
    let metadata: Option<MetadataSubstate> = global.and_then(|global| {
        substate_store
            .get_substate(&SubstateId(
                global.node_deref(),
                NodeModuleId::Metadata,
                SubstateOffset::Metadata(MetadataOffset::Metadata),
            ))
            .map(|s| s.substate)
            .map(|s| s.to_runtime().into())
    });
    let metadata = metadata.ok_or(DisplayError::ResourceManagerNotFound)?;

    writeln!(
        output,
        "{}: {:?}",
        "Resource Type".green().bold(),
        resource_manager.resource_type
    );
    writeln!(
        output,
        "{}: {}",
        "Metadata".green().bold(),
        metadata.metadata.len()
    );
    for (last, e) in metadata.metadata.iter().identify_last() {
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
        resource_manager.total_supply
    );
    Ok(())
}
