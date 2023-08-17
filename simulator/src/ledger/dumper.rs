#![allow(unused_must_use)]
use crate::utils::*;
use colored::*;
use radix_engine::blueprints::resource::*;
use radix_engine::system::system::KeyValueEntrySubstate;
use radix_engine::system::system_db_reader::SystemDatabaseReader;
use radix_engine::types::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_queries::query::ResourceAccounter;
use radix_engine_queries::typed_substate_layout::MetadataEntrySubstate;
use radix_engine_store_interface::{
    db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper},
    interface::SubstateDatabase,
};
use utils::ContextualDisplay;

/// Represents an error when displaying an entity.
#[derive(Debug, Clone)]
pub enum EntityDumpError {
    PackageNotFound,
    ComponentNotFound,
    ResourceManagerNotFound,
    InvalidStore(String),
}

/// Dump a package into console.
pub fn dump_package<T: SubstateDatabase, O: std::io::Write>(
    package_address: PackageAddress,
    substate_db: &T,
    output: &mut O,
) -> Result<(), EntityDumpError> {
    let address_bech32_encoder = AddressBech32Encoder::new(&NetworkDefinition::simulator());
    let (_, substate) = substate_db
        .list_mapped::<SpreadPrefixKeyMapper, KeyValueEntrySubstate<PackageOriginalCodeSubstate>, MapKey>(
            package_address.as_node_id(),
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_ORIGINAL_CODE_PARTITION_OFFSET)
                .unwrap(),
        )
        .next()
        .ok_or(EntityDumpError::PackageNotFound)?;

    writeln!(
        output,
        "{}: {}",
        "Package Address".green().bold(),
        package_address.display(&address_bech32_encoder)
    );
    writeln!(
        output,
        "{}: {} bytes",
        "Code size".green().bold(),
        substate.value.unwrap().code.len()
    );

    let metadata = get_entity_metadata(package_address.as_node_id(), substate_db);
    writeln!(output, "{}: {}", "Metadata".green().bold(), metadata.len());
    for (last, (key, value)) in metadata.iter().identify_last() {
        writeln!(output, "{} {}: {:?}", list_item_prefix(last), key, value);
    }

    Ok(())
}

/// Dump a component into console.
pub fn dump_component<T: SubstateDatabase, O: std::io::Write>(
    component_address: ComponentAddress,
    substate_db: &T,
    output: &mut O,
) -> Result<(), EntityDumpError> {
    let address_bech32_encoder = AddressBech32Encoder::new(&NetworkDefinition::simulator());

    let reader = SystemDatabaseReader::new(substate_db);

    let (package_address, blueprint_name, resources) = {
        let object_info = reader
            .get_object_info(component_address)
            .ok_or_else(|| EntityDumpError::ComponentNotFound)?;
        let blueprint_id = object_info.blueprint_info.blueprint_id;

        let mut accounter = ResourceAccounter::new(substate_db);
        accounter.traverse(component_address.as_node_id().clone());
        let resources = accounter.close();

        (
            blueprint_id.package_address,
            blueprint_id.blueprint_name,
            resources,
        )
    };

    writeln!(
        output,
        "{}: {}",
        "Component Address".green().bold(),
        component_address.display(&address_bech32_encoder),
    );

    writeln!(
        output,
        "{}: {{ package_address: {}, blueprint_name: \"{}\" }}",
        "Blueprint ID".green().bold(),
        package_address.display(&address_bech32_encoder),
        blueprint_name
    );

    writeln!(
        output,
        "{}: {}",
        "Owned Fungible Resources".green().bold(),
        resources.balances.len()
    );
    for (last, (resource_address, amount)) in resources.balances.iter().identify_last() {
        let metadata = get_entity_metadata(resource_address.as_node_id(), substate_db);
        let symbol = if let Some(MetadataValue::String(symbol)) = metadata.get("symbol") {
            symbol.as_str()
        } else {
            "?"
        };
        writeln!(
            output,
            "{} {}: {} {}",
            list_item_prefix(last),
            resource_address.display(&address_bech32_encoder),
            amount,
            symbol,
        );
    }

    writeln!(
        output,
        "{}: {}",
        "Owned Non-fungibles Resources".green().bold(),
        resources.non_fungibles.len()
    );
    for (last, (resource_address, ids)) in resources.non_fungibles.iter().identify_last() {
        let metadata = get_entity_metadata(resource_address.as_node_id(), substate_db);
        let symbol = if let Some(MetadataValue::String(symbol)) = metadata.get("symbol") {
            symbol.as_str()
        } else {
            "?"
        };
        writeln!(
            output,
            "{} {}: {} {}",
            list_item_prefix(last),
            resource_address.display(&address_bech32_encoder),
            ids.len(),
            symbol,
        );
        for (last, id) in ids.iter().identify_last() {
            writeln!(output, "   {} {}", list_item_prefix(last), id);
        }
    }

    let metadata = get_entity_metadata(component_address.as_node_id(), substate_db);
    writeln!(output, "{}: {}", "Metadata".green().bold(), metadata.len());
    for (last, (key, value)) in metadata.iter().identify_last() {
        writeln!(output, "{} {}: {:?}", list_item_prefix(last), key, value);
    }

    Ok(())
}

