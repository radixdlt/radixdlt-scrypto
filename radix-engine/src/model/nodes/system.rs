use crate::engine::{AuthModule, HeapRENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{
    HardAuthRule, HardProofRule, HardResourceOrNonFungible, InvokeError, MethodAuthorization,
    SystemSubstate,
};
use crate::types::*;
use crate::wasm::*;
use scrypto::core::SystemCreateInput;

#[derive(Debug, TypeId, Encode, Decode)]
pub enum SystemError {
    InvalidRequestData(DecodeError),
    InvalidMethod,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct System {
    pub info: SystemSubstate,
}

impl System {
    pub fn auth(system_fn: &SystemFnIdentifier) -> Vec<MethodAuthorization> {
        match system_fn {
            SystemFnIdentifier::Create => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        NonFungibleAddress::new(SYSTEM_TOKEN, AuthModule::system_id()),
                    )),
                ))]
            }
            SystemFnIdentifier::SetEpoch => {
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
        system_fn: SystemFnIdentifier,
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
            SystemFnIdentifier::Create => {
                let _: SystemCreateInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(SystemError::InvalidRequestData(e)))?;

                let node_id = system_api
                    .node_create(HeapRENode::System(System {
                        info: SystemSubstate { epoch: 0 },
                    }))
                    .map_err(InvokeError::Downstream)?;

                let system_node_id = node_id.clone();

                system_api
                    .node_globalize(node_id)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&system_node_id))
            }
            _ => Err(InvokeError::Error(SystemError::InvalidMethod)),
        }
    }

    pub fn main<'s, Y, W, I, R>(
        component_address: ComponentAddress,
        system_fn: SystemFnIdentifier,
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
            SystemFnIdentifier::GetCurrentEpoch => {
                let _: SystemGetCurrentEpochInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(SystemError::InvalidRequestData(e)))?;
                let mut node_ref = system_api
                    .borrow_node(&RENodeId::System(component_address))
                    .map_err(InvokeError::Downstream)?;
                Ok(ScryptoValue::from_typed(&node_ref.system().info.epoch))
            }
            SystemFnIdentifier::SetEpoch => {
                let SystemSetEpochInput { epoch } = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(SystemError::InvalidRequestData(e)))?;
                let mut system_node_ref = system_api
                    .borrow_node_mut(&RENodeId::System(SYS_SYSTEM_COMPONENT))
                    .map_err(InvokeError::Downstream)?;
                system_node_ref.system_mut().info.epoch = epoch;
                Ok(ScryptoValue::from_typed(&()))
            }
            SystemFnIdentifier::GetTransactionHash => {
                let _: SystemGetTransactionHashInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(SystemError::InvalidRequestData(e)))?;
                Ok(ScryptoValue::from_typed(
                    &system_api
                        .read_transaction_hash()
                        .map_err(InvokeError::Downstream)?,
                ))
            }
            _ => Err(InvokeError::Error(SystemError::InvalidMethod)),
        }
    }
}
