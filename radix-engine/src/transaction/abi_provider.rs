use sbor::rust::borrow::ToOwned;
use sbor::rust::string::ToString;
use scrypto::abi;
use scrypto::engine::types::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;

pub fn export_abi<S: ReadableSubstateStore>(
    substate_store: &S,
    package_address: PackageAddress,
    blueprint_name: &str,
) -> Result<abi::Blueprint, RuntimeError> {
    let package: ValidatedPackage = substate_store
        .get_decoded_substate(&package_address)
        .map(|(package, _)| package)
        .ok_or(RuntimeError::PackageNotFound(package_address))?;

    let abi = package
        .blueprint_abi(blueprint_name)
        .ok_or(RuntimeError::BlueprintNotFound(
            package_address,
            blueprint_name.to_owned(),
        ))?;

    // Return ABI
    Ok(abi::Blueprint {
        package_address: package_address.to_string(),
        blueprint_name: blueprint_name.to_owned(),
        functions: abi.1.clone(),
        methods: abi.2.clone(),
    })
}

pub fn export_abi_by_component<S: ReadableSubstateStore>(
    substate_store: &S,
    component_address: ComponentAddress,
) -> Result<abi::Blueprint, RuntimeError> {
    let component: Component = substate_store
        .get_decoded_substate(&component_address)
        .map(|(component, _)| component)
        .ok_or(RuntimeError::ComponentNotFound(component_address))?;
    export_abi(
        substate_store,
        component.package_address(),
        component.blueprint_name(),
    )
}
