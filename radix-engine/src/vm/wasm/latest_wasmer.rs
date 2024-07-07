use crate::blueprints::package::*;
use crate::errors::*;
use crate::vm::wasm::*;
use ::latest_wasmer::*;
use moka::sync::Cache;
use radix_common::constants::*;
use radix_engine_interface::prelude::*;
use std::cell::*;
use std::sync::*;

// region:Wasmer Engine
pub struct WasmerEngine {
    // TODO: Investigate whether making this a global static can provide us with
    // any performance gains. Also, does it have any effect on the memory?
    /// The store that everything will be run against.
    store: Arc<RefCell<Store>>,
    // TODO: Investigate if we would be better off moving to WASMER's FS cache
    // implementation instead of this implementation.
    modules_cache: Cache<CodeHash, Arc<WasmerModule>>,
}

unsafe impl Send for WasmerEngine {}
unsafe impl Sync for WasmerEngine {}

impl WasmerEngine {
    pub fn new(options: WasmerEngineOptions) -> Self {
        let compiler = Singlepass::new();

        let modules_cache = moka::sync::Cache::builder()
            .weigher(|_, _| 1u32)
            .max_capacity(options.max_cache_size as u64)
            .build();

        Self {
            store: Arc::new(RefCell::new(Store::new(compiler))),
            modules_cache,
        }
    }
}

impl WasmEngine for WasmerEngine {
    type WasmInstance = WasmerInstance;

