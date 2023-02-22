use crate::errors::{InterpreterError, RuntimeError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::logger::*;
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, IndexedScryptoValue, ScryptoValue,
};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct LoggerSubstate {
    pub logs: Vec<(Level, String)>,
}

pub struct LoggerNativePackage;
impl LoggerNativePackage {
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            LOGGER_LOG_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;

                Self::log(receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub(crate) fn log<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: LoggerLogInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Logger(LoggerOffset::Logger),
            LockFlags::MUTABLE,
        )?;
        let mut substate = api.kernel_get_substate_ref_mut(handle)?;
        let logger = substate.logger();
        logger.logs.push((input.level, input.message));

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}
