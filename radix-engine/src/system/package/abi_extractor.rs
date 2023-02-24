use crate::ledger::*;
use crate::system::node_substates::RuntimeSubstate;
use crate::types::*;
use radix_engine_interface::abi;
use radix_engine_interface::api::types::{
    PackageOffset, RENodeId, SubstateId, SubstateOffset,
};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ExportError {
    ComponentNotFound(ComponentAddress),
    PackageNotFound(PackageAddress),
    BlueprintNotFound(PackageAddress, String),
}

pub fn export_abi<S: ReadableSubstateStore>(
    substate_store: &S,
    package_address: PackageAddress,
    blueprint_name: &str,
) -> Result<abi::BlueprintAbi, ExportError> {
    let package_value: RuntimeSubstate = substate_store
        .get_substate(&SubstateId(
            RENodeId::GlobalPackage(package_address),
            NodeModuleId::SELF,
            SubstateOffset::Package(PackageOffset::Info),
        ))
        .map(|s| s.substate.to_runtime())
        .ok_or(ExportError::PackageNotFound(package_address))?;

    let abi = package_value
        .package_info()
        .blueprint_abis
        .get(blueprint_name)
        .ok_or(ExportError::BlueprintNotFound(
            package_address,
            blueprint_name.to_owned(),
        ))?
        .clone();
    Ok(abi)
}

pub fn export_abi_by_component<S: ReadableSubstateStore>(
    substate_store: &S,
    component_address: ComponentAddress,
) -> Result<abi::BlueprintAbi, ExportError> {
    let component_value: RuntimeSubstate = substate_store
        .get_substate(&SubstateId(
            RENodeId::GlobalComponent(component_address),
            NodeModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .map(|s| s.substate.to_runtime())
        .ok_or(ExportError::ComponentNotFound(component_address))?;

    let component_ref = component_value.to_ref();
    let component_info = component_ref.component_info();
    export_abi(
        substate_store,
        component_info.package_address,
        &component_info.blueprint_name,
    )
}
