use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::core::SNodeRef;
use scrypto::engine::types::*;
use scrypto::prelude::HashSet;
use scrypto::resource::AccessRule;
use scrypto::values::*;

use crate::engine::call_frame::{DataInstruction, SubstateAddress};
use crate::engine::*;
use crate::fee::*;
use crate::model::*;
use crate::wasm::*;

pub trait SystemApi<'borrowed, W, I>
where
    W: WasmEngine<I>,
    I: WasmInstance,
{
    fn wasm_engine(&mut self) -> &mut W;

    fn wasm_instrumenter(&mut self) -> &mut WasmInstrumenter;

    fn cost_unit_counter(&mut self) -> &mut CostUnitCounter;

    fn fee_table(&self) -> &FeeTable;

    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    fn native_globalize(&mut self, value_id: &ValueId);

    fn borrow_global_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<&ResourceManager, RuntimeError>;

    fn borrow_native_value(&mut self, value_id: &ValueId) -> RENativeValueRef<'borrowed>;
    fn return_native_value(&mut self, value_id: ValueId, val_ref: RENativeValueRef<'borrowed>);
    fn take_native_value(&mut self, value_id: &ValueId) -> REValue;

    fn native_create<V: Into<(REValue, HashSet<ValueId>)>>(&mut self, value: V) -> ValueId;
    fn create_resource(&mut self, resource_manager: ResourceManager) -> ResourceAddress;
    fn create_local_component(
        &mut self,
        component: Component,
    ) -> Result<ComponentAddress, RuntimeError>;

    fn data(
        &mut self,
        address: SubstateAddress,
        instruction: DataInstruction,
    ) -> Result<ScryptoValue, RuntimeError>;
    fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<NonFungible>;
    fn set_non_fungible(
        &mut self,
        non_fungible_address: NonFungibleAddress,
        non_fungible: Option<NonFungible>,
    );

    fn get_epoch(&mut self) -> u64;

    fn get_transaction_hash(&mut self) -> Hash;

    fn generate_uuid(&mut self) -> u128;

    fn user_log(&mut self, level: Level, message: String);

    fn check_access_rule(
        &mut self,
        access_rule: AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError>;
}
