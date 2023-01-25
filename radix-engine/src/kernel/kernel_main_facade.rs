use super::module::SysCallOutput;
use crate::errors::RuntimeError;
use crate::errors::*;
use crate::kernel::kernel_api::{
    KernelNodeApi, KernelSubstateApi, KernelWasmApi, LockFlags, LockInfo,
};
use crate::kernel::module::BaseModule;
use crate::kernel::*;
use crate::system::global::GlobalAddressSubstate;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::system::substates::{SubstateRef, SubstateRefMut};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::blueprints::resource::Resource;
use radix_engine_interface::api::types::{
    ComponentOffset, GlobalAddress, GlobalOffset, LockHandle, RENodeId, SubstateId, SubstateOffset,
    VaultId,
};

impl<'g, 's, W, R, M> KernelNodeApi for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        mut fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        fee = self
            .module
            .on_lock_fee(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                vault_id,
                fee,
                contingent,
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(fee)
    }

    fn get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::ReadOwnedNodes,
            )
            .map_err(RuntimeError::ModuleError)?;

        let node_ids = self.current_frame.get_visible_nodes();

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::ReadOwnedNodes,
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(node_ids)
    }

    fn get_visible_node_data(
        &mut self,
        node_id: RENodeId,
    ) -> Result<RENodeVisibilityOrigin, RuntimeError> {
        let visibility = self.current_frame.get_node_visibility(node_id)?;
        Ok(visibility)
    }

    fn drop_node(&mut self, node_id: RENodeId) -> Result<HeapRENode, RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::DropNode { node_id: &node_id },
            )
            .map_err(RuntimeError::ModuleError)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        if !VisibilityProperties::check_drop_node_visibility(
            current_mode,
            &self.current_frame.actor,
            node_id,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidDropNodeVisibility {
                    mode: current_mode,
                    actor: self.current_frame.actor.clone(),
                    node_id,
                },
            ));
        }

        let node = self.drop_node_internal(node_id)?;

        // Restore current mode
        self.execution_mode = current_mode;

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::DropNode { node: &node },
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(node)
    }

    fn allocate_node_id(&mut self, node_type: RENodeType) -> Result<RENodeId, RuntimeError> {
        // TODO: Add costing
        let node_id = self.id_allocator.allocate_node_id(node_type)?;

        Ok(node_id)
    }

    fn create_node(&mut self, node_id: RENodeId, re_node: RENodeInit) -> Result<(), RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::CreateNode { node: &re_node },
            )
            .map_err(RuntimeError::ModuleError)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        if !VisibilityProperties::check_create_node_visibility(
            current_mode,
            &self.current_frame.actor,
            &re_node,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidCreateNodeVisibility {
                    mode: current_mode,
                    actor: self.current_frame.actor.clone(),
                },
            ));
        }

        match (node_id, &re_node) {
            (
                RENodeId::Global(GlobalAddress::Package(..)),
                RENodeInit::Global(GlobalAddressSubstate::Package(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Resource(..)),
                RENodeInit::Global(GlobalAddressSubstate::Resource(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::EpochManager(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::Clock(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::Validator(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::Identity(..)),
            ) => {}
            (
                RENodeId::Global(address),
                RENodeInit::Global(GlobalAddressSubstate::Component(component)),
            ) => {
                // TODO: Get rid of this logic
                let (package_address, blueprint_name) = self
                    .execute_in_mode::<_, _, RuntimeError>(
                        ExecutionMode::Globalize,
                        |system_api| {
                            let handle = system_api.lock_substate(
                                RENodeId::Component(*component),
                                SubstateOffset::Component(ComponentOffset::Info),
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = system_api.get_ref(handle)?;
                            let info = substate_ref.component_info();
                            let package_blueprint =
                                (info.package_address, info.blueprint_name.clone());
                            system_api.drop_lock(handle)?;
                            Ok(package_blueprint)
                        },
                    )?;

                match address {
                    GlobalAddress::Component(ComponentAddress::Account(..)) => {
                        if !(package_address.eq(&ACCOUNT_PACKAGE)
                            && blueprint_name.eq(&ACCOUNT_BLUEPRINT))
                        {
                            return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id)));
                        }
                    }
                    GlobalAddress::Component(ComponentAddress::Normal(..)) => {
                        if package_address.eq(&ACCOUNT_PACKAGE)
                            && blueprint_name.eq(&ACCOUNT_BLUEPRINT)
                        {
                            return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id)));
                        }
                    }
                    _ => {
                        return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id)));
                    }
                }
            }
            (RENodeId::Bucket(..), RENodeInit::Bucket(..)) => {}
            (RENodeId::TransactionRuntime(..), RENodeInit::TransactionRuntime(..)) => {}
            (RENodeId::Proof(..), RENodeInit::Proof(..)) => {}
            (RENodeId::AuthZoneStack(..), RENodeInit::AuthZoneStack(..)) => {}
            (RENodeId::Vault(..), RENodeInit::Vault(..)) => {}
            (RENodeId::Component(..), RENodeInit::Component(..)) => {}
            (RENodeId::Worktop, RENodeInit::Worktop(..)) => {}
            (RENodeId::Logger, RENodeInit::Logger(..)) => {}
            (RENodeId::Package(..), RENodeInit::Package(..)) => {}
            (RENodeId::KeyValueStore(..), RENodeInit::KeyValueStore(..)) => {}
            (RENodeId::NonFungibleStore(..), RENodeInit::NonFungibleStore(..)) => {}
            (RENodeId::ResourceManager(..), RENodeInit::ResourceManager(..)) => {}
            (RENodeId::EpochManager(..), RENodeInit::EpochManager(..)) => {}
            (RENodeId::Validator(..), RENodeInit::Validator(..)) => {}
            (RENodeId::Clock(..), RENodeInit::Clock(..)) => {}
            (RENodeId::Identity(..), RENodeInit::Identity(..)) => {}
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id))),
        }

        // TODO: For Scrypto components, check state against blueprint schema

        let push_to_store = match re_node {
            RENodeInit::Global(..) | RENodeInit::Logger(..) => true,
            _ => false,
        };

        self.id_allocator.take_node_id(node_id)?;
        self.current_frame.create_node(
            node_id,
            re_node,
            &mut self.heap,
            &mut self.track,
            push_to_store,
        )?;

        // Restore current mode
        self.execution_mode = current_mode;

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::CreateNode { node_id: &node_id },
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(())
    }
}