    fn instantiate(&self, code_hash: CodeHash, code: &[u8]) -> Self::WasmInstance {
        match self.modules_cache.get(&code_hash) {
            Some(cached_module) => cached_module.instantiate(self.store.clone()),
            None => {
                let new_module = Arc::new(WasmerModule {
                    module: Module::new(&self.store.borrow(), code)
                        .expect("Failed to parse WASM module"),
                });
                self.modules_cache.insert(code_hash, new_module.clone());
                new_module.instantiate(self.store.clone())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct WasmerEngineOptions {
    max_cache_size: usize,
    // TODO: I think I can the engine to use a different compiler by having it
    // be an argument here and then operate over a `dyn WasmEngine`.
}

impl Default for WasmerEngine {
    fn default() -> Self {
        Self::new(WasmerEngineOptions {
            max_cache_size: WASM_ENGINE_CACHE_SIZE,
        })
    }
}
// endregion:Wasmer Engine

// region:Wasmer Module
pub struct WasmerModule {
    module: Module,
}

impl WasmerModule {
    fn instantiate(&self, store: Arc<RefCell<Store>>) -> WasmerInstance {
        // native functions starts
        pub fn buffer_consume(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            buffer_id: BufferId,
            destination_ptr: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let result = runtime.buffer_consume(buffer_id);
            match result {
                Ok(slice) => {
                    write_memory(&memory, store, destination_ptr, &slice)?;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }

        #[allow(clippy::too_many_arguments)]
        pub fn blueprint_call(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            package_address_ptr: u32,
            package_address_len: u32,
            blueprint_name_ptr: u32,
            blueprint_name_len: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let package_address = read_memory(
                &memory,
                store.as_store_ref(),
                package_address_ptr,
                package_address_len,
            )?;
            let blueprint_name = read_memory(
                &memory,
                store.as_store_ref(),
                blueprint_name_ptr,
                blueprint_name_len,
            )?;
            let ident = read_memory(&memory, store.as_store_ref(), ident_ptr, ident_len)?;
            let args = read_memory(&memory, store.as_store_ref(), args_ptr, args_len)?;

            runtime
                .blueprint_call(package_address, blueprint_name, ident, args)
                .map(|buffer| buffer.0)
        }

        pub fn address_allocate(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            package_address_ptr: u32,
            package_address_len: u32,
            blueprint_name_ptr: u32,
            blueprint_name_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            runtime
                .address_allocate(
                    read_memory(
                        &memory,
                        store.as_store_ref(),
                        package_address_ptr,
                        package_address_len,
                    )?,
                    read_memory(
                        &memory,
                        store.as_store_ref(),
                        blueprint_name_ptr,
                        blueprint_name_len,
                    )?,
                )
                .map(|buffer| buffer.0)
        }

        pub fn address_get_reservation_address(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            node_id_ptr: u32,
            node_id_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            runtime
                .address_get_reservation_address(read_memory(
                    &memory,
                    store.as_store_ref(),
                    node_id_ptr,
                    node_id_len,
                )?)
                .map(|buffer| buffer.0)
        }

        pub fn object_call(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            receiver_ptr: u32,
            receiver_len: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let receiver = read_memory(&memory, store.as_store_ref(), receiver_ptr, receiver_len)?;
            let ident = read_memory(&memory, store.as_store_ref(), ident_ptr, ident_len)?;
            let args = read_memory(&memory, store.as_store_ref(), args_ptr, args_len)?;

            runtime
                .object_call(receiver, ident, args)
                .map(|buffer| buffer.0)
        }

        #[allow(clippy::too_many_arguments)]
        pub fn object_call_module(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            receiver_ptr: u32,
            receiver_len: u32,
            module: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let receiver = read_memory(&memory, store.as_store_ref(), receiver_ptr, receiver_len)?;
            let ident = read_memory(&memory, store.as_store_ref(), ident_ptr, ident_len)?;
            let args = read_memory(&memory, store.as_store_ref(), args_ptr, args_len)?;

            runtime
                .object_call_module(receiver, module, ident, args)
                .map(|buffer| buffer.0)
        }

        pub fn object_call_direct(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            receiver_ptr: u32,
            receiver_len: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let receiver = read_memory(&memory, store.as_store_ref(), receiver_ptr, receiver_len)?;
            let ident = read_memory(&memory, store.as_store_ref(), ident_ptr, ident_len)?;
            let args = read_memory(&memory, store.as_store_ref(), args_ptr, args_len)?;

            runtime
                .object_call_direct(receiver, ident, args)
                .map(|buffer| buffer.0)
        }

        pub fn object_new(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            blueprint_name_ptr: u32,
            blueprint_name_len: u32,
            object_states_ptr: u32,
            object_states_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            runtime
                .object_new(
                    read_memory(
                        &memory,
                        store.as_store_ref(),
                        blueprint_name_ptr,
                        blueprint_name_len,
                    )?,
                    read_memory(
                        &memory,
                        store.as_store_ref(),
                        object_states_ptr,
                        object_states_len,
                    )?,
                )
                .map(|buffer| buffer.0)
        }

        pub fn object_globalize(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            obj_ptr: u32,
            obj_len: u32,
            modules_ptr: u32,
            modules_len: u32,
            address_ptr: u32,
            address_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            runtime
                .globalize_object(
                    read_memory(&memory, store.as_store_ref(), obj_ptr, obj_len)?,
                    read_memory(&memory, store.as_store_ref(), modules_ptr, modules_len)?,
                    read_memory(&memory, store.as_store_ref(), address_ptr, address_len)?,
                )
                .map(|buffer| buffer.0)
        }

        pub fn object_instance_of(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            component_id_ptr: u32,
            component_id_len: u32,
            package_address_ptr: u32,
            package_address_len: u32,
            blueprint_name_ptr: u32,
            blueprint_name_len: u32,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            runtime.instance_of(
                read_memory(
                    &memory,
                    store.as_store_ref(),
                    component_id_ptr,
                    component_id_len,
                )?,
                read_memory(
                    &memory,
                    store.as_store_ref(),
                    package_address_ptr,
                    package_address_len,
                )?,
                read_memory(
                    &memory,
                    store.as_store_ref(),
                    blueprint_name_ptr,
                    blueprint_name_len,
                )?,
            )
        }

        pub fn object_get_blueprint_id(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            component_id_ptr: u32,
            component_id_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            runtime
                .blueprint_id(read_memory(
                    &memory,
                    store.as_store_ref(),
                    component_id_ptr,
                    component_id_len,
                )?)
                .map(|buffer| buffer.0)
        }

        pub fn object_get_outer_object(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            component_id_ptr: u32,
            component_id_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            runtime
                .get_outer_object(read_memory(
                    &memory,
                    store.as_store_ref(),
                    component_id_ptr,
                    component_id_len,
                )?)
                .map(|buffer| buffer.0)
        }

        pub fn key_value_store_new(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            schema_id_ptr: u32,
            schema_id_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            runtime
                .key_value_store_new(read_memory(
                    &memory,
                    store.as_store_ref(),
                    schema_id_ptr,
                    schema_id_len,
                )?)
                .map(|buffer| buffer.0)
        }

        pub fn key_value_store_open_entry(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            node_id_ptr: u32,
            node_id_len: u32,
            key_ptr: u32,
            key_len: u32,
            flags: u32,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            runtime.key_value_store_open_entry(
                read_memory(&memory, store.as_store_ref(), node_id_ptr, node_id_len)?,
                read_memory(&memory, store.as_store_ref(), key_ptr, key_len)?,
                flags,
            )
        }

        pub fn key_value_store_remove_entry(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            node_id_ptr: u32,
            node_id_len: u32,
            key_ptr: u32,
            key_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            runtime
                .key_value_store_remove_entry(
                    read_memory(&memory, store.as_store_ref(), node_id_ptr, node_id_len)?,
                    read_memory(&memory, store.as_store_ref(), key_ptr, key_len)?,
                )
                .map(|buffer| buffer.0)
        }

        pub fn key_value_entry_read(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            handle: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.key_value_entry_get(handle).map(|buffer| buffer.0)
        }

        pub fn key_value_entry_write(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            handle: u32,
            data_ptr: u32,
            data_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let data = read_memory(&memory, store.as_store_ref(), data_ptr, data_len)?;

            runtime.key_value_entry_set(handle, data)
        }

        pub fn key_value_entry_remove(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            handle: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime
                .key_value_entry_remove(handle)
                .map(|buffer| buffer.0)
        }

        pub fn key_value_entry_close(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            handle: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.key_value_entry_close(handle)
        }

        pub fn field_entry_read(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            handle: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.field_entry_read(handle).map(|buffer| buffer.0)
        }

        pub fn field_entry_write(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            handle: u32,
            data_ptr: u32,
            data_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let data = read_memory(&memory, store.as_store_ref(), data_ptr, data_len)?;

            runtime.field_entry_write(handle, data)
        }

        pub fn field_entry_close(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            handle: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.field_entry_close(handle)
        }

        pub fn actor_open_field(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            object_handle: u32,
            field: u8,
            flags: u32,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.actor_open_field(object_handle, field, flags)
        }

        pub fn actor_get_node_id(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            actor_ref_handle: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime
                .actor_get_node_id(actor_ref_handle)
                .map(|buffer| buffer.0)
        }

        pub fn actor_get_package_address(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.actor_get_package_address().map(|buffer| buffer.0)
        }

        pub fn actor_get_blueprint_name(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.actor_get_blueprint_name().map(|buffer| buffer.0)
        }

        fn actor_emit_event(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            event_name_ptr: u32,
            event_name_len: u32,
            event_data_ptr: u32,
            event_data_len: u32,
            flags: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let event_name = read_memory(
                &memory,
                store.as_store_ref(),
                event_name_ptr,
                event_name_len,
            )?;
            let event_data = read_memory(
                &memory,
                store.as_store_ref(),
                event_data_ptr,
                event_data_len,
            )?;
            let event_flags = EventFlags::from_bits(flags).ok_or(InvokeError::SelfError(
                WasmRuntimeError::InvalidEventFlags(flags),
            ))?;

            runtime.actor_emit_event(event_name, event_data, event_flags)
        }

        pub fn costing_get_execution_cost_unit_limit(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.costing_get_execution_cost_unit_limit()
        }

        pub fn costing_get_execution_cost_unit_price(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime
                .costing_get_execution_cost_unit_price()
                .map(|buffer| buffer.0)
        }

        pub fn costing_get_finalization_cost_unit_limit(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.costing_get_finalization_cost_unit_limit()
        }

        pub fn costing_get_finalization_cost_unit_price(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime
                .costing_get_finalization_cost_unit_price()
                .map(|buffer| buffer.0)
        }

        pub fn costing_get_tip_percentage(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.costing_get_tip_percentage()
        }

        pub fn costing_get_fee_balance(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.costing_get_fee_balance().map(|buffer| buffer.0)
        }

        pub fn costing_get_usd_price(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.costing_get_usd_price().map(|buffer| buffer.0)
        }

        fn consume_wasm_execution_units(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            n: u64,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();
            // TODO: wasm-instrument uses u64 for cost units. We need to decide
            // if we want to move from u32 to u64 as well.
            runtime.consume_wasm_execution_units(n as u32)
        }

        fn sys_log(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            level_ptr: u32,
            level_len: u32,
            message_ptr: u32,
            message_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let level = read_memory(&memory, store.as_store_ref(), level_ptr, level_len)?;
            let message = read_memory(&memory, store.as_store_ref(), message_ptr, message_len)?;

            runtime.sys_log(level, message)
        }

        fn sys_bech32_encode_address(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            address_ptr: u32,
            address_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let address = read_memory(&memory, store.as_store_ref(), address_ptr, address_len)?;

            runtime
                .sys_bech32_encode_address(address)
                .map(|buffer| buffer.0)
        }

        fn sys_panic(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            message_ptr: u32,
            message_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let message = read_memory(&memory, store.as_store_ref(), message_ptr, message_len)?;

            runtime.sys_panic(message)
        }

        pub fn sys_get_transaction_hash(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.sys_get_transaction_hash().map(|buffer| buffer.0)
        }

        pub fn sys_generate_ruid(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, _) = env.data_and_store_mut();
            let (_, runtime) = data.grab_runtime();

            runtime.sys_generate_ruid().map(|buffer| buffer.0)
        }

        pub fn bls12381_v1_verify(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            message_ptr: u32,
            message_len: u32,
            public_key_ptr: u32,
            public_key_len: u32,
            signature_ptr: u32,
            signature_len: u32,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let message = read_memory(&memory, store.as_store_ref(), message_ptr, message_len)?;

            let public_key = read_memory(
                &memory,
                store.as_store_ref(),
                public_key_ptr,
                public_key_len,
            )?;
            let signature =
                read_memory(&memory, store.as_store_ref(), signature_ptr, signature_len)?;

            runtime.crypto_utils_bls12381_v1_verify(message, public_key, signature)
        }

        pub fn bls12381_v1_aggregate_verify(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            pub_keys_and_msgs_ptr: u32,
            pub_keys_and_msgs_len: u32,
            signature_ptr: u32,
            signature_len: u32,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let pub_keys_and_msgs = read_memory(
                &memory,
                store.as_store_ref(),
                pub_keys_and_msgs_ptr,
                pub_keys_and_msgs_len,
            )?;
            let signature =
                read_memory(&memory, store.as_store_ref(), signature_ptr, signature_len)?;

            runtime.crypto_utils_bls12381_v1_aggregate_verify(pub_keys_and_msgs, signature)
        }

        pub fn bls12381_v1_fast_aggregate_verify(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            message_ptr: u32,
            message_len: u32,
            public_keys_ptr: u32,
            public_keys_len: u32,
            signature_ptr: u32,
            signature_len: u32,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let message = read_memory(&memory, store.as_store_ref(), message_ptr, message_len)?;

            let public_keys = read_memory(
                &memory,
                store.as_store_ref(),
                public_keys_ptr,
                public_keys_len,
            )?;
            let signature =
                read_memory(&memory, store.as_store_ref(), signature_ptr, signature_len)?;

            runtime.crypto_utils_bls12381_v1_fast_aggregate_verify(message, public_keys, signature)
        }

        pub fn bls12381_g2_signature_aggregate(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            signatures_ptr: u32,
            signatures_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let signatures = read_memory(
                &memory,
                store.as_store_ref(),
                signatures_ptr,
                signatures_len,
            )?;

            runtime
                .crypto_utils_bls12381_g2_signature_aggregate(signatures)
                .map(|buffer| buffer.0)
        }

        pub fn keccak256_hash(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            data_ptr: u32,
            data_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, runtime) = data.grab_runtime();

            let data = read_memory(&memory, store.as_store_ref(), data_ptr, data_len)?;

            runtime
                .crypto_utils_keccak256_hash(data)
                .map(|buffer| buffer.0)
        }

        #[cfg(feature = "crate_tests")]
        pub fn host_read_memory(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            memory_ptr: u32,
            data_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (data, store) = env.data_and_store_mut();
            let (memory, _) = data.grab_runtime();

            let _ = read_memory(&memory, store.as_store_ref(), memory_ptr, data_len)?;

            Ok(())
        }

        #[cfg(feature = "crate_tests")]
        pub fn host_write_memory(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
            memory_ptr: u32,
            data_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            // - generate some random data of of given length data_len
            // - attempt to write this data into given memory offset memory_ptr
            let (data, store) = env.data_and_store_mut();
            let (memory, _) = data.grab_runtime();
            let data = vec![0u8; data_len as usize];

            write_memory(&memory, store, memory_ptr, &data)?;

            Ok(())
        }

        #[cfg(feature = "crate_tests")]
        pub fn host_check_memory_is_clean(
            mut env: FunctionEnvMut<WasmerInstanceEnv>,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            // - generate some random data of of given length data_len
            // - attempt to write this data into given memory offset memory_ptr
            let (data, store) = env.data_and_store_mut();
            let (memory, _) = data.grab_runtime();
            let view = memory.view(&store);
            let memory_slice = unsafe { view.data_unchecked() };

            println!(
                "memory len = {:?} : {:02x?}",
                memory_slice.len(),
                memory_slice
            );
            let clean = !memory_slice.iter().any(|&x| x != 0x0);

            Ok(clean as u64)
        }
        // native functions ends

        // Creating the environment - before the instance is created there is
        // no memory and therefore it is set to `None`. This will be set after
        // the instantiation.
        let runtime_pointer = Arc::new(Mutex::new(0));
        let env = FunctionEnv::new(
            &mut store.borrow_mut(),
            WasmerInstanceEnv {
                memory: None,
                runtime_ptr: runtime_pointer.clone(),
            },
        );

        // Defining the imports.
        let import_object = imports! {
            MODULE_ENV_NAME => {
                BLUEPRINT_CALL_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, blueprint_call),
                ADDRESS_ALLOCATE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, address_allocate),
                ADDRESS_GET_RESERVATION_ADDRESS_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, address_get_reservation_address),
                OBJECT_NEW_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, object_new),
                OBJECT_GLOBALIZE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, object_globalize),
                OBJECT_INSTANCE_OF_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, object_instance_of),
                OBJECT_GET_BLUEPRINT_ID_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, object_get_blueprint_id),
                OBJECT_GET_OUTER_OBJECT_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, object_get_outer_object),
                OBJECT_CALL_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, object_call),
                OBJECT_CALL_MODULE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, object_call_module),
                OBJECT_CALL_DIRECT_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, object_call_direct),
                KEY_VALUE_STORE_NEW_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, key_value_store_new),
                KEY_VALUE_STORE_OPEN_ENTRY_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, key_value_store_open_entry),
                KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, key_value_store_remove_entry),
                KEY_VALUE_ENTRY_READ_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, key_value_entry_read),
                KEY_VALUE_ENTRY_WRITE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, key_value_entry_write),
                KEY_VALUE_ENTRY_REMOVE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, key_value_entry_remove),
                KEY_VALUE_ENTRY_CLOSE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, key_value_entry_close),
                FIELD_ENTRY_READ_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, field_entry_read),
                FIELD_ENTRY_WRITE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, field_entry_write),
                FIELD_ENTRY_CLOSE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, field_entry_close),
                ACTOR_OPEN_FIELD_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, actor_open_field),
                ACTOR_GET_OBJECT_ID_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, actor_get_node_id),
                ACTOR_GET_PACKAGE_ADDRESS_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, actor_get_package_address),
                ACTOR_GET_BLUEPRINT_NAME_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, actor_get_blueprint_name),
                ACTOR_EMIT_EVENT_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, actor_emit_event),
                COSTING_CONSUME_WASM_EXECUTION_UNITS_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, consume_wasm_execution_units),
                COSTING_GET_EXECUTION_COST_UNIT_LIMIT_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, costing_get_execution_cost_unit_limit),
                COSTING_GET_EXECUTION_COST_UNIT_PRICE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, costing_get_execution_cost_unit_price),
                COSTING_GET_FINALIZATION_COST_UNIT_LIMIT_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, costing_get_finalization_cost_unit_limit),
                COSTING_GET_FINALIZATION_COST_UNIT_PRICE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, costing_get_finalization_cost_unit_price),
                COSTING_GET_USD_PRICE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, costing_get_usd_price),
                COSTING_GET_TIP_PERCENTAGE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, costing_get_tip_percentage),
                COSTING_GET_FEE_BALANCE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, costing_get_fee_balance),
                SYS_LOG_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, sys_log),
                SYS_BECH32_ENCODE_ADDRESS_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, sys_bech32_encode_address),
                SYS_PANIC_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, sys_panic),
                SYS_GET_TRANSACTION_HASH_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, sys_get_transaction_hash),
                SYS_GENERATE_RUID_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, sys_generate_ruid),
                BUFFER_CONSUME_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, buffer_consume),
                CRYPTO_UTILS_BLS12381_V1_VERIFY_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, bls12381_v1_verify),
                CRYPTO_UTILS_BLS12381_V1_AGGREGATE_VERIFY_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, bls12381_v1_aggregate_verify),
                CRYPTO_UTILS_BLS12381_V1_FAST_AGGREGATE_VERIFY_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, bls12381_v1_fast_aggregate_verify),
                CRYPTO_UTILS_BLS12381_G2_SIGNATURE_AGGREGATE_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, bls12381_g2_signature_aggregate),
                CRYPTO_UTILS_KECCAK256_HASH_FUNCTION_NAME => Function::new_typed_with_env(&mut store.borrow_mut(), &env, keccak256_hash),
            }
        };

        #[cfg(feature = "crate_tests")]
        let import_object = {
            let mut exports = import_object
                .get_namespace_exports(MODULE_ENV_NAME)
                .unwrap();

            exports.insert(
                "test_host_read_memory",
                Function::new_typed_with_env(&mut store.borrow_mut(), &env, host_read_memory),
            );
            exports.insert(
                "test_host_write_memory",
                Function::new_typed_with_env(&mut store.borrow_mut(), &env, host_write_memory),
            );
            exports.insert(
                "test_host_check_memory_is_clean",
                Function::new_typed_with_env(
                    &mut store.borrow_mut(),
                    &env,
                    host_check_memory_is_clean,
                ),
            );
            let mut import_object = Imports::new();
            import_object.register_namespace(MODULE_ENV_NAME, exports);
            import_object
        };

        // Creating a WASM Instance.
        let instance = Instance::new(&mut store.borrow_mut(), &self.module, &import_object)
            .expect("Failed to instantiate module");

        // Getting the memory from the instance and setting it in the env.
        // TODO: Is there a guarantee that the `expect` below never happens? Is
        // another layer handling it?
        env.as_mut(&mut store.borrow_mut()).memory = Some(
            instance
                .exports
                .get::<Memory>(EXPORT_MEMORY)
                .expect("No memory found for instance")
                .clone(),
        );

        WasmerInstance {
            instance,
            runtime_ptr: runtime_pointer,
            store: store.clone(),
        }
    }
}
// endregion:Wasmer Module

