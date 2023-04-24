#![allow(unused_must_use)]
use crate::utils::*;
use colored::*;
use radix_engine::blueprints::resource::*;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::types::*;
use radix_engine_interface::blueprints::package::PackageCodeSubstate;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_stores::interface::SubstateDatabase;
use radix_engine_stores::jmt_support::JmtKeyMapper;
use radix_engine_stores::query::ResourceAccounter;
use utils::ContextualDisplay;

/// Represents an error when displaying an entity.
#[derive(Debug, Clone)]
pub enum EntityDumpError {
    PackageNotFound,
    ComponentNotFound,
    ResourceManagerNotFound,
}

/// Dump a package into console.
pub fn dump_package<T: SubstateDatabase, O: std::io::Write>(
    package_address: PackageAddress,
    substate_db: &T,
    output: &mut O,
) -> Result<(), EntityDumpError> {
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());
    let substate = substate_db
        .read_mapped_substate::<JmtKeyMapper, PackageCodeSubstate>(
            package_address.as_node_id(),
            SysModuleId::Object.into(),
            PackageOffset::Code.into(),
        )
        .ok_or(EntityDumpError::PackageNotFound)?;

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
        substate.code.len()
    );
    Ok(())
}

/// Dump a component into console.
pub fn dump_component<T: SubstateDatabase, O: std::io::Write>(
    component_address: ComponentAddress,
    substate_db: &T,
    output: &mut O,
) -> Result<(), EntityDumpError> {
    let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());

    let (package_address, blueprint_name, resources) = {
        let type_info = substate_db
            .read_mapped_substate::<JmtKeyMapper, TypeInfoSubstate>(
                component_address.as_node_id(),
                SysModuleId::TypeInfo.into(),
                TypeInfoOffset::TypeInfo.into(),
            )
            .ok_or(EntityDumpError::ComponentNotFound)?;
        let blueprint = match type_info {
            TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => blueprint,
            TypeInfoSubstate::KeyValueStore(_)
            | TypeInfoSubstate::Index
            | TypeInfoSubstate::SortedIndex => {
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
        component_address.display(&bech32_encoder),
    );

    writeln!(
        output,
        "{}: {{ package_address: {}, blueprint_name: \"{}\" }}",
        "Blueprint".green().bold(),
        package_address.display(&bech32_encoder),
        blueprint_name
    );

    writeln!(output, "{}", "Fungible Resources".green().bold());
    for (last, (component_address, amount)) in resources.balances.iter().identify_last() {
        writeln!(
            output,
            "{} {}: {}",
            list_item_prefix(last),
            component_address.display(&bech32_encoder),
            amount
        );
    }

    writeln!(output, "{}", "Non-fungibles Resources".green().bold());
    for (last, (component_address, ids)) in resources.non_fungibles.iter().identify_last() {
        writeln!(
            output,
            "{} {}",
            list_item_prefix(last),
            component_address.display(&bech32_encoder)
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
    if resource_address.as_node_id().entity_type() == Some(EntityType::GlobalNonFungibleResource) {
        let resource_manager = substate_db
            .read_mapped_substate::<JmtKeyMapper, NonFungibleResourceManagerSubstate>(
                resource_address.as_node_id(),
                SysModuleId::Object.into(),
                ResourceManagerOffset::ResourceManager.into(),
            )
            .ok_or(EntityDumpError::ResourceManagerNotFound)?;
        writeln!(
            output,
            "{}: {}",
            "Resource Type".green().bold(),
            "Non-fungible"
        );
        writeln!(
            output,
            "{}: {:?}",
            "ID Type".green().bold(),
            resource_manager.id_type
        );
        writeln!(
            output,
            "{}: {}",
            "Total Supply".green().bold(),
            resource_manager.total_supply
        );
    } else {
        let resource_manager = substate_db
            .read_mapped_substate::<JmtKeyMapper, FungibleResourceManagerSubstate>(
                resource_address.as_node_id(),
                SysModuleId::Object.into(),
                ResourceManagerOffset::ResourceManager.into(),
            )
            .ok_or(EntityDumpError::ResourceManagerNotFound)?;
        writeln!(output, "{}: {}", "Resource Type".green().bold(), "Fungible");
        writeln!(
            output,
            "{}: {:?}",
            "Divisibility".green().bold(),
            resource_manager.divisibility
        );
        writeln!(
            output,
            "{}: {}",
            "Total Supply".green().bold(),
            resource_manager.total_supply
        );
    }
    Ok(())
}
