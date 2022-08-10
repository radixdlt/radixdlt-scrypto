use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::core::Receiver;
use scrypto::engine::types::*;
use scrypto::prelude::TypeName;
use scrypto::resource::AccessRule;
use scrypto::values::*;

use crate::engine::node::*;
use crate::engine::*;
use crate::fee::*;
use crate::model::AuthZone;
use crate::wasm::*;

pub trait SystemApi<'s, W, I, C>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    C: FeeReserve,
{
    fn fee_reserve(&mut self) -> &mut C;

    // TODO: possible to consider AuthZone as a RENode?
    fn auth_zone(&mut self, frame_id: usize) -> &mut AuthZone;

    fn invoke_function(
        &mut self,
        type_name: TypeName,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    fn invoke_method(
        &mut self,
        receiver: Receiver,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    // TODO: Convert to substate_borrow
    fn borrow_node(&mut self, node_id: &RENodeId) -> Result<RENodeRef<'_, 's>, FeeReserveError>;

    /// Removes an RENode and all of it's children from the Heap
    fn node_drop(&mut self, node_id: &RENodeId) -> Result<HeapRootRENode, FeeReserveError>;

    /// Creates a new RENode and places it in the Heap
    fn node_create(&mut self, re_node: HeapRENode) -> Result<RENodeId, RuntimeError>;

    /// Moves an RENode from Heap to Store
    fn node_globalize(&mut self, node_id: RENodeId) -> Result<(), RuntimeError>;

    /// Borrow a mutable substate
    fn substate_borrow_mut(
        &mut self,
        substate_id: &SubstateId,
    ) -> Result<NativeSubstateRef, FeeReserveError>;

    /// Return a mutable substate
    fn substate_return_mut(&mut self, val_ref: NativeSubstateRef) -> Result<(), FeeReserveError>;

    // TODO: Convert use substate_borrow interface
    fn substate_read(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError>;
    fn substate_write(
        &mut self,
        substate_id: SubstateId,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError>;
    fn substate_take(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError>;

    fn transaction_hash(&mut self) -> Result<Hash, FeeReserveError>;

    fn generate_uuid(&mut self) -> Result<u128, FeeReserveError>;

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), FeeReserveError>;

    fn check_access_rule(
        &mut self,
        access_rule: AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError>;
}
