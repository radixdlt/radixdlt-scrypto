use crate::engine::node::*;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::{Resource, SubstateRef, SubstateRefMut};
use crate::types::*;
use bitflags::bitflags;
use scrypto::core::FnIdent;

bitflags! {
    #[derive(Encode, Decode, TypeId)]
    pub struct LockFlags: u32 {
        /// Allows the locked substate to be mutated
        const MUTABLE = 0b00000001;
        /// Checks that the substate locked is unmodified from the beginning of
        /// the transaction. This is used mainly for locking fees in vaults which
        /// requires this in order to be able to support rollbacks
        const UNMODIFIED_BASE = 0b00000010;
        /// Forces a write of a substate even on a transaction failure
        /// Currently used for vault fees.
        const FORCE_WRITE = 0b00000100;
    }
}

impl LockFlags {
    pub fn read_only() -> Self {
        LockFlags::empty()
    }
}

pub trait SystemApi<'s, R>
where
    R: FeeReserve,
{
    fn execute_in_mode<X, RTN, E>(
        &mut self,
        execution_mode: ExecutionMode,
        execute: X,
    ) -> Result<RTN, RuntimeError>
    where
        RuntimeError: From<E>,
        X: FnOnce(&mut Self) -> Result<RTN, E>;

    fn consume_cost_units(&mut self, units: u32) -> Result<(), RuntimeError>;

    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError>;

    fn get_actor(&self) -> &REActor;

    fn invoke(
        &mut self,
        fn_ident: FnIdent,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    /// Retrieves all nodes referenceable by the current frame
    fn get_visible_node_ids(&mut self) -> Result<Vec<RENodeId>, RuntimeError>;

    /// Removes an RENode and all of it's children from the Heap
    fn node_drop(&mut self, node_id: RENodeId) -> Result<HeapRootRENode, RuntimeError>;

    /// Creates a new RENode and places it in the Heap
    fn create_node(&mut self, re_node: HeapRENode) -> Result<RENodeId, RuntimeError>;

    /// Moves an RENode from Heap to Store
    fn node_globalize(&mut self, node_id: RENodeId) -> Result<GlobalAddress, RuntimeError>;

    /// Locks a visible substate
    fn lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError>;

    /// Drops a lock
    fn drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError>;

    /// Get a non-mutable reference to a locked substate
    fn get_ref(&mut self, lock_handle: LockHandle) -> Result<SubstateRef, RuntimeError>;

    /// Get a mutable reference to a locked substate
    fn get_ref_mut(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<SubstateRefMut<'_, 's, R>, RuntimeError>;

    fn read_transaction_hash(&mut self) -> Result<Hash, RuntimeError>;

    fn read_blob(&mut self, blob_hash: &Hash) -> Result<&[u8], RuntimeError>;

    fn generate_uuid(&mut self) -> Result<u128, RuntimeError>;

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError>;
}
