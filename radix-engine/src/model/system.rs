use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::{
    SystemGetCurrentEpochInput, SystemGetTransactionHashInput, SystemSetEpochInput,
};
use scrypto::engine::types::RENodeId;
use scrypto::values::ScryptoValue;

use crate::engine::SystemApi;
use crate::fee::FeeReserve;
use crate::fee::FeeReserveError;
use crate::model::SystemError::InvalidMethod;
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SystemError {
    InvalidRequestData(DecodeError),
    InvalidMethod,
    CostingError(FeeReserveError),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct System {
    pub epoch: u64,
}

impl System {
    pub fn main<
        'p,
        's,
        Y: SystemApi<'p, 's, W, I, C>,
        W: WasmEngine<I>,
        I: WasmInstance,
        C: FeeReserve,
    >(
        method_name: &str,
        arg: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, SystemError> {
        match method_name {
            "get_epoch" => {
                let _: SystemGetCurrentEpochInput =
                    scrypto_decode(&arg.raw).map_err(|e| SystemError::InvalidRequestData(e))?;
                let node_ref = system_api
                    .borrow_node(&RENodeId::System)
                    .map_err(SystemError::CostingError)?;
                Ok(ScryptoValue::from_typed(&node_ref.system().epoch))
            }
            "set_epoch" => {
                let SystemSetEpochInput { epoch } =
                    scrypto_decode(&arg.raw).map_err(|e| SystemError::InvalidRequestData(e))?;
                let mut system_node_ref = system_api
                    .borrow_node_mut(&RENodeId::System)
                    .map_err(SystemError::CostingError)?;
                system_node_ref.system().epoch = epoch;
                system_api
                    .return_node_mut(system_node_ref)
                    .map_err(SystemError::CostingError)?;
                Ok(ScryptoValue::from_typed(&()))
            }
            "transaction_hash" => {
                let _: SystemGetTransactionHashInput =
                    scrypto_decode(&arg.raw).map_err(|e| SystemError::InvalidRequestData(e))?;
                Ok(ScryptoValue::from_typed(
                    &system_api
                        .transaction_hash()
                        .map_err(SystemError::CostingError)?,
                ))
            }
            _ => Err(InvalidMethod),
        }
    }
}
