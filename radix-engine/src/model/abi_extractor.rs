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
    let package_value: RuntimeSubstate = substate_store
        .get_substate(&SubstateId(
            RENodeId::Package(package_address),
            SubstateOffset::Package(PackageOffset::Package),
        ))
        .map(|s| s.substate.to_runtime())
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
    let node_id = RENodeId::Global(GlobalAddress::Component(component_address));
    let global = substate_store
        .get_substate(&SubstateId(
            node_id,
            SubstateOffset::Global(GlobalOffset::Global),
        ))
        .map(|s| s.substate.to_runtime())
        .ok_or(RuntimeError::KernelError(KernelError::RENodeNotFound(
            node_id,
        )))?;
    let component_id = global.global_re_node().node_deref();

    let component_value: RuntimeSubstate = substate_store
        .get_substate(&SubstateId(
            component_id,
            SubstateOffset::Component(ComponentOffset::Info),
        ))
        .map(|s| s.substate.to_runtime())
        .ok_or(RuntimeError::KernelError(KernelError::RENodeNotFound(
            component_id,
        )))?;

    let component_ref = component_value.to_ref();
    let component_info = component_ref.component_info();
    export_abi(
        substate_store,
        component_info.package_address,
        &component_info.blueprint_name,
    )
}