// region:Wasmer Instance
pub struct WasmerInstance {
    instance: Instance,
    // TODO: Investigate whether making this a global static can provide us with
    // any performance gains. Also, does it have any effect on the memory?
    /// The store that everything will be run against.
    store: Arc<RefCell<Store>>,
    // TODO: More of a coding style, but it would be nice if we can switch this
    // into a typed pointer of some sort. Maybe a `Rc<RefMut<dyn WasmRuntime>>`
    // is best here?
    /// A pointer to the WasmRuntime.
    runtime_ptr: Arc<Mutex<usize>>,
}

impl WasmInstance for WasmerInstance {
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: Vec<Buffer>,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
        // Setup the runtime pointer
        *self.runtime_ptr.lock().expect("Runtime ptr unavailable") = runtime as *mut _ as usize;

        // Prepare the inputs or the arguments that the WASM function will be
        // invoked with.
        let input = args
            .iter()
            .map(Buffer::as_i64)
            .map(Value::I64)
            .collect::<Vec<_>>();
        let output = self
            .instance
            .exports
            .get_function(func_name)
            .map_err(|_| {
                InvokeError::SelfError(WasmRuntimeError::UnknownExport(func_name.to_string()))
            })?
            .call(&mut self.store.borrow_mut(), &input)
            .map_err(|error| {
                let e_str = format!("{:?}", error);
                match error.downcast::<InvokeError<WasmRuntimeError>>() {
                    Ok(e) => e,
                    _ => InvokeError::SelfError(WasmRuntimeError::ExecutionError(e_str)),
                }
            });

