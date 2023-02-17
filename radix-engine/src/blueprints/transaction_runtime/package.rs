use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi, LockFlags};
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::transaction_runtime::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TransactionRuntimeError {
    OutOfUUid,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionRuntimeSubstate {
    pub hash: Hash,
    pub next_id: u32,
}

pub struct TransactionRuntimeNativePackage;
impl TransactionRuntimeNativePackage {
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<ComponentId>,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            TRANSACTION_RUNTIME_GET_HASH_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;

                Self::get_hash(receiver, input, api)
            }
            TRANSACTION_RUNTIME_GENERATE_UUID_IDENT => {
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
        _ignored: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: TransactionRuntimeGetHashInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.kernel_lock_substate(
            RENodeId::TransactionRuntime,
            NodeModuleId::SELF,
            SubstateOffset::TransactionRuntime(TransactionRuntimeOffset::TransactionRuntime),
            LockFlags::read_only(),
        )?;
        let substate = api.kernel_get_substate_ref(handle)?;
        let transaction_runtime_substate = substate.transaction_runtime();
        Ok(IndexedScryptoValue::from_typed(
            &transaction_runtime_substate.hash,
        ))
    }

    pub(crate) fn generate_uuid<Y>(
        _ignored: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: TransactionRuntimeGenerateUuid =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.kernel_lock_substate(
            RENodeId::TransactionRuntime,
            NodeModuleId::SELF,
            SubstateOffset::TransactionRuntime(TransactionRuntimeOffset::TransactionRuntime),
            LockFlags::MUTABLE,
        )?;
        let mut substate_mut = api.kernel_get_substate_ref_mut(handle)?;
        let tx_hash_substate = substate_mut.transaction_runtime();

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
