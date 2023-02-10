use crate::engine::*;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::EngineApi;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TransactionRuntimeError {
    OutOfUUid,
}

impl ExecutableInvocation for TransactionRuntimeGetHashInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = ResolvedActor::method(
            NativeFn::TransactionRuntime(TransactionRuntimeFn::Get),
            ResolvedReceiver::new(RENodeId::TransactionRuntime(self.receiver)),
        );
        let call_frame_update = CallFrameUpdate::empty();

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for TransactionRuntimeGetHashInvocation {
    type Output = Hash;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        let offset =
            SubstateOffset::TransactionRuntime(TransactionRuntimeOffset::TransactionRuntime);
        let node_id = RENodeId::TransactionRuntime(self.receiver);
        let handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate = api.get_ref(handle)?;
        let transaction_runtime_substate = substate.transaction_runtime();
        Ok((
            transaction_runtime_substate.hash.clone(),
            CallFrameUpdate::empty(),
        ))
    }
}

impl ExecutableInvocation for TransactionRuntimeGenerateUuidInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = ResolvedActor::method(
            NativeFn::TransactionRuntime(TransactionRuntimeFn::GenerateUuid),
            ResolvedReceiver::new(RENodeId::TransactionRuntime(self.receiver)),
        );

        let call_frame_update = CallFrameUpdate::empty();

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for TransactionRuntimeGenerateUuidInvocation {
    type Output = u128;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        let offset =
            SubstateOffset::TransactionRuntime(TransactionRuntimeOffset::TransactionRuntime);
        let node_id = RENodeId::TransactionRuntime(self.receiver);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_mut = api.get_ref_mut(handle)?;
        let tx_hash_substate = substate_mut.transaction_runtime();

        if tx_hash_substate.next_id == u32::MAX {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::TransactionRuntimeError(TransactionRuntimeError::OutOfUUid),
            ));
        }

        let uuid = generate_uuid(&tx_hash_substate.hash, tx_hash_substate.next_id);
        tx_hash_substate.next_id = tx_hash_substate.next_id + 1;

        Ok((uuid, CallFrameUpdate::empty()))
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
