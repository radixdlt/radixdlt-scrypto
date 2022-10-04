use scrypto::abi;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;
use crate::types::*;

pub fn export_abi<S: ReadableSubstateStore>(
    substate_store: &S,
    package_address: PackageAddress,
    blueprint_name: &str,
) -> Result<abi::BlueprintAbi, RuntimeError> {
    let package_value: Substate = substate_store
        .get_substate(&SubstateId(
            RENodeId::Package(package_address),
            SubstateOffset::Package(PackageOffset::Package),
        ))
        .map(|s| s.substate)
        .ok_or(RuntimeError::KernelError(KernelError::PackageNotFound(
            package_address,
        )))?;

    let abi = package_value
        .package()
        .blueprint_abis
        .get(blueprint_name)
        .ok_or(RuntimeError::KernelError(KernelError::BlueprintNotFound(
            package_address,
            blueprint_name.to_owned(),
        )))?
        .clone();
    Ok(abi)
}

pub fn export_abi_by_component<S: ReadableSubstateStore>(
    substate_store: &S,
    component_address: ComponentAddress,
) -> Result<abi::BlueprintAbi, RuntimeError> {
    let component_value: Substate = substate_store
        .get_substate(&SubstateId(
            RENodeId::Component(component_address),
            SubstateOffset::Component(ComponentOffset::Info),
        ))
        .map(|s| s.substate)
        .ok_or(RuntimeError::KernelError(KernelError::RENodeNotFound(
            RENodeId::Component(component_address),
        )))?;
    let component_info = component_value.component_info();
    export_abi(
        substate_store,
        component_info.package_address,
        &component_info.blueprint_name,
    )
}