impl<'g, 's, W, R, M> KernelSubstateApi for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::LockSubstate {
                    node_id: &node_id,
                    offset: &offset,
                    flags: &flags,
                },
            )
            .map_err(RuntimeError::ModuleError)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        // Deref
        let (node_id, derefed_lock) =
            if let Some((node_id, derefed_lock)) = self.node_offset_deref(node_id, &offset)? {
                (node_id, Some(derefed_lock))
            } else {
                (node_id, None)
            };

        // TODO: Check if valid offset for node_id

        // Authorization
        let actor = &self.current_frame.actor;
        if !VisibilityProperties::check_substate_visibility(
            current_mode,
            actor,
            node_id,
            offset.clone(),
            flags,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidSubstateVisibility {
                    mode: current_mode,
                    actor: actor.clone(),
                    node_id,
                    offset,
                    flags,
                },
            ));
        }

        let maybe_lock_handle = self.current_frame.acquire_lock(
            &mut self.heap,
            &mut self.track,
            node_id,
            offset.clone(),
            flags,
        );

        let lock_handle = match maybe_lock_handle {
            Ok(lock_handle) => lock_handle,
            Err(RuntimeError::KernelError(KernelError::TrackError(TrackError::NotFound(
                SubstateId(node_id, ref offset),
            )))) => {
                if self.try_virtualize(node_id, &offset)? {
                    self.current_frame.acquire_lock(
                        &mut self.heap,
                        &mut self.track,
                        node_id,
                        offset.clone(),
                        flags,
                    )?
                } else {
                    return maybe_lock_handle;
                }
            }
            Err(err) => {
                match &err {
                    // TODO: This is a hack to allow for package imports to be visible
                    // TODO: Remove this once we are able to get this information through the Blueprint ABI
                    RuntimeError::CallFrameError(CallFrameError::RENodeNotVisible(
                        RENodeId::Global(GlobalAddress::Package(package_address)),
                    )) => {
                        let node_id = RENodeId::Global(GlobalAddress::Package(*package_address));
                        let offset = SubstateOffset::Global(GlobalOffset::Global);
                        self.track
                            .acquire_lock(
                                SubstateId(node_id, offset.clone()),
                                LockFlags::read_only(),
                            )
                            .map_err(|_| err.clone())?;
                        self.track
                            .release_lock(SubstateId(node_id, offset.clone()), false)
                            .map_err(|_| err)?;
                        self.current_frame
                            .add_stored_ref(node_id, RENodeVisibilityOrigin::Normal);
                        self.current_frame.acquire_lock(
                            &mut self.heap,
                            &mut self.track,
                            node_id,
                            offset.clone(),
                            flags,
                        )?
                    }
                    _ => return Err(err),
                }
            }
        };

        if let Some(lock_handle) = derefed_lock {
            self.current_frame
                .drop_lock(&mut self.heap, &mut self.track, lock_handle)?;
        }

        // Restore current mode
        self.execution_mode = current_mode;

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::LockSubstate { lock_handle },
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(lock_handle)
    }

    fn get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        self.current_frame.get_lock_info(lock_handle)
    }

    fn drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::DropLock {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;

        self.current_frame
            .drop_lock(&mut self.heap, &mut self.track, lock_handle)?;

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::DropLock,
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(())
    }

    fn get_ref(&mut self, lock_handle: LockHandle) -> Result<SubstateRef, RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::GetRef {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;

        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.
        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::GetRef { lock_handle },
            )
            .map_err(RuntimeError::ModuleError)?;

        let substate_ref =
            self.current_frame
                .get_ref(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref)
    }

    fn get_ref_mut(&mut self, lock_handle: LockHandle) -> Result<SubstateRefMut, RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::GetRefMut {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;

        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.
        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::GetRefMut,
            )
            .map_err(RuntimeError::ModuleError)?;

        let substate_ref_mut =
            self.current_frame
                .get_ref_mut(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref_mut)
    }
}

impl<'g, 's, W, R, M> KernelWasmApi<W> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn scrypto_interpreter(&mut self) -> &ScryptoInterpreter<W> {
        self.scrypto_interpreter
    }

    fn emit_wasm_instantiation_event(&mut self, code: &[u8]) -> Result<(), RuntimeError> {
        self.module
            .on_wasm_instantiation(&self.current_frame, &mut self.heap, &mut self.track, code)
            .map_err(RuntimeError::ModuleError)?;

        Ok(())
    }
}

impl<'g, 's, W, R, M> KernelApi<W> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}
