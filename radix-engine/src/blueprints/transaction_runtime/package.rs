use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::transaction_runtime::*;
use radix_engine_interface::schema::PackageSchema;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum TransactionRuntimeError {
    OutOfUUid,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct TransactionRuntimeSubstate {
    pub hash: Hash,
    pub next_id: u32,
}

pub struct TransactionRuntimeNativePackage;

impl TransactionRuntimeNativePackage {
    pub fn schema() -> PackageSchema {
        todo!()
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            TRANSACTION_RUNTIME_GET_HASH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;

                Self::get_hash(receiver, input, api)
            }
            TRANSACTION_RUNTIME_GENERATE_UUID_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;

                Self::generate_uuid(receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub(crate) fn get_hash<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: TransactionRuntimeGetHashInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::TransactionRuntime(TransactionRuntimeOffset::TransactionRuntime),
            LockFlags::read_only(),
        )?;
        let transaction_runtime_substate: &TransactionRuntimeSubstate =
            api.kernel_get_substate_ref(handle)?;
        Ok(IndexedScryptoValue::from_typed(
            &transaction_runtime_substate.hash,
        ))
    }

    pub(crate) fn generate_uuid<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: TransactionRuntimeGenerateUuid = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::TransactionRuntime(TransactionRuntimeOffset::TransactionRuntime),
            LockFlags::MUTABLE,
        )?;
        let tx_hash_substate: &mut TransactionRuntimeSubstate =
            api.kernel_get_substate_ref_mut(handle)?;

        if tx_hash_substate.next_id == u32::MAX {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::TransactionRuntimeError(TransactionRuntimeError::OutOfUUid),
            ));
        }

        let uuid = generate_uuid(&tx_hash_substate.hash, tx_hash_substate.next_id);
        tx_hash_substate.next_id = tx_hash_substate.next_id + 1;

        Ok(IndexedScryptoValue::from_typed(&uuid))
    }
}

fn generate_uuid(hash: &Hash, id: u32) -> u128 {
    // Take the lower 16 bytes
    let mut temp = hash.lower_16_bytes();

    // Put TX runtime counter to the last 4 bytes.
    temp[12..16].copy_from_slice(&id.to_be_bytes());

    // Construct UUID v4 variant 1
    (u128::from_be_bytes(temp) & 0xffffffff_ffff_0fff_3fff_ffffffffffffu128)
        | 0x00000000_0000_4000_8000_000000000000u128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_gen() {
        let id = generate_uuid(
            &Hash::from_str("71f26aab5eec6679f67c71211aba9a3486cc8d24194d339385ee91ee5ca7b30d")
                .unwrap(),
            5,
        );
        assert_eq!(
            NonFungibleLocalId::uuid(id).unwrap().to_string(),
            "{86cc8d24-194d-4393-85ee-91ee00000005}"
        );

        let id = generate_uuid(&Hash([0u8; 32]), 5);
        assert_eq!(
            NonFungibleLocalId::uuid(id).unwrap().to_string(),
            "{00000000-0000-4000-8000-000000000005}"
        );

        let id = generate_uuid(&Hash([255u8; 32]), 5);
        assert_eq!(
            NonFungibleLocalId::uuid(id).unwrap().to_string(),
            "{ffffffff-ffff-4fff-bfff-ffff00000005}"
        );
    }
}
