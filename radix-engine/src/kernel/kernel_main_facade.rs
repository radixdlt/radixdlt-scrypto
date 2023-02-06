use crate::errors::*;
use crate::kernel::kernel_api::{Invokable, KernelNodeApi, KernelSubstateApi, KernelWasmApi};
use crate::kernel::module::BaseModule;
use crate::kernel::module::KernelApiCallOutput;
use crate::kernel::*;
use crate::system::global::GlobalAddressSubstate;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_properties::VisibilityProperties;
use crate::system::node_substates::{SubstateRef, SubstateRefMut};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::Resource;

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
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            fee = api
                .module
                .on_lock_fee(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    vault_id,
                    fee,
                    contingent,
                )
                .map_err(RuntimeError::ModuleError)?;

            Ok(fee)
        })
    }

    fn get_visible_node_data(
        &mut self,
        node_id: RENodeId,
    ) -> Result<RENodeVisibilityOrigin, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            let visibility = api.current_frame.get_node_visibility(node_id)?;
            Ok(visibility)
        })
    }

    fn drop_node(&mut self, node_id: RENodeId) -> Result<HeapRENode, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            api.module
                .pre_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallInput::DropNode { node_id: &node_id },
                )
                .map_err(RuntimeError::ModuleError)?;

            if !VisibilityProperties::check_drop_node_visibility(
                api.execution_mode(),
                &api.current_frame.actor,
                node_id,
            ) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidDropNodeVisibility {
                        mode: api.execution_mode(),
                        actor: api.current_frame.actor.clone(),
                        node_id,
                    },
                ));
            }

            let node = api.drop_node_internal(node_id)?;

            api.module
                .post_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallOutput::DropNode { node: &node },
                )
                .map_err(RuntimeError::ModuleError)?;

            Ok(node)
        })
    }

    fn allocate_node_id(&mut self, node_type: RENodeType) -> Result<RENodeId, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            // TODO: Add costing
            let node_id = api.id_allocator.allocate_node_id(node_type)?;

            Ok(node_id)
        })
    }

    fn create_node(
        &mut self,
        node_id: RENodeId,
        re_node: RENodeInit,
        module_init: BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            api.module
                .pre_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallInput::CreateNode { node: &re_node },
                )
                .map_err(RuntimeError::ModuleError)?;

            if !VisibilityProperties::check_create_node_visibility(
                api.execution_mode(),
                &api.current_frame.actor,
                &re_node,
                &module_init,
            ) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidCreateNodeVisibility {
                        mode: api.execution_mode(),
                        actor: api.current_frame.actor.clone(),
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
                    RENodeId::Global(GlobalAddress::Component(..)),
                    RENodeInit::Global(GlobalAddressSubstate::AccessController(..)),
                ) => {}
                (
                    RENodeId::Global(..),
                    RENodeInit::Global(GlobalAddressSubstate::Component(..)),
                ) => {}
                (RENodeId::Global(..), RENodeInit::Global(GlobalAddressSubstate::Account(..))) => {}
                (RENodeId::Bucket(..), RENodeInit::Bucket(..)) => {}
                (RENodeId::TransactionRuntime, RENodeInit::TransactionRuntime(..)) => {}
                (RENodeId::Proof(..), RENodeInit::Proof(..)) => {}
                (RENodeId::AuthZoneStack, RENodeInit::AuthZoneStack(..)) => {}
                (RENodeId::Vault(..), RENodeInit::Vault(..)) => {}
                (RENodeId::Component(..), RENodeInit::Component(..)) => {}
                (RENodeId::Worktop, RENodeInit::Worktop(..)) => {}
                (RENodeId::Logger, RENodeInit::Logger(..)) => {}
                (RENodeId::Package(..), RENodeInit::Package(..)) => {}
                (RENodeId::KeyValueStore(..), RENodeInit::KeyValueStore) => {}
                (RENodeId::NonFungibleStore(..), RENodeInit::NonFungibleStore(..)) => {}
                (RENodeId::ResourceManager(..), RENodeInit::ResourceManager(..)) => {}
                (RENodeId::EpochManager(..), RENodeInit::EpochManager(..)) => {}
                (RENodeId::Validator(..), RENodeInit::Validator(..)) => {}
                (RENodeId::Clock(..), RENodeInit::Clock(..)) => {}
                (RENodeId::Identity(..), RENodeInit::Identity(..)) => {}
                (RENodeId::AccessController(..), RENodeInit::AccessController(..)) => {}
                (RENodeId::Account(..), RENodeInit::Account(..)) => {}
                _ => return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id))),
            }

            // TODO: For Scrypto components, check state against blueprint schema

            let push_to_store = match re_node {
                RENodeInit::Global(..) | RENodeInit::Logger(..) => true,
                _ => false,
            };

            api.id_allocator.take_node_id(node_id)?;
            api.current_frame.create_node(
                node_id,
                re_node,
                module_init,
                &mut api.heap,
                &mut api.track,
                push_to_store,
            )?;

            api.module
                .post_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallOutput::CreateNode { node_id: &node_id },
                )
                .map_err(RuntimeError::ModuleError)?;

            Ok(())
        })
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
        module_id: NodeModuleId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            api.module
                .pre_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallInput::LockSubstate {
                        node_id: &node_id,
                        offset: &offset,
                        flags: &flags,
                    },
                )
                .map_err(RuntimeError::ModuleError)?;

            // TODO: Check if valid offset for node_id

            // Authorization
            let actor = &api.current_frame.actor;
            if !VisibilityProperties::check_substate_visibility(
                api.execution_mode(),
                actor,
                node_id,
                offset.clone(),
                flags,
            ) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidSubstateVisibility {
                        mode: api.execution_mode(),
                        actor: actor.clone(),
                        node_id,
                        offset,
                        flags,
                    },
                ));
            }

            let maybe_lock_handle = api.current_frame.acquire_lock(
                &mut api.heap,
                &mut api.track,
                node_id,
                module_id,
                offset.clone(),
                flags,
            );

            let lock_handle = match maybe_lock_handle {
                Ok(lock_handle) => lock_handle,
                Err(RuntimeError::KernelError(KernelError::TrackError(TrackError::NotFound(
                    SubstateId(node_id, module_id, ref offset),
                )))) => {
                    if api.try_virtualize(node_id, &offset)? {
                        api.current_frame.acquire_lock(
                            &mut api.heap,
                            &mut api.track,
                            node_id,
                            module_id,
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
                            let node_id =
                                RENodeId::Global(GlobalAddress::Package(*package_address));
                            let module_id = NodeModuleId::SELF;
                            let offset = SubstateOffset::Global(GlobalOffset::Global);
                            api.track
                                .acquire_lock(
                                    SubstateId(node_id, module_id, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| err.clone())?;
                            api.track
                                .release_lock(SubstateId(node_id, module_id, offset.clone()), false)
                                .map_err(|_| err)?;
                            api.current_frame
                                .add_stored_ref(node_id, RENodeVisibilityOrigin::Normal);
                            api.current_frame.acquire_lock(
                                &mut api.heap,
                                &mut api.track,
                                node_id,
                                module_id,
                                offset.clone(),
                                flags,
                            )?
                        }
                        _ => return Err(err),
                    }
                }
            };

            api.module
                .post_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallOutput::LockSubstate { lock_handle },
                )
                .map_err(RuntimeError::ModuleError)?;

            Ok(lock_handle)
        })
    }

    fn get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            api.current_frame.get_lock_info(lock_handle)
        })
    }

    fn drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            api.module
                .pre_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallInput::DropLock {
                        lock_handle: &lock_handle,
                    },
                )
                .map_err(RuntimeError::ModuleError)?;

            api.current_frame
                .drop_lock(&mut api.heap, &mut api.track, lock_handle)?;

            api.module
                .post_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallOutput::DropLock,
                )
                .map_err(RuntimeError::ModuleError)?;

            Ok(())
        })
    }

    fn get_ref(&mut self, lock_handle: LockHandle) -> Result<SubstateRef, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            api.module
                .pre_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallInput::GetRef {
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
            api.module
                .post_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallOutput::GetRef { lock_handle },
                )
                .map_err(RuntimeError::ModuleError)?;

            let substate_ref =
                api.current_frame
                    .get_ref(lock_handle, &mut api.heap, &mut api.track)?;

            Ok(substate_ref)
        })
    }

    fn get_ref_mut(&mut self, lock_handle: LockHandle) -> Result<SubstateRefMut, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            api.module
                .pre_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallInput::GetRefMut {
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
            api.module
                .post_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallOutput::GetRefMut,
                )
                .map_err(RuntimeError::ModuleError)?;

            let substate_ref_mut =
                api.current_frame
                    .get_ref_mut(lock_handle, &mut api.heap, &mut api.track)?;

            Ok(substate_ref_mut)
        })
    }
}

