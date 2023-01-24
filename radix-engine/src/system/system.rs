use super::kernel_modules::fee::FeeReserve;
use super::substates::{SubstateRef, SubstateRefMut};
use crate::blueprints::resource::Resource;
use crate::errors::*;
use crate::kernel::HeapRENode;
use crate::kernel::LockFlags;
use crate::kernel::LockInfo;
use crate::kernel::RENodeInit;
use crate::kernel::RENodeVisibilityOrigin;
use crate::{
    blueprints::transaction_processor::{InstructionOutput, TransactionProcessorRunInvocation},
    kernel::{BaseModule, IdAllocator, Kernel, ScryptoInterpreter, SubstateApi, Track},
    wasm::WasmEngine,
};
use radix_engine_interface::api::{types::*, Invokable};
use sbor::rust::borrow::Cow;
use transaction::model::{AuthZoneParams, Instruction, RuntimeValidationRequest};

pub struct System<
    'g, // Lifetime of values outliving all frames
    's, // Substate store lifetime
    W,  // WASM engine type
    R,  // Fee reserve type
    M,
> where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    kernel: Kernel<'g, 's, W, R, M>,
}

impl<'g, 's, W, R, M> System<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    pub fn new(
        auth_zone_params: AuthZoneParams,
        id_allocator: &'g mut IdAllocator,
        track: &'g mut Track<'s, R>,
        scrypto_interpreter: &'g ScryptoInterpreter<W>,
        module: &'g mut M,
    ) -> Self {
        Self {
            kernel: Kernel::new(
                auth_zone_params,
                id_allocator,
                track,
                scrypto_interpreter,
                module,
            ),
        }
    }

    pub fn run_transaction<'a>(
        &mut self,
        transaction_hash: Hash,
        runtime_validations: Cow<'a, [RuntimeValidationRequest]>,
        instructions: Cow<'a, [Instruction]>,
        blobs: Cow<'a, [Vec<u8>]>,
    ) -> Result<Vec<InstructionOutput>, RuntimeError> {
        self.kernel.invoke(TransactionProcessorRunInvocation {
            transaction_hash,
            runtime_validations,
            instructions,
            blobs,
        })
    }
}

impl<'g, 's, W, R, M> SubstateApi for System<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn consume_cost_units(&mut self, units: u32) -> Result<(), RuntimeError> {
        self.kernel.consume_cost_units(units)
    }

    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        self.kernel.lock_fee(vault_id, fee, contingent)
    }

    fn get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, RuntimeError> {
        self.kernel.get_visible_nodes()
    }

    fn get_visible_node_data(
        &mut self,
        node_id: RENodeId,
    ) -> Result<RENodeVisibilityOrigin, RuntimeError> {
        self.kernel.get_visible_node_data(node_id)
    }

    fn drop_node(&mut self, node_id: RENodeId) -> Result<HeapRENode, RuntimeError> {
        self.kernel.drop_node(node_id)
    }

    fn allocate_node_id(&mut self, node_type: RENodeType) -> Result<RENodeId, RuntimeError> {
        self.kernel.allocate_node_id(node_type)
    }

    fn create_node(&mut self, node_id: RENodeId, re_node: RENodeInit) -> Result<(), RuntimeError> {
        self.kernel.create_node(node_id, re_node)
    }

    fn lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        self.kernel.lock_substate(node_id, offset, flags)
    }

    fn get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        self.kernel.get_lock_info(lock_handle)
    }

    fn drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        self.kernel.drop_lock(lock_handle)
    }

    fn get_ref(&mut self, lock_handle: LockHandle) -> Result<SubstateRef, RuntimeError> {
        self.kernel.get_ref(lock_handle)
    }

    fn get_ref_mut(&mut self, lock_handle: LockHandle) -> Result<SubstateRefMut, RuntimeError> {
        self.kernel.get_ref_mut(lock_handle)
    }
}
