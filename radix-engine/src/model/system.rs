use crate::engine::SystemApi;
use crate::fee::FeeReserve;
use crate::model::{
    HardAuthRule, HardProofRule, HardResourceOrNonFungible, InvokeError, MethodAuthorization,
};
use crate::types::*;
use crate::wasm::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub enum SystemError {
    InvalidRequestData(DecodeError),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct System {
    pub epoch: u64,
}

impl System {
    pub fn auth(system_fn: &SystemFnIdentifier) -> Vec<MethodAuthorization> {
        match system_fn {
            SystemFnIdentifier::SetEpoch => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        NonFungibleAddress::new(SYSTEM_TOKEN, NonFungibleId::from_u32(0)),
                    )),
                ))]
            }
            _ => vec![],
        }
    }

    pub fn main<'s, Y, W, I, R>(
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
                let node_ref = system_api
                    .borrow_node(&RENodeId::System)
                    .map_err(InvokeError::Downstream)?;
                Ok(ScryptoValue::from_typed(&node_ref.system().epoch))
            }
            SystemFnIdentifier::SetEpoch => {
                let SystemSetEpochInput { epoch } = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(SystemError::InvalidRequestData(e)))?;
                let mut system_node_ref = system_api
                    .substate_borrow_mut(&SubstateId::System)
                    .map_err(InvokeError::Downstream)?;
                system_node_ref.system().epoch = epoch;
                system_api
                    .substate_return_mut(system_node_ref)
                    .map_err(InvokeError::Downstream)?;
                Ok(ScryptoValue::from_typed(&()))
            }
            SystemFnIdentifier::GetTransactionHash => {
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
