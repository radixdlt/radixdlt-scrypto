use sbor::DecodeError;
use scrypto::buffer::scrypto_decode;
use scrypto::core::{SystemGetCurrentEpochInput, SystemGetTransactionHashInput};
use scrypto::values::ScryptoValue;

use crate::engine::SystemApi;
use crate::ledger::ReadableSubstateStore;
use crate::model::SystemError::InvalidMethod;
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SystemError {
    InvalidRequestData(DecodeError),
    InvalidMethod,
}

pub struct System {}

impl System {
    pub fn static_main<
        'p,
        's,
        Y: SystemApi<'p, 's, W, I, S>,
        W: WasmEngine<I>,
        I: WasmInstance,
        S: ReadableSubstateStore,
    >(
        method_name: &str,
        arg: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, SystemError> {
        match method_name {
            "current_epoch" => {
                let _: SystemGetCurrentEpochInput =
                    scrypto_decode(&arg.raw).map_err(|e| SystemError::InvalidRequestData(e))?;
                // TODO: Make this stateful
                Ok(ScryptoValue::from_typed(&system_api.get_epoch()))
            }
            "transaction_hash" => {
                let _: SystemGetTransactionHashInput =
                    scrypto_decode(&arg.raw).map_err(|e| SystemError::InvalidRequestData(e))?;
                Ok(ScryptoValue::from_typed(&system_api.get_transaction_hash()))
            }
            _ => Err(InvalidMethod),
        }
    }
}
