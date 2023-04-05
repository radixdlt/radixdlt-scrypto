#![allow(unused_must_use)]
use crate::utils::*;
use colored::*;
use radix_engine::blueprints::resource::*;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::types::*;
use radix_engine_interface::blueprints::package::PackageCodeSubstate;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_stores::interface::SubstateDatabase;
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
    let package = substate_db
        .get_substate(
            package_address.as_node_id(),
            TypedModuleId::ObjectState.into(),
            &PackageOffset::Code.into(),
        )
        .expect("Database misconfigured");
    let substate = package.ok_or(EntityDumpError::PackageNotFound)?;

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
        scrypto_decode::<PackageCodeSubstate>(&substate.0)
            .unwrap()
            .code
            .len()
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
        let substate = substate_db
            .get_substate(
                component_address.as_node_id(),
                TypedModuleId::TypeInfo.into(),
                &TypeInfoOffset::TypeInfo.into(),
            )
            .expect("Database misconfigured")
            .ok_or(EntityDumpError::ComponentNotFound)?;
        let type_info: TypeInfoSubstate = scrypto_decode(&substate.0).unwrap();
        let blueprint = match type_info {
            TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => blueprint,
            TypeInfoSubstate::KeyValueStore(_) => panic!("Unexpected"),
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
    for (last, (component_address, amount)) in resources.fungibles.iter().identify_last() {
        writeln!(
            output,
            "{} {}: {}",
            list_item_prefix(last),
            component_address.display(&bech32_encoder),
            amount
        );
    }

    writeln!(output, "{}", "Non-fungible Resources".green().bold());
    for (last, (component_address, ids)) in resources.non_fungibles.iter().identify_last() {
        writeln!(
            output,
            "{} {}",
            list_item_prefix(last),
            component_address.display(&bech32_encoder),
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
    let substate = substate_db
        .get_substate(
            resource_address.as_node_id(),
            TypedModuleId::ObjectState.into(),
            &ResourceManagerOffset::ResourceManager.into(),
        )
        .expect("Database misconfigured")
        .ok_or(EntityDumpError::ResourceManagerNotFound)?;

    if resource_address.as_node_id().entity_type() == Some(EntityType::GlobalNonFungibleResource) {
        let resource_manager: NonFungibleResourceManagerSubstate =
            scrypto_decode(&substate.0).unwrap();
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
        let resource_manager: FungibleResourceManagerSubstate =
            scrypto_decode(&substate.0).unwrap();
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
