use crate::engine::node::*;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::{Resource, SubstateRef, SubstateRefMut};
use crate::types::*;
use crate::wasm::*;
use scrypto::core::FnIdent;

pub trait SystemApi<'s, W, I, R>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    R: FeeReserve,
{
    fn consume_cost_units(&mut self, units: u32) -> Result<(), RuntimeError>;

    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError>;

    fn invoke(
        &mut self,
        function_identifier: FnIdent,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    /// Retrieves all nodes owned by the current frame
    fn get_owned_node_ids(&mut self) -> Result<Vec<RENodeId>, RuntimeError>;

    fn borrow_node(&mut self, node_id: &RENodeId) -> Result<RENodeRef<'_, 's, R>, RuntimeError>;

    fn borrow_node_mut(
        &mut self,
        node_id: &RENodeId,
    ) -> Result<RENodeRefMut<'_, 's, R>, RuntimeError>;

    /// Removes an RENode and all of it's children from the Heap
    fn node_drop(&mut self, node_id: RENodeId) -> Result<HeapRootRENode, RuntimeError>;

    /// Creates a new RENode and places it in the Heap
    fn node_create(&mut self, re_node: HeapRENode) -> Result<RENodeId, RuntimeError>;

    /// Moves an RENode from Heap to Store
    fn node_globalize(&mut self, node_id: RENodeId) -> Result<GlobalAddress, RuntimeError>;

    fn lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<LockHandle, RuntimeError>;
    fn drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError>;

    fn borrow(&mut self, lock_handle: LockHandle) -> Result<SubstateRef, RuntimeError>;
    fn get_mut(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<SubstateRefMut<'_, 's, R>, RuntimeError>;

    fn read_transaction_hash(&mut self) -> Result<Hash, RuntimeError>;

    fn read_blob(&mut self, blob_hash: &Hash) -> Result<&[u8], RuntimeError>;

    fn generate_uuid(&mut self) -> Result<u128, RuntimeError>;

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError>;
}
