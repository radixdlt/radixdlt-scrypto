#![allow(unused_must_use)]
use crate::utils::*;
use colored::*;
use radix_engine::blueprints::resource::*;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::system::system::KeyValueEntrySubstate;
use radix_engine::types::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_queries::query::ResourceAccounter;
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
        "Package".green().bold(),
        package_address.display(&address_bech32_encoder)
    );
    writeln!(
        output,
        "{}: {} bytes",
        "Code size".green().bold(),
        substate.value.unwrap().code.len()
    );
    Ok(())
}

/// Dump a component into console.
pub fn dump_component<T: SubstateDatabase, O: std::io::Write>(
    component_address: ComponentAddress,
    substate_db: &T,
    output: &mut O,
) -> Result<(), EntityDumpError> {
    let address_bech32_encoder = AddressBech32Encoder::new(&NetworkDefinition::simulator());

    let (package_address, blueprint_name, resources) = {
        let type_info = substate_db
            .get_mapped::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
                component_address.as_node_id(),
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
            )
            .ok_or(EntityDumpError::ComponentNotFound)?;
        let blueprint = match type_info {
            TypeInfoSubstate::Object(ObjectInfo {
                blueprint_id: blueprint,
                ..
            }) => blueprint,
            _ => {
                panic!("Unexpected")
            }
        };

        let mut accounter = ResourceAccounter::new(substate_db);
        accounter.traverse(component_address.as_node_id().clone());
        let resources = accounter.close();

        (
            blueprint.package_address,
            blueprint.blueprint_name,
            resources,
        )
    };

    writeln!(
        output,
        "{}: {}",
        "Component".green().bold(),
        component_address.display(&address_bech32_encoder),
    );

    writeln!(
        output,
        "{}: {{ package_address: {}, blueprint_name: \"{}\" }}",
        "Blueprint".green().bold(),
        package_address.display(&address_bech32_encoder),
        blueprint_name
    );

    writeln!(output, "{}", "Fungible Resources".green().bold());
    for (last, (component_address, amount)) in resources.balances.iter().identify_last() {
        writeln!(
            output,
            "{} {}: {}",
            list_item_prefix(last),
            component_address.display(&address_bech32_encoder),
            amount
        );
    }

    writeln!(output, "{}", "Non-fungibles Resources".green().bold());
    for (last, (component_address, ids)) in resources.non_fungibles.iter().identify_last() {
        writeln!(
            output,
            "{} {}",
            list_item_prefix(last),
            component_address.display(&address_bech32_encoder)
        );
        for (last, id) in ids.iter().identify_last() {
            writeln!(output, "   {} {}", list_item_prefix(last), id);
        }
    }

    Ok(())
}

/// Dump a resource into console.
pub fn dump_resource_manager<T: SubstateDatabase, O: std::io::Write>(
    resource_address: ResourceAddress,
    substate_db: &T,
    output: &mut O,
) -> Result<(), EntityDumpError> {
    let type_info = substate_db
        .get_mapped::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
            resource_address.as_node_id(),
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        )
        .ok_or(EntityDumpError::ResourceManagerNotFound)?;

    let info = match type_info {
        TypeInfoSubstate::Object(info)
            if info.blueprint_id.package_address.eq(&RESOURCE_PACKAGE) =>
        {
            info
        }
        _ => {
            return Err(EntityDumpError::InvalidStore(
                "Expected Resource Manager".to_string(),
            ))
        }
    };

    if info
        .blueprint_id
        .blueprint_name
        .eq(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT)
    {
        let id_type = substate_db
            .get_mapped::<SpreadPrefixKeyMapper, NonFungibleIdType>(
                resource_address.as_node_id(),
                MAIN_BASE_PARTITION,
                &NonFungibleResourceManagerField::IdType.into(),
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
            let total_supply = substate_db
                .get_mapped::<SpreadPrefixKeyMapper, Decimal>(
                    resource_address.as_node_id(),
                    MAIN_BASE_PARTITION,
                    &NonFungibleResourceManagerField::TotalSupply.into(),
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
        let divisibility = substate_db
            .get_mapped::<SpreadPrefixKeyMapper, FungibleResourceManagerDivisibilitySubstate>(
                resource_address.as_node_id(),
                MAIN_BASE_PARTITION,
                &FungibleResourceManagerField::Divisibility.into(),
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
            let total_supply = substate_db
                .get_mapped::<SpreadPrefixKeyMapper, FungibleResourceManagerTotalSupplySubstate>(
                    resource_address.as_node_id(),
                    MAIN_BASE_PARTITION,
                    &FungibleResourceManagerField::TotalSupply.into(),
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
    Ok(())
}