impl<'g, 's, W, R, M> KernelWasmApi<W> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn scrypto_interpreter(&mut self) -> &ScryptoInterpreter<W> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            Ok(api.scrypto_interpreter)
        })
        .expect("No error expected")
    }

    fn emit_wasm_instantiation_event(&mut self, code: &[u8]) -> Result<(), RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            api.module
                .on_wasm_instantiation(&api.current_frame, &mut api.heap, &mut api.track, code)
                .map_err(RuntimeError::ModuleError)?;

            Ok(())
        })
    }
}

impl<'g, 's, W, R, N, M> Invokable<N, RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
    N: ExecutableInvocation,
{
    fn invoke(&mut self, invocation: N) -> Result<<N as Invocation>::Output, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Kernel, |api| {
            api.module
                .pre_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallInput::Invoke {
                        fn_identifier: invocation.fn_identifier(),
                        input_size: 0, // TODO: Fix this
                        depth: api.current_frame.depth,
                    },
                )
                .map_err(RuntimeError::ModuleError)?;

            let (actor, call_frame_update, executor) = invocation.resolve(api)?;
            let rtn = api.invoke_internal(executor, actor, call_frame_update)?;

            api.module
                .post_kernel_api_call(
                    &api.current_frame,
                    &mut api.heap,
                    &mut api.track,
                    KernelApiCallOutput::Invoke { rtn: &rtn },
                )
                .map_err(RuntimeError::ModuleError)?;

            Ok(rtn)
        })
    }
}

impl<'g, 's, W, R, M> KernelInvokeApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}

impl<'g, 's, W, R, M> KernelApi<W, RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}
