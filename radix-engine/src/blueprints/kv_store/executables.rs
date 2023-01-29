use super::KeyValueStoreEntrySubstate;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::*;
use crate::system::node::RENodeInit;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientDerefApi;
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::blueprints::kv_store::*;

pub struct KeyValueStore;

impl ExecutableInvocation for KeyValueStoreCreateInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = ResolvedActor::function(NativeFn::KeyValueStore(KeyValueStoreFn::Create));
        let call_frame_update = CallFrameUpdate::empty();

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for KeyValueStoreCreateInvocation {
    type Output = Own;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let node_id = api.allocate_node_id(RENodeType::KeyValueStore)?;

        api.create_node(node_id, RENodeInit::KeyValueStore)?;

        let update = CallFrameUpdate {
            nodes_to_move: vec![node_id],
            node_refs_to_copy: HashSet::new(),
        };

        Ok((Own::KeyValueStore(node_id.into()), update))
    }
}

pub struct KeyValueStoreInsertExecutable(RENodeId, Vec<u8>, Vec<u8>);

impl ExecutableInvocation for KeyValueStoreInsertInvocation {
    type Exec = KeyValueStoreInsertExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = ResolvedReceiver::new(RENodeId::KeyValueStore(self.receiver));

        let actor = ResolvedActor::method(
            NativeFn::KeyValueStore(KeyValueStoreFn::Insert),
            resolved_receiver,
        );
        let executor =
            KeyValueStoreInsertExecutable(resolved_receiver.receiver, self.key, self.value);

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for KeyValueStoreInsertExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = self.0;
        let key = self.1;
        let value = self.2;

        let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key));
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_ref = api.get_ref_mut(handle)?;
        *substate_ref.kv_store_entry() = KeyValueStoreEntrySubstate(Some(value));

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct KeyValueStoreGetExecutable(RENodeId, Vec<u8>);

impl ExecutableInvocation for KeyValueStoreGetInvocation {
    type Exec = KeyValueStoreGetExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let call_frame_update = CallFrameUpdate::empty();
        let resolved_receiver = ResolvedReceiver::new(RENodeId::KeyValueStore(self.receiver));

        let actor = ResolvedActor::method(
            NativeFn::KeyValueStore(KeyValueStoreFn::Get),
            resolved_receiver,
        );
        let executor = KeyValueStoreGetExecutable(resolved_receiver.receiver, self.key);

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for KeyValueStoreGetExecutable {
    type Output = Option<Vec<u8>>;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Option<Vec<u8>>, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = self.0;
        let key = self.1;

        let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key));
        let handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = api.get_ref(handle)?;
        let substate = substate_ref.kv_store_entry();
        let value = substate.0.clone();
        Ok((value, CallFrameUpdate::empty()))
    }
}
