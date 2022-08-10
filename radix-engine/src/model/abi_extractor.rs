use sbor::rust::borrow::ToOwned;
use scrypto::abi;
use scrypto::engine::types::*;

use crate::engine::*;
use crate::ledger::*;

pub fn export_abi<S: ReadableSubstateStore>(
    substate_store: &S,
    package_address: PackageAddress,
    blueprint_name: &str,
) -> Result<abi::BlueprintAbi, RuntimeError> {
    let package_value: Substate = substate_store
        .get_substate(&SubstateId::Package(package_address))
        .map(|s| s.substate)
        .ok_or(RuntimeError::PackageNotFound(package_address))?;

    let abi = package_value
        .package()
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
    let component_value: Substate = substate_store
        .get_substate(&SubstateId::ComponentInfo(component_address))
        .map(|s| s.substate)
        .ok_or(RuntimeError::RENodeNotFound(RENodeId::Component(
            component_address,
        )))?;
    let component = component_value.component();
    export_abi(
        substate_store,
        component.package_address(),
        component.blueprint_name(),
    )
}
