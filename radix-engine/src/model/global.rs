use crate::engine::{InvokeError, SystemApi};
use crate::fee::FeeReserve;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance};


#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum GlobalRENode {
    Component(scrypto::component::Component),
    Package(PackageAddress),
    Resource(ResourceAddress),
}

impl GlobalRENode {
    pub fn node_id(&self) -> RENodeId {
        match self {
            GlobalRENode::Package(package_address) => RENodeId::Package(*package_address),
            GlobalRENode::Component(component) => RENodeId::Component(component.0),
            GlobalRENode::Resource(resource_address) => RENodeId::ResourceManager(*resource_address),
        }
    }
}

pub enum GlobalRENodeError {
}

pub fn main<'s, Y, W, I, R>(
    address: GlobalAddress,
    fn_identifier: FnIdentifier,
    args: ScryptoValue,
    system_api: &mut Y,
) -> Result<ScryptoValue, InvokeError<GlobalRENodeError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
{
    let mut substate_ref = system_api.substate_borrow_mut(&SubstateId::Global(address))
        .map_err(InvokeError::downstream)?;
    let global_re_node =  substate_ref.global_re_node();
    let node_id = global_re_node.node_id();

    let rtn = system_api.invoke_method(Receiver::Ref(node_id), fn_identifier, args)
        .map_err(InvokeError::downstream)?;
    system_api.substate_return_mut(substate_ref).map_err(InvokeError::downstream)?;
    Ok(rtn)
}
