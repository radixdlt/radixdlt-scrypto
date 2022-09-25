use crate::engine::node::*;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::AuthZone;
use crate::model::Resource;
use crate::types::*;
use crate::wasm::*;

pub trait SystemApi<'s, W, I, R>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    R: FeeReserve,
{
    // TODO: possible to consider AuthZone as a RENode?
    fn auth_zone(&mut self, frame_id: usize) -> &mut AuthZone;

    fn consume_cost_units(&mut self, units: u32) -> Result<(), RuntimeError>;

    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError>;

    fn invoke_function(
        &mut self,
        fn_identifier: FnIdentifier,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    fn invoke_method(
        &mut self,
        receiver: Receiver,
        function: FnIdentifier,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    // TODO: Convert to substate_borrow
    fn borrow_node(&mut self, node_id: &RENodeId) -> Result<RENodeRef<'_, 's, R>, RuntimeError>;

    fn borrow_node_mut(
        &mut self,
        node_id: &RENodeId,
    ) -> Result<RENodeRefMut<'_, 's, R>, RuntimeError>;

    /// Removes an RENode and all of it's children from the Heap
    fn node_drop(&mut self, node_id: &RENodeId) -> Result<HeapRootRENode, RuntimeError>;

    /// Creates a new RENode and places it in the Heap
    fn node_create(&mut self, re_node: HeapRENode) -> Result<RENodeId, RuntimeError>;

    /// Moves an RENode from Heap to Store
    fn node_globalize(&mut self, node_id: RENodeId) -> Result<(), RuntimeError>;

    // TODO: Convert use substate_borrow interface
    fn substate_read(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError>;

    fn substate_write(
        &mut self,
        substate_id: SubstateId,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError>;

    fn substate_take(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError>;

    fn transaction_hash(&mut self) -> Result<Hash, RuntimeError>;

    fn read_blob(&mut self, blob_hash: &Hash) -> Result<&[u8], RuntimeError>;

    fn generate_uuid(&mut self) -> Result<u128, RuntimeError>;

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError>;

    fn check_access_rule(
        &mut self,
        access_rule: AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError>;
}