/// Dump a resource into console.
pub fn dump_resource_manager<T: SubstateDatabase, O: std::io::Write>(
    resource_address: ResourceAddress,
    substate_db: &T,
    output: &mut O,
) -> Result<(), EntityDumpError> {
    let address_bech32_encoder = AddressBech32Encoder::new(&NetworkDefinition::simulator());

    writeln!(
        output,
        "{}: {}",
        "Resource Address".green().bold(),
        resource_address.display(&address_bech32_encoder)
    );

    let reader = SystemDatabaseReader::new(substate_db);
    let info = reader
        .get_object_info(resource_address)
        .ok_or_else(|| EntityDumpError::ResourceManagerNotFound)?;

    if info
        .blueprint_info
        .blueprint_id
        .blueprint_name
        .eq(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT)
    {
        let id_type: NonFungibleIdType = reader
            .read_object_field(
                resource_address.as_node_id(),
                ObjectModuleId::Main,
                NonFungibleResourceManagerField::IdType.into(),
            )
            .ok_or(EntityDumpError::InvalidStore(
                "Missing NonFungible IdType".to_string(),
            ))?;

        writeln!(
            output,
            "{}: {}",
            "Resource Type".green().bold(),
            "Non-fungible"
        );
        writeln!(output, "{}: {:?}", "ID Type".green().bold(), id_type);

        if info.get_features().contains(TRACK_TOTAL_SUPPLY_FEATURE) {
            let total_supply: Decimal = reader
                .read_object_field(
                    resource_address.as_node_id(),
                    ObjectModuleId::Main,
                    NonFungibleResourceManagerField::TotalSupply.into(),
                )
                .ok_or(EntityDumpError::InvalidStore(
                    "Missing Total Supply".to_string(),
                ))?;

            writeln!(
                output,
                "{}: {}",
                "Total Supply".green().bold(),
                total_supply
            );
        }
    } else {
        let divisibility: FungibleResourceManagerDivisibilitySubstate = reader
            .read_object_field(
                resource_address.as_node_id(),
                ObjectModuleId::Main,
                FungibleResourceManagerField::Divisibility.into(),
            )
            .ok_or(EntityDumpError::InvalidStore(
                "Missing Divisibility".to_string(),
            ))?;

        writeln!(output, "{}: {}", "Resource Type".green().bold(), "Fungible");
        writeln!(
            output,
            "{}: {:?}",
            "Divisibility".green().bold(),
            divisibility
        );

        if info.get_features().contains(TRACK_TOTAL_SUPPLY_FEATURE) {
            let total_supply: FungibleResourceManagerTotalSupplySubstate = reader
                .read_object_field(
                    resource_address.as_node_id(),
                    ObjectModuleId::Main,
                    FungibleResourceManagerField::TotalSupply.into(),
                )
                .ok_or(EntityDumpError::InvalidStore(
                    "Missing Total Supply".to_string(),
                ))?;

            writeln!(
                output,
                "{}: {}",
                "Total Supply".green().bold(),
                total_supply
            );
        }
    }

    let metadata = get_entity_metadata(resource_address.as_node_id(), substate_db);
    writeln!(output, "{}: {}", "Metadata".green().bold(), metadata.len());
    for (last, (key, value)) in metadata.iter().identify_last() {
        writeln!(output, "{} {}: {:?}", list_item_prefix(last), key, value);
    }

    Ok(())
}

fn get_entity_metadata<T: SubstateDatabase>(
    entity_node_id: &NodeId,
    substate_db: &T,
) -> IndexMap<String, MetadataValue> {
    let mut metadata = indexmap!();
    for (substate_key, substate_value) in substate_db
        .list_mapped::<SpreadPrefixKeyMapper, MetadataEntrySubstate, MapKey>(
            entity_node_id,
            METADATA_BASE_PARTITION
                .at_offset(METADATA_KV_STORE_PARTITION_OFFSET)
                .unwrap(),
        )
    {
        if let SubstateKey::Map(key) = substate_key {
            if let Some(value) = substate_value.value {
                let key = scrypto_decode::<String>(&key).unwrap();
                metadata.insert(key, value);
            }
        }
    }
    metadata
}