        // Scrypto WASM invocations return a slice and the returned data can
        // then be dereferenced, this step dereferences the data from the slice.
        match output {
            Ok(data) => {
                if let Some(v) = data.as_ref().first().and_then(Value::i64) {
                    read_slice(
                        self.instance
                            .exports
                            .get_memory(EXPORT_MEMORY)
                            .map_err(|_| WasmRuntimeError::MemoryAccessError)?,
                        &self.store.as_ref().borrow(),
                        Slice::transmute_i64(v),
                    )
                    .map_err(InvokeError::SelfError)
                } else {
                    Err(InvokeError::SelfError(WasmRuntimeError::InvalidWasmPointer))
                }
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Clone)]
pub struct WasmerInstanceEnv {
    memory: Option<Memory>,
    runtime_ptr: Arc<Mutex<usize>>,
}

impl WasmerInstanceEnv {
    /// Note: can panic if:
    /// 1. An issue with the runtime pointer.
    /// 2. If the memory is not actually initialized.
    pub fn grab_runtime(&self) -> (Memory, &mut Box<dyn WasmRuntime>) {
        let memory = self.memory.as_ref().expect("Memory is not initialized");
        let ptr = self.runtime_ptr.lock().expect("Runtime ptr unavailable");
        let runtime = unsafe { &mut *(*ptr as *mut Box<dyn WasmRuntime>) };
        (memory.clone(), runtime)
    }
}
// endregion:Wasmer Instance

// region:Memory Functions
pub fn read_memory(
    memory: &Memory,
    store: impl AsStoreRef,
    ptr: u32,
    len: u32,
) -> Result<Vec<u8>, WasmRuntimeError> {
    let ptr = ptr as usize;
    let len = len as usize;

    let view = memory.view(&store);
    let memory_slice = unsafe { view.data_unchecked() };
    let memory_size = memory_slice.len();
    if ptr > memory_size || ptr + len > memory_size {
        return Err(WasmRuntimeError::MemoryAccessError);
    }

    Ok(memory_slice[ptr..ptr + len].to_vec())
}

pub fn write_memory(
    memory: &Memory,
    store: impl AsStoreMut,
    ptr: u32,
    data: &[u8],
) -> Result<(), WasmRuntimeError> {
    let ptr = ptr as usize;
    let len = data.len();

    let view = memory.view(&store);
    let memory_slice = unsafe { view.data_unchecked_mut() };
    let memory_size = memory_slice.len();
    if ptr > memory_size || ptr + len > memory_size {
        return Err(WasmRuntimeError::MemoryAccessError);
    }

    memory_slice[ptr..ptr + data.len()].copy_from_slice(data);
    Ok(())
}

pub fn read_slice(
    memory: &Memory,
    store: impl AsStoreRef,
    v: Slice,
) -> Result<Vec<u8>, WasmRuntimeError> {
    let ptr = v.ptr();
    let len = v.len();

    read_memory(memory, store, ptr, len)
}

pub fn get_memory_size(memory: &Memory, store: impl AsStoreRef) -> Result<usize, WasmRuntimeError> {
    let view = memory.view(&store);
    let memory_slice = unsafe { view.data_unchecked() };

    Ok(memory_slice.len())
}
// endregion:memory-functions
