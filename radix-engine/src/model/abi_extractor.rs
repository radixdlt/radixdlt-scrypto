use sbor::rust::borrow::ToOwned;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::engine::types::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;

pub fn export_abi<S: ReadableSubstateStore>(
    substate_store: &S,
    package_address: PackageAddress,
    blueprint_name: &str,
) -> Result<abi::BlueprintAbi, RuntimeError> {
    let package: ValidatedPackage = substate_store
        .get_substate(&Address::Package(package_address))
        .map(|s| scrypto_decode(&s).unwrap())
        .ok_or(RuntimeError::PackageNotFound(package_address))?;

    let abi = package
        .blueprint_abi(blueprint_name)
        .ok_or(RuntimeError::BlueprintNotFound(
            package_address,
            blueprint_name.to_owned(),
        ))?
        .clone();
    Ok(abi)
}

pub fn export_abi_by_component<S: ReadableSubstateStore>(
    substate_store: &S,
    component_address: ComponentAddress,
) -> Result<abi::BlueprintAbi, RuntimeError> {
    let component: Component = substate_store
        .get_substate(&Address::GlobalComponent(component_address))
        .map(|s| scrypto_decode(&s).unwrap())
        .ok_or(RuntimeError::ComponentNotFound(component_address))?;
    export_abi(
        substate_store,
        component.package_address(),
        component.blueprint_name(),
    )
}
