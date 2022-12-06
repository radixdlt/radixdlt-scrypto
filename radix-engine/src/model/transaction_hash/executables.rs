use crate::engine::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::types::*;

impl<W: WasmEngine> ExecutableInvocation<W> for TransactionHashGetInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolverApi<W>>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::TransactionHash(TransactionHashMethod::Get)),
            ResolvedReceiver::new(RENodeId::TransactionHash(self.receiver)),
        );
        let call_frame_update = CallFrameUpdate::empty();
        let executor = NativeExecutor(self);

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for TransactionHashGetInvocation {
    type Output = Hash;

    fn main<Y>(self, api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError>,
    {
        let offset = SubstateOffset::TransactionHash(TransactionHashOffset::TransactionHash);
        let node_id = RENodeId::TransactionHash(self.receiver);
        let handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate = api.get_ref(handle)?;
        let transaction_hash_substate = substate.transaction_hash();
        Ok((
            transaction_hash_substate.hash.clone(),
            CallFrameUpdate::empty(),
        ))
    }
}
