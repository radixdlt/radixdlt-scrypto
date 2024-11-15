use crate::errors::InvokeError;
use crate::internal_prelude::*;
use crate::vm::wasm::errors::*;
use radix_engine_interface::api::actor_api::EventFlags;
use radix_engine_interface::api::ActorRefHandle;
use radix_engine_interface::blueprints::package::CodeHash;
use sbor::rust::boxed::Box;
use sbor::rust::vec::Vec;

/// Represents the runtime that can be invoked by Scrypto modules.
pub trait WasmRuntime {
    fn allocate_buffer(&mut self, buffer: Vec<u8>)
        -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn buffer_consume(
        &mut self,
        buffer_id: BufferId,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>>;

    fn object_call(
        &mut self,
        receiver: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn object_call_module(
        &mut self,
        receiver: Vec<u8>,
        module_id: u32,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn object_call_direct(
        &mut self,
        receiver: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn blueprint_call(
        &mut self,
        package_address: Vec<u8>,
        blueprint_name: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn object_new(
        &mut self,
        blueprint_name: Vec<u8>,
        object_states: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn address_allocate(
        &mut self,
        package_address: Vec<u8>,
        blueprint_name: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn address_get_reservation_address(
        &mut self,
        node_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn globalize_object(
        &mut self,
        node_id: Vec<u8>,
        modules: Vec<u8>,
        address: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn key_value_store_new(
        &mut self,
        schema: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn key_value_store_open_entry(
        &mut self,
        node_id: Vec<u8>,
        key: Vec<u8>,
        flags: u32,
    ) -> Result<SubstateHandle, InvokeError<WasmRuntimeError>>;

    fn key_value_entry_get(&mut self, handle: u32)
        -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn key_value_entry_set(
        &mut self,
        handle: u32,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn key_value_entry_remove(
        &mut self,
        handle: u32,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn key_value_entry_close(&mut self, handle: u32) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn key_value_store_remove_entry(
        &mut self,
        node_id: Vec<u8>,
        key: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn instance_of(
        &mut self,
        object_id: Vec<u8>,
        package_address: Vec<u8>,
        blueprint_name: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>>;

    fn blueprint_id(&mut self, object_id: Vec<u8>)
        -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn get_outer_object(
        &mut self,
        component_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn actor_open_field(
        &mut self,
        object_handle: u32,
        field: u8,
        flags: u32,
    ) -> Result<SubstateHandle, InvokeError<WasmRuntimeError>>;

    fn field_entry_read(
        &mut self,
        handle: SubstateHandle,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn field_entry_write(
        &mut self,
        handle: SubstateHandle,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn field_entry_close(
        &mut self,
        handle: SubstateHandle,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn actor_get_node_id(
        &mut self,
        actor_ref_handle: ActorRefHandle,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn actor_get_package_address(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn actor_get_blueprint_name(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn consume_wasm_execution_units(&mut self, n: u32)
        -> Result<(), InvokeError<WasmRuntimeError>>;

    fn costing_get_execution_cost_unit_limit(
        &mut self,
    ) -> Result<u32, InvokeError<WasmRuntimeError>>;

    fn costing_get_execution_cost_unit_price(
        &mut self,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn costing_get_finalization_cost_unit_limit(
        &mut self,
    ) -> Result<u32, InvokeError<WasmRuntimeError>>;

    fn costing_get_finalization_cost_unit_price(
        &mut self,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn costing_get_usd_price(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn costing_get_tip_percentage(&mut self) -> Result<u32, InvokeError<WasmRuntimeError>>;

    fn costing_get_fee_balance(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn actor_emit_event(
        &mut self,
        event_name: Vec<u8>,
        event_payload: Vec<u8>,
        event_flags: EventFlags,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn sys_log(
        &mut self,
        level: Vec<u8>,
        message: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn sys_bech32_encode_address(
        &mut self,
        address: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn sys_get_transaction_hash(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn sys_generate_ruid(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn sys_panic(&mut self, message: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn crypto_utils_bls12381_v1_verify(
        &mut self,
        message: Vec<u8>,
        public_key: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>>;

    fn crypto_utils_bls12381_v1_aggregate_verify(
        &mut self,
        pub_keys_and_msgs: Vec<u8>,
        signatures: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>>;

    fn crypto_utils_bls12381_v1_fast_aggregate_verify(
        &mut self,
        message: Vec<u8>,
        public_keys: Vec<u8>,
        signatures: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>>;

    fn crypto_utils_bls12381_g2_signature_aggregate(
        &mut self,
        signatures: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn crypto_utils_keccak256_hash(
        &mut self,
        data: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn crypto_utils_blake2b_256_hash(
        &mut self,
        data: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn crypto_utils_ed25519_verify(
        &mut self,
        message: Vec<u8>,
        public_key: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>>;

    fn crypto_utils_secp256k1_ecdsa_verify(
        &mut self,
        message: Vec<u8>,
        public_key: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>>;

    fn crypto_utils_secp256k1_ecdsa_verify_and_key_recover(
        &mut self,
        message: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn crypto_utils_secp256k1_ecdsa_verify_and_key_recover_uncompressed(
        &mut self,
        message: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;
}

/// Represents an instantiated, invocable Scrypto module.
pub trait WasmInstance {
    /// Invokes an export defined in this module.
    ///
    /// The expected signature is as follows:
    /// - The input is a list of U64, each of which represents a `(BufferId, BufferLen)`.
    /// - The return data is U64, which represents a `(SlicePtr, SliceLen)`.
    ///
    /// The return data is copied into a `Vec<u8>`.
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: Vec<Buffer>,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>>;
}

/// A Scrypto WASM engine validates, instruments and runs Scrypto modules.
pub trait WasmEngine {
    type WasmInstance: WasmInstance;

    /// Instantiate a Scrypto module.
    ///
    /// The code must have been validated and instrumented!
    fn instantiate(&self, code_hash: CodeHash, instrumented_code: &[u8]) -> Self::WasmInstance;
}
