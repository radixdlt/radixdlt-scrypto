use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::{
    SystemFnIdentifier, SystemGetCurrentEpochInput, SystemGetTransactionHashInput,
    SystemSetEpochInput,
};
use scrypto::engine::types::{RENodeId, SubstateId};
use scrypto::values::ScryptoValue;

use crate::engine::SystemApi;
use crate::fee::FeeReserve;
use crate::fee::FeeReserveError;
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SystemError {
    InvalidRequestData(DecodeError),
    CostingError(FeeReserveError),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct System {
    pub epoch: u64,
}

impl System {
    pub fn main<'s, Y: SystemApi<'s, W, I, C>, W: WasmEngine<I>, I: WasmInstance, C: FeeReserve>(
        system_fn: SystemFnIdentifier,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, SystemError> {
        match system_fn {
            SystemFnIdentifier::GetCurrentEpoch => {
                let _: SystemGetCurrentEpochInput =
                    scrypto_decode(&args.raw).map_err(|e| SystemError::InvalidRequestData(e))?;
                let node_ref = system_api
                    .borrow_node(&RENodeId::System)
                    .map_err(SystemError::CostingError)?;
                Ok(ScryptoValue::from_typed(&node_ref.system().epoch))
            }
            SystemFnIdentifier::SetEpoch => {
                let SystemSetEpochInput { epoch } =
                    scrypto_decode(&args.raw).map_err(|e| SystemError::InvalidRequestData(e))?;
                let mut system_node_ref = system_api
                    .substate_borrow_mut(&SubstateId::System)
                    .map_err(SystemError::CostingError)?;
                system_node_ref.system().epoch = epoch;
                system_api
                    .substate_return_mut(system_node_ref)
                    .map_err(SystemError::CostingError)?;
                Ok(ScryptoValue::from_typed(&()))
            }
            SystemFnIdentifier::GetTransactionHash => {
                let _: SystemGetTransactionHashInput =
                    scrypto_decode(&args.raw).map_err(|e| SystemError::InvalidRequestData(e))?;
                Ok(ScryptoValue::from_typed(
                    &system_api
                        .transaction_hash()
                        .map_err(SystemError::CostingError)?,
                ))
            }
        }
    }
}
