use crate::engine::{AuthModule, HeapRENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{
    HardAuthRule, HardProofRule, HardResourceOrNonFungible, InvokeError, MethodAuthorization,
};
use crate::types::*;
use crate::wasm::*;
use scrypto::core::{SystemCreateInput, SystemFunctionFnIdent};

#[derive(Debug, TypeId, Encode, Decode)]
pub enum SystemError {
    InvalidRequestData(DecodeError),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct System {
    pub epoch: u64,
}

impl System {
    pub fn function_auth(system_fn: &SystemFunctionFnIdent) -> Vec<MethodAuthorization> {
        match system_fn {
            SystemFunctionFnIdent::Create => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        NonFungibleAddress::new(SYSTEM_TOKEN, AuthModule::system_id()),
                    )),
                ))]
            }
        }
    }

    pub fn method_auth(system_fn: &SystemMethodFnIdent) -> Vec<MethodAuthorization> {
        match system_fn {
            SystemMethodFnIdent::SetEpoch => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        NonFungibleAddress::new(SYSTEM_TOKEN, AuthModule::supervisor_id()),
                    )),
                ))]
            }
            _ => vec![],
        }
    }

    pub fn static_main<'s, Y, W, I, R>(
        system_fn: SystemFunctionFnIdent,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<SystemError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match system_fn {
            SystemFunctionFnIdent::Create => {
                let _: SystemCreateInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(SystemError::InvalidRequestData(e)))?;

                let node_id = system_api
                    .node_create(HeapRENode::System(System { epoch: 0 }))
                    .map_err(InvokeError::Downstream)?;

                let system_node_id = node_id.clone();

                system_api
                    .node_globalize(node_id)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&system_node_id))
            }
        }
    }

    pub fn main<'s, Y, W, I, R>(
        component_address: ComponentAddress,
        system_fn: SystemMethodFnIdent,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<SystemError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match system_fn {
            SystemMethodFnIdent::GetCurrentEpoch => {
                let _: SystemGetCurrentEpochInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(SystemError::InvalidRequestData(e)))?;
                let node_ref = system_api
                    .borrow_node(&RENodeId::System(component_address))
                    .map_err(InvokeError::Downstream)?;
                Ok(ScryptoValue::from_typed(&node_ref.system().epoch))
            }
            SystemMethodFnIdent::SetEpoch => {
                let SystemSetEpochInput { epoch } = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(SystemError::InvalidRequestData(e)))?;
                let mut system_node_ref = system_api
                    .substate_borrow_mut(&SubstateId::System(component_address))
                    .map_err(InvokeError::Downstream)?;
                system_node_ref.system_mut().epoch = epoch;
                system_api
                    .substate_return_mut(system_node_ref)
                    .map_err(InvokeError::Downstream)?;
                Ok(ScryptoValue::from_typed(&()))
            }
            SystemMethodFnIdent::GetTransactionHash => {
                let _: SystemGetTransactionHashInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(SystemError::InvalidRequestData(e)))?;
                Ok(ScryptoValue::from_typed(
                    &system_api
                        .transaction_hash()
                        .map_err(InvokeError::Downstream)?,
                ))
            }
        }
    }
}
