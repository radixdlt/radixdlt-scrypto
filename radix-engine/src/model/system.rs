use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::{
    SystemGetCurrentEpochInput, SystemGetTransactionHashInput, SystemSetEpochInput,
};
use scrypto::engine::types::RENodeId;
use scrypto::values::ScryptoValue;

use crate::engine::SystemApi;
use crate::fee::CostUnitCounter;
use crate::fee::CostUnitCounterError;
use crate::model::SystemError::InvalidMethod;
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SystemError {
    InvalidRequestData(DecodeError),
    InvalidMethod,
    CostingError(CostUnitCounterError),
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
        C: CostUnitCounter,
    >(
        method_name: &str,
        arg: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, SystemError> {
        match method_name {
            "get_epoch" => {
                let _: SystemGetCurrentEpochInput =
                    scrypto_decode(&arg.raw).map_err(|e| SystemError::InvalidRequestData(e))?;
                let value = system_api
                    .borrow_value(&RENodeId::System)
                    .map_err(SystemError::CostingError)?;
                Ok(ScryptoValue::from_typed(&value.system().epoch))
            }
            "set_epoch" => {
                let SystemSetEpochInput { epoch } =
                    scrypto_decode(&arg.raw).map_err(|e| SystemError::InvalidRequestData(e))?;
                let mut system_value = system_api
                    .borrow_value_mut(&RENodeId::System)
                    .map_err(SystemError::CostingError)?;
                system_value.system().epoch = epoch;
                system_api
                    .return_value_mut(system_value)
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
