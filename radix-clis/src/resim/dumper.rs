#![allow(unused_must_use)]
use crate::utils::*;
use colored::*;
use radix_blueprint_schema_init::BlueprintFeature;
use radix_common::network::NetworkDefinition;
use radix_common::prelude::*;
use radix_engine::blueprints::resource::*;
use radix_engine::object_modules::metadata::{MetadataCollection, MetadataEntryEntryPayload};
use radix_engine::system::system_db_reader::SystemDatabaseReader;
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::resource::NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT;
use radix_engine_interface::types::{BlueprintPartitionOffset, CollectionDescriptor};
use radix_engine_interface::{prelude::MetadataValue, types::PackagePartitionOffset};
use radix_rust::ContextualDisplay;
use radix_substate_store_interface::interface::*;
use radix_substate_store_queries::query::ResourceAccounter;
use radix_substate_store_queries::typed_substate_layout::*;

/// Represents an error when displaying an entity.
#[derive(Debug, Clone)]
pub enum EntityDumpError {
    PackageNotFound,
    ComponentNotFound,
    ResourceManagerNotFound,
    InvalidStore(String),
    /// If you run `resim show`, without any address but no default account address has been set.
    NoAddressProvidedAndNotDefaultAccountSet,
}

/// Dump a package into console.
pub fn dump_package<T: SubstateDatabase, O: std::io::Write>(
    package_address: PackageAddress,
    substate_db: &T,
    output: &mut O,
) -> Result<(), EntityDumpError> {
    let address_bech32_encoder = AddressBech32Encoder::new(&NetworkDefinition::simulator());
    let (_, substate) = substate_db
        .list_map_values::<PackageCodeOriginalCodeEntrySubstate>(
            package_address,
            PackagePartitionOffset::CodeOriginalCodeKeyValue.as_main_partition(),
            None::<SubstateKey>,
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
        substate
            .into_value()
            .unwrap()
            .fully_update_and_into_latest_version()
            .code
            .len()
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
            .map_err(|_| EntityDumpError::ComponentNotFound)?;
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
        let name = if let Some(MetadataValue::String(name)) = metadata.get("name") {
            name.as_str()
        } else {
            "?"
        };
        let symbol_text = if let Some(MetadataValue::String(symbol)) = metadata.get("symbol") {
            format!(" ({})", symbol)
        } else {
            "".to_string()
        };
        writeln!(
            output,
            "{} {}: {} {}{}",
            list_item_prefix(last),
            resource_address.display(&address_bech32_encoder),
            amount,
            name,
            symbol_text,
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
        let name = if let Some(MetadataValue::String(name)) = metadata.get("name") {
            name.as_str()
        } else {
            "?"
        };
        let symbol_text = if let Some(MetadataValue::String(symbol)) = metadata.get("symbol") {
            format!(" ({})", symbol)
        } else {
            "".to_string()
        };
        writeln!(
            output,
            "{} {}: {} {}{}",
            list_item_prefix(last),
            resource_address.display(&address_bech32_encoder),
            ids.len(),
            name,
            symbol_text,
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
        .map_err(|_| EntityDumpError::ResourceManagerNotFound)?;

    if info
        .blueprint_info
        .blueprint_id
        .blueprint_name
        .eq(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT)
    {
        let id_type: VersionedNonFungibleResourceManagerIdType = reader
            .read_typed_object_field(
                resource_address.as_node_id(),
                ModuleId::Main,
                NonFungibleResourceManagerField::IdType.into(),
            )
            .map_err(|_| EntityDumpError::InvalidStore("Missing NonFungible IdType".to_string()))?;

        writeln!(
            output,
            "{}: {}",
            "Resource Type".green().bold(),
            "Non-fungible"
        );
        writeln!(output, "{}: {:?}", "ID Type".green().bold(), id_type);

        if info
            .get_features()
            .contains(NonFungibleResourceManagerFeature::TrackTotalSupply.feature_name())
        {
            let total_supply = reader
                .read_typed_object_field::<NonFungibleResourceManagerTotalSupplyFieldPayload>(
                    resource_address.as_node_id(),
                    ModuleId::Main,
                    NonFungibleResourceManagerField::TotalSupply.into(),
                )
                .map_err(|_| EntityDumpError::InvalidStore("Missing Total Supply".to_string()))?
                .fully_update_and_into_latest_version();

            writeln!(
                output,
                "{}: {}",
                "Total Supply".green().bold(),
                total_supply
            );
        }
    } else {
        let divisibility = reader
            .read_typed_object_field::<FungibleResourceManagerDivisibilityFieldPayload>(
                resource_address.as_node_id(),
                ModuleId::Main,
                FungibleResourceManagerField::Divisibility.into(),
            )
            .map_err(|_| EntityDumpError::InvalidStore("Missing Divisibility".to_string()))?
            .fully_update_and_into_latest_version();

        writeln!(output, "{}: {}", "Resource Type".green().bold(), "Fungible");
        writeln!(
            output,
            "{}: {:?}",
            "Divisibility".green().bold(),
            divisibility
        );

        if info
            .get_features()
            .contains(FungibleResourceManagerFeature::TrackTotalSupply.feature_name())
        {
            let total_supply = reader
                .read_typed_object_field::<FungibleResourceManagerTotalSupplyFieldPayload>(
                    resource_address.as_node_id(),
                    ModuleId::Main,
                    FungibleResourceManagerField::TotalSupply.into(),
                )
                .map_err(|_| EntityDumpError::InvalidStore("Missing Total Supply".to_string()))?
                .fully_update_and_into_latest_version();

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
    let reader = SystemDatabaseReader::new(substate_db);
    reader
        .collection_iter(
            entity_node_id,
            ModuleId::Metadata,
            MetadataCollection::EntryKeyValue.collection_index(),
        )
        .unwrap()
        .map(|(key, value)| {
            let map_key = key.into_map();
            let key = scrypto_decode::<String>(&map_key).unwrap();
            let value = scrypto_decode::<MetadataEntryEntryPayload>(&value).unwrap();
            (key, value.fully_update_and_into_latest_version())
        })
        .collect()
}
