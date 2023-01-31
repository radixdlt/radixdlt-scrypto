use crate::errors::KernelError;
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

impl ExecutableInvocation for KeyValueStoreGetInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::KeyValueStore(self.receiver));

        let resolved_receiver = ResolvedReceiver::new(RENodeId::KeyValueStore(self.receiver));
        let actor = ResolvedActor::method(
            NativeFn::KeyValueStore(KeyValueStoreFn::Get),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for KeyValueStoreGetInvocation {
    type Output = LockHandle;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(LockHandle, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(self.hash));
        let handle = api.lock_substate(
            RENodeId::KeyValueStore(self.receiver),
            offset,
            LockFlags::read_only(),
        )?;
        Ok((handle, CallFrameUpdate::empty()))
    }
}

pub struct KeyValueStoreGetMutExecutable(RENodeId, Hash);

impl ExecutableInvocation for KeyValueStoreGetMutInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::KeyValueStore(self.receiver));

        let resolved_receiver = ResolvedReceiver::new(RENodeId::KeyValueStore(self.receiver));
        let actor = ResolvedActor::method(
            NativeFn::KeyValueStore(KeyValueStoreFn::Get),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for KeyValueStoreGetMutInvocation {
    type Output = LockHandle;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(LockHandle, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(self.hash));
        let handle = api.lock_substate(
            RENodeId::KeyValueStore(self.receiver),
            offset,
            LockFlags::MUTABLE,
        )?;

        Ok((handle, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for KeyValueStoreInsertInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::KeyValueStore(self.receiver));
        for id in IndexedScryptoValue::from_value(self.key.clone())
            .owned_node_ids()
            .map_err(|e| RuntimeError::KernelError(KernelError::ReadOwnedNodesError(e)))?
        {
            call_frame_update.nodes_to_move.push(id);
        }
        // TODO: reference passing?

        let resolved_receiver = ResolvedReceiver::new(RENodeId::KeyValueStore(self.receiver));
        let actor = ResolvedActor::method(
            NativeFn::KeyValueStore(KeyValueStoreFn::Insert),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for KeyValueStoreInsertInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(self.hash));
        let handle = api.lock_substate(
            RENodeId::KeyValueStore(self.receiver),
            offset,
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.get_ref_mut(handle)?;
        *substate_ref.kv_store_entry() = KeyValueStoreEntrySubstate::Some(self.key, self.value);

        Ok(((), CallFrameUpdate::empty()))
    }
}
