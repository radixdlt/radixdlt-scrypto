use radix_engine_interface::abi;
use radix_engine_interface::api::types::{
    ComponentOffset, GlobalAddress, GlobalOffset, PackageOffset, RENodeId, SubstateId,
    SubstateOffset,
};

use crate::ledger::*;
use crate::model::*;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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
    let global_substate: RuntimeSubstate = substate_store
        .get_substate(&SubstateId(
            RENodeId::Global(GlobalAddress::Package(package_address)),
            SubstateOffset::Global(GlobalOffset::Global),
        ))
        .map(|s| s.substate.to_runtime())
        .ok_or(ExportError::PackageNotFound(package_address))?;

    let package_value: RuntimeSubstate = substate_store
        .get_substate(&SubstateId(
            global_substate.global().node_deref(),
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
    let node_id = RENodeId::Global(GlobalAddress::Component(component_address));
    let global = substate_store
        .get_substate(&SubstateId(
            node_id,
            SubstateOffset::Global(GlobalOffset::Global),
        ))
        .map(|s| s.substate.to_runtime())
        .ok_or(ExportError::ComponentNotFound(component_address))?;
    let component_id = global.global().node_deref();

    let component_value: RuntimeSubstate = substate_store
        .get_substate(&SubstateId(
            component_id,
            SubstateOffset::Component(ComponentOffset::Info),
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
