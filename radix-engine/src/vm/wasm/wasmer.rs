use crate::errors::InvokeError;
use crate::types::*;
#[cfg(feature = "coverage")]
use crate::utils::save_coverage_data;
use crate::vm::wasm::constants::*;
use crate::vm::wasm::errors::*;
use crate::vm::wasm::traits::*;
use radix_engine_interface::api::actor_api::EventFlags;
use radix_engine_interface::blueprints::package::CodeHash;
use sbor::rust::sync::{Arc, Mutex};
use wasmer::{
    imports, Function, HostEnvInitError, Instance, LazyInit, Module, RuntimeError, Store,
    Universal, Val, WasmerEnv,
};
use wasmer_compiler_singlepass::Singlepass;

// IMPORTANT:
// The below integration of Wasmer is not yet checked rigorously enough for production use
// TODO: Address the below issues before considering production use.

/// A `WasmerModule` defines a parsed WASM module, which is a template which can be instantiated.
///
/// Unlike `WasmerInstance`, this is correctly `Send + Sync` - which is good, because this is the
/// thing which is cached in the ScryptoInterpreter caches.
pub struct WasmerModule {
    module: Module,
    #[allow(dead_code)]
    code_size_bytes: usize,
}

/// WARNING - this type should not actually be Send + Sync - it should really store a raw pointer,
/// not a raw pointer masked as a usize.
///
/// For information on why the pointer is masked, see the docs for `WasmerInstanceEnv`
pub struct WasmerInstance {
    instance: Instance,

    /// This field stores a (masked) runtime pointer to a `Box<dyn WasmRuntime>` which is shared
    /// by the instance and each WasmerInstanceEnv (every function that requires `env`).
    ///
    /// On every call into the WASM (ie every call to `invoke_export`), a `&'a mut System API` is
    /// wrapped in a temporary `RadixEngineWasmRuntime<'a>` and boxed, and a pointer to the freshly
    /// created `Box<dyn WasmRuntime>` is written behind the Mutex into this field.
    ///
    /// This same Mutex (via Arc cloning) is shared into each `WasmerInstanceEnv`, and so
    /// when the WASM makes calls back into env, it can read the pointer to the current
    /// WasmRuntime, and use that to call into the `&mut System API`.
    ///
    /// For information on why the pointer is masked, see the docs for `WasmerInstanceEnv`
    runtime_ptr: Arc<Mutex<usize>>,
}

/// The WasmerInstanceEnv implements WasmerEnv - and this needs to be `Send + Sync` for
/// Wasmer to work (see `Function::new_native_with_env`).
///
/// This is likely because Wasmer wants to be forward-compatible with multi-threaded WASM,
/// or that it uses multiple threads internally.
///
/// Currently, the SystemAPI is not Sync (and so should not be accessed by multiple threads)
/// we believe our use of Wasmer does not allow it to call us from multiple threads -
/// but we need to double-check this.
///
/// In any case, we temporarily work around this incompatibility by masking the pointer as a usize.
///
/// There are still a number of changes we should consider to improve things:
/// * `WasmerInstanceEnv` shouldn't contain an Instance - just a memory reference - see
///    the docs on the `WasmerEnv` trait
/// * If we instantiate the module just before we call into it, we could potentially pass an actual
///   `Arc<Mutex<T>>` for `a', T: WasmRuntime<'a>` (wrapping a `&'a mut SystemAPI`) into the WasmerInstanceEnv
///   on *module instantiation*. In this case, it doesn't need to be on a WasmerInstance at all
/// * Else at the very least, change this to be a pointer type, and manually implement Sync/Send
#[derive(Clone)]
pub struct WasmerInstanceEnv {
    instance: LazyInit<Instance>,
    /// See notes on `WasmerInstance.runtime_ptr`
    runtime_ptr: Arc<Mutex<usize>>,
}

pub struct WasmerEngine {
    store: Store,
    // This flag disables cache in wasm_instrumenter/wasmi/wasmer to prevent non-determinism when fuzzing
    #[cfg(all(not(feature = "radix_engine_fuzzing"), not(feature = "moka")))]
    modules_cache: RefCell<lru::LruCache<CodeHash, Arc<WasmerModule>>>,
    #[cfg(all(not(feature = "radix_engine_fuzzing"), feature = "moka"))]
    modules_cache: moka::sync::Cache<CodeHash, Arc<WasmerModule>>,
    #[cfg(feature = "radix_engine_fuzzing")]
    modules_cache: usize,
}

pub fn read_memory(instance: &Instance, ptr: u32, len: u32) -> Result<Vec<u8>, WasmRuntimeError> {
    let ptr = ptr as usize;
    let len = len as usize;

    let memory = instance
        .exports
        .get_memory(EXPORT_MEMORY)
        .map_err(|_| WasmRuntimeError::MemoryAccessError)?;
    let memory_slice = unsafe { memory.data_unchecked() };
    let memory_size = memory_slice.len();
    if ptr > memory_size || ptr + len > memory_size {
        return Err(WasmRuntimeError::MemoryAccessError);
    }

    Ok(memory_slice[ptr..ptr + len].to_vec())
}

pub fn write_memory(instance: &Instance, ptr: u32, data: &[u8]) -> Result<(), WasmRuntimeError> {
    let ptr = ptr as usize;
    let len = data.len();

    let memory = instance
        .exports
        .get_memory(EXPORT_MEMORY)
        .map_err(|_| WasmRuntimeError::MemoryAccessError)?;
    let memory_slice = unsafe { memory.data_unchecked_mut() };
    let memory_size = memory_slice.len();
    if ptr > memory_size || ptr + len > memory_size {
        return Err(WasmRuntimeError::MemoryAccessError);
    }

    memory_slice[ptr..ptr + data.len()].copy_from_slice(data);
    Ok(())
}

pub fn read_slice(instance: &Instance, v: Slice) -> Result<Vec<u8>, WasmRuntimeError> {
    let ptr = v.ptr();
    let len = v.len();

    read_memory(instance, ptr, len)
}

pub fn get_memory_size(instance: &Instance) -> Result<usize, WasmRuntimeError> {
    let memory = instance
        .exports
        .get_memory(EXPORT_MEMORY)
        .map_err(|_| WasmRuntimeError::MemoryAccessError)?;
    let memory_slice = unsafe { memory.data_unchecked() };

    Ok(memory_slice.len())
}

impl WasmerEnv for WasmerInstanceEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.instance.initialize(instance.clone());
        Ok(())
    }
}

macro_rules! grab_runtime {
    ($env: expr) => {{
        let instance = unsafe { $env.instance.get_unchecked() };
        let ptr = $env.runtime_ptr.lock().expect("Runtime ptr unavailable");
        let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };
        (instance, runtime)
    }};
}

impl From<WasmRuntimeError> for RuntimeError {
    fn from(error: WasmRuntimeError) -> Self {
        RuntimeError::user(Box::new(error))
    }
}

impl WasmerModule {
    fn instantiate(&self) -> WasmerInstance {
        // native functions starts
        pub fn buffer_consume(
            env: &WasmerInstanceEnv,
            buffer_id: BufferId,
            destination_ptr: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let result = runtime.buffer_consume(buffer_id);
            match result {
                Ok(slice) => {
                    write_memory(instance, destination_ptr, &slice)?;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }

        pub fn blueprint_call(
            env: &WasmerInstanceEnv,
            package_address_ptr: u32,
            package_address_len: u32,
            blueprint_name_ptr: u32,
            blueprint_name_len: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let package_address = read_memory(&instance, package_address_ptr, package_address_len)?;
            let blueprint_name = read_memory(&instance, blueprint_name_ptr, blueprint_name_len)?;
            let ident = read_memory(&instance, ident_ptr, ident_len)?;
            let args = read_memory(&instance, args_ptr, args_len)?;

            runtime
                .blueprint_call(package_address, blueprint_name, ident, args)
                .map(|buffer| buffer.0)
        }

        pub fn address_allocate(
            env: &WasmerInstanceEnv,
            package_address_ptr: u32,
            package_address_len: u32,
            blueprint_name_ptr: u32,
            blueprint_name_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            runtime
                .address_allocate(
                    read_memory(&instance, package_address_ptr, package_address_len)?,
                    read_memory(&instance, blueprint_name_ptr, blueprint_name_len)?,
                )
                .map(|buffer| buffer.0)
        }

        pub fn address_get_reservation_address(
            env: &WasmerInstanceEnv,
            node_id_ptr: u32,
            node_id_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            runtime
                .address_get_reservation_address(read_memory(&instance, node_id_ptr, node_id_len)?)
                .map(|buffer| buffer.0)
        }

        pub fn object_call(
            env: &WasmerInstanceEnv,
            receiver_ptr: u32,
            receiver_len: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let receiver = read_memory(&instance, receiver_ptr, receiver_len)?;
            let ident = read_memory(&instance, ident_ptr, ident_len)?;
            let args = read_memory(&instance, args_ptr, args_len)?;

            runtime
                .object_call(receiver, ident, args)
                .map(|buffer| buffer.0)
        }

        pub fn object_call_module(
            env: &WasmerInstanceEnv,
            receiver_ptr: u32,
            receiver_len: u32,
            module: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let receiver = read_memory(&instance, receiver_ptr, receiver_len)?;
            let ident = read_memory(&instance, ident_ptr, ident_len)?;
            let args = read_memory(&instance, args_ptr, args_len)?;

            runtime
                .object_call_module(receiver, module, ident, args)
                .map(|buffer| buffer.0)
        }

        pub fn object_call_direct(
            env: &WasmerInstanceEnv,
            receiver_ptr: u32,
            receiver_len: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let receiver = read_memory(&instance, receiver_ptr, receiver_len)?;
            let ident = read_memory(&instance, ident_ptr, ident_len)?;
            let args = read_memory(&instance, args_ptr, args_len)?;

            runtime
                .object_call_direct(receiver, ident, args)
                .map(|buffer| buffer.0)
        }

        pub fn object_new(
            env: &WasmerInstanceEnv,
            blueprint_name_ptr: u32,
            blueprint_name_len: u32,
            object_states_ptr: u32,
            object_states_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            runtime
                .object_new(
                    read_memory(&instance, blueprint_name_ptr, blueprint_name_len)?,
                    read_memory(&instance, object_states_ptr, object_states_len)?,
                )
                .map(|buffer| buffer.0)
        }

        pub fn object_globalize(
            env: &WasmerInstanceEnv,
            obj_ptr: u32,
            obj_len: u32,
            modules_ptr: u32,
            modules_len: u32,
            address_ptr: u32,
            address_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            runtime
                .globalize_object(
                    read_memory(&instance, obj_ptr, obj_len)?,
                    read_memory(&instance, modules_ptr, modules_len)?,
                    read_memory(&instance, address_ptr, address_len)?,
                )
                .map(|buffer| buffer.0)
        }

        pub fn object_instance_of(
            env: &WasmerInstanceEnv,
            component_id_ptr: u32,
            component_id_len: u32,
            package_address_ptr: u32,
            package_address_len: u32,
            blueprint_name_ptr: u32,
            blueprint_name_len: u32,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            runtime.instance_of(
                read_memory(&instance, component_id_ptr, component_id_len)?,
                read_memory(&instance, package_address_ptr, package_address_len)?,
                read_memory(&instance, blueprint_name_ptr, blueprint_name_len)?,
            )
        }

        pub fn object_get_blueprint_id(
            env: &WasmerInstanceEnv,
            component_id_ptr: u32,
            component_id_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            runtime
                .blueprint_id(read_memory(&instance, component_id_ptr, component_id_len)?)
                .map(|buffer| buffer.0)
        }

        pub fn object_get_outer_object(
            env: &WasmerInstanceEnv,
            component_id_ptr: u32,
            component_id_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            runtime
                .get_outer_object(read_memory(&instance, component_id_ptr, component_id_len)?)
                .map(|buffer| buffer.0)
        }

        pub fn key_value_store_new(
            env: &WasmerInstanceEnv,
            schema_id_ptr: u32,
            schema_id_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            runtime
                .key_value_store_new(read_memory(&instance, schema_id_ptr, schema_id_len)?)
                .map(|buffer| buffer.0)
        }

        pub fn key_value_store_open_entry(
            env: &WasmerInstanceEnv,
            node_id_ptr: u32,
            node_id_len: u32,
            key_ptr: u32,
            key_len: u32,
            flags: u32,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            runtime.key_value_store_open_entry(
                read_memory(&instance, node_id_ptr, node_id_len)?,
                read_memory(&instance, key_ptr, key_len)?,
                flags,
            )
        }

        pub fn key_value_store_remove_entry(
            env: &WasmerInstanceEnv,
            node_id_ptr: u32,
            node_id_len: u32,
            key_ptr: u32,
            key_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            runtime
                .key_value_store_remove_entry(
                    read_memory(&instance, node_id_ptr, node_id_len)?,
                    read_memory(&instance, key_ptr, key_len)?,
                )
                .map(|buffer| buffer.0)
        }

        pub fn key_value_entry_read(
            env: &WasmerInstanceEnv,
            handle: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.key_value_entry_get(handle).map(|buffer| buffer.0)
        }

        pub fn key_value_entry_write(
            env: &WasmerInstanceEnv,
            handle: u32,
            data_ptr: u32,
            data_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let data = read_memory(&instance, data_ptr, data_len)?;

            runtime.key_value_entry_set(handle, data)
        }

        pub fn key_value_entry_remove(
            env: &WasmerInstanceEnv,
            handle: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime
                .key_value_entry_remove(handle)
                .map(|buffer| buffer.0)
        }

        pub fn key_value_entry_close(
            env: &WasmerInstanceEnv,
            handle: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.key_value_entry_close(handle)
        }

        pub fn field_entry_read(
            env: &WasmerInstanceEnv,
            handle: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.field_entry_read(handle).map(|buffer| buffer.0)
        }

        pub fn field_entry_write(
            env: &WasmerInstanceEnv,
            handle: u32,
            data_ptr: u32,
            data_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let data = read_memory(&instance, data_ptr, data_len)?;

            runtime.field_entry_write(handle, data)
        }

        pub fn field_entry_close(
            env: &WasmerInstanceEnv,
            handle: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.field_entry_close(handle)
        }

        pub fn actor_open_field(
            env: &WasmerInstanceEnv,
            object_handle: u32,
            field: u8,
            flags: u32,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.actor_open_field(object_handle, field, flags)
        }

        pub fn actor_get_node_id(
            env: &WasmerInstanceEnv,
            actor_ref_handle: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime
                .actor_get_node_id(actor_ref_handle)
                .map(|buffer| buffer.0)
        }

        pub fn actor_get_package_address(
            env: &WasmerInstanceEnv,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.actor_get_package_address().map(|buffer| buffer.0)
        }

        pub fn actor_get_blueprint_name(
            env: &WasmerInstanceEnv,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.actor_get_blueprint_name().map(|buffer| buffer.0)
        }

        fn actor_emit_event(
            env: &WasmerInstanceEnv,
            event_name_ptr: u32,
            event_name_len: u32,
            event_data_ptr: u32,
            event_data_len: u32,
            flags: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let event_name = read_memory(&instance, event_name_ptr, event_name_len)?;
            let event_data = read_memory(&instance, event_data_ptr, event_data_len)?;
            let event_flags = EventFlags::from_bits(flags).ok_or(InvokeError::SelfError(
                WasmRuntimeError::InvalidEventFlags(flags),
            ))?;

            runtime.actor_emit_event(event_name, event_data, event_flags)
        }

        pub fn costing_get_execution_cost_unit_limit(
            env: &WasmerInstanceEnv,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.costing_get_execution_cost_unit_limit()
        }

        pub fn costing_get_execution_cost_unit_price(
            env: &WasmerInstanceEnv,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime
                .costing_get_execution_cost_unit_price()
                .map(|buffer| buffer.0)
        }

        pub fn costing_get_finalization_cost_unit_limit(
            env: &WasmerInstanceEnv,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.costing_get_finalization_cost_unit_limit()
        }

        pub fn costing_get_finalization_cost_unit_price(
            env: &WasmerInstanceEnv,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime
                .costing_get_finalization_cost_unit_price()
                .map(|buffer| buffer.0)
        }

        pub fn costing_get_tip_percentage(
            env: &WasmerInstanceEnv,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.costing_get_tip_percentage()
        }

        pub fn costing_get_fee_balance(
            env: &WasmerInstanceEnv,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.costing_get_fee_balance().map(|buffer| buffer.0)
        }

        pub fn costing_get_usd_price(
            env: &WasmerInstanceEnv,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.costing_get_usd_price().map(|buffer| buffer.0)
        }

        fn consume_wasm_execution_units(
            env: &WasmerInstanceEnv,
            n: u64,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);
            // TODO: wasm-instrument uses u64 for cost units. We need to decide if we want to move from u32
            // to u64 as well.
            runtime.consume_wasm_execution_units(n as u32)
        }

        fn sys_log(
            env: &WasmerInstanceEnv,
            level_ptr: u32,
            level_len: u32,
            message_ptr: u32,
            message_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let level = read_memory(&instance, level_ptr, level_len)?;
            let message = read_memory(&instance, message_ptr, message_len)?;

            runtime.sys_log(level, message)
        }

        fn sys_bech32_encode_address(
            env: &WasmerInstanceEnv,
            address_ptr: u32,
            address_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let address = read_memory(&instance, address_ptr, address_len)?;

            runtime
                .sys_bech32_encode_address(address)
                .map(|buffer| buffer.0)
        }

        fn sys_panic(
            env: &WasmerInstanceEnv,
            message_ptr: u32,
            message_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let message = read_memory(&instance, message_ptr, message_len)?;

            runtime.sys_panic(message)
        }

        pub fn sys_get_transaction_hash(
            env: &WasmerInstanceEnv,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.sys_get_transaction_hash().map(|buffer| buffer.0)
        }

        pub fn sys_generate_ruid(
            env: &WasmerInstanceEnv,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime.sys_generate_ruid().map(|buffer| buffer.0)
        }

        pub fn bls12381_v1_verify(
            env: &WasmerInstanceEnv,
            message_ptr: u32,
            message_len: u32,
            public_key_ptr: u32,
            public_key_len: u32,
            signature_ptr: u32,
            signature_len: u32,
        ) -> Result<u32, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let message = read_memory(&instance, message_ptr, message_len)?;

            let public_key = read_memory(&instance, public_key_ptr, public_key_len)?;
            let signature = read_memory(instance, signature_ptr, signature_len)?;

            runtime.crypto_utils_bls12381_v1_verify(message, public_key, signature)
        }

        pub fn keccak256_hash(
            env: &WasmerInstanceEnv,
            data_ptr: u32,
            data_len: u32,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let data = read_memory(instance, data_ptr, data_len)?;

            runtime
                .crypto_utils_keccak256_hash(data)
                .map(|buffer| buffer.0)
        }

        #[cfg(feature = "radix_engine_tests")]
        pub fn host_read_memory(
            env: &WasmerInstanceEnv,
            memory_ptr: u32,
            data_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (instance, _runtime) = grab_runtime!(env);

            let _ = read_memory(&instance, memory_ptr, data_len)?;

            Ok(())
        }

        #[cfg(feature = "radix_engine_tests")]
        pub fn host_write_memory(
            env: &WasmerInstanceEnv,
            memory_ptr: u32,
            data_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            // - generate some random data of of given length data_len
            // - attempt to write this data into given memory offset memory_ptr
            let (instance, _runtime) = grab_runtime!(env);
            let data = vec![0u8; data_len as usize];

            let _ = write_memory(&instance, memory_ptr, &data)?;

            Ok(())
        }

        #[cfg(feature = "radix_engine_tests")]
        pub fn host_check_memory_is_clean(
            env: &WasmerInstanceEnv,
        ) -> Result<u64, InvokeError<WasmRuntimeError>> {
            // - generate some random data of of given length data_len
            // - attempt to write this data into given memory offset memory_ptr
            let (instance, _runtime) = grab_runtime!(env);

            let memory = instance
                .exports
                .get_memory(EXPORT_MEMORY)
                .map_err(|_| WasmRuntimeError::MemoryAccessError)?;
            let memory_slice = unsafe { memory.data_unchecked() };

            println!(
                "memory len = {:?} : {:02x?}",
                memory_slice.len(),
                memory_slice
            );
            let clean = !memory_slice.iter().any(|&x| x != 0x0);

            Ok(clean as u64)
        }
        // native functions ends

        // env
        let env = WasmerInstanceEnv {
            instance: LazyInit::new(),
            runtime_ptr: Arc::new(Mutex::new(0)),
        };

        // imports
        let import_object = imports! {
            MODULE_ENV_NAME => {
                BLUEPRINT_CALL_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), blueprint_call),
                ADDRESS_ALLOCATE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), address_allocate),
                ADDRESS_GET_RESERVATION_ADDRESS_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), address_get_reservation_address),
                OBJECT_NEW_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), object_new),
                OBJECT_GLOBALIZE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), object_globalize),
                OBJECT_INSTANCE_OF_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), object_instance_of),
                OBJECT_GET_BLUEPRINT_ID_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), object_get_blueprint_id),
                OBJECT_GET_OUTER_OBJECT_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), object_get_outer_object),
                OBJECT_CALL_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), object_call),
                OBJECT_CALL_MODULE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), object_call_module),
                OBJECT_CALL_DIRECT_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), object_call_direct),
                KEY_VALUE_STORE_NEW_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_store_new),
                KEY_VALUE_STORE_OPEN_ENTRY_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_store_open_entry),
                KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_store_remove_entry),
                KEY_VALUE_ENTRY_READ_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_entry_read),
                KEY_VALUE_ENTRY_WRITE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_entry_write),
                KEY_VALUE_ENTRY_REMOVE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_entry_remove),
                KEY_VALUE_ENTRY_CLOSE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_entry_close),
                FIELD_ENTRY_READ_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), field_entry_read),
                FIELD_ENTRY_WRITE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), field_entry_write),
                FIELD_ENTRY_CLOSE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), field_entry_close),
                ACTOR_OPEN_FIELD_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), actor_open_field),
                ACTOR_GET_OBJECT_ID_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), actor_get_node_id),
                ACTOR_GET_PACKAGE_ADDRESS_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), actor_get_package_address),
                ACTOR_GET_BLUEPRINT_NAME_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), actor_get_blueprint_name),
                ACTOR_EMIT_EVENT_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), actor_emit_event),
                COSTING_CONSUME_WASM_EXECUTION_UNITS_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), consume_wasm_execution_units),
                COSTING_GET_EXECUTION_COST_UNIT_LIMIT_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), costing_get_execution_cost_unit_limit),
                COSTING_GET_EXECUTION_COST_UNIT_PRICE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), costing_get_execution_cost_unit_price),
                COSTING_GET_FINALIZATION_COST_UNIT_LIMIT_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), costing_get_finalization_cost_unit_limit),
                COSTING_GET_FINALIZATION_COST_UNIT_PRICE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), costing_get_finalization_cost_unit_price),
                COSTING_GET_USD_PRICE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), costing_get_usd_price),
                COSTING_GET_TIP_PERCENTAGE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), costing_get_tip_percentage),
                COSTING_GET_FEE_BALANCE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), costing_get_fee_balance),
                SYS_LOG_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), sys_log),
                SYS_BECH32_ENCODE_ADDRESS_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), sys_bech32_encode_address),
                SYS_PANIC_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), sys_panic),
                SYS_GET_TRANSACTION_HASH_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), sys_get_transaction_hash),
                SYS_GENERATE_RUID_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), sys_generate_ruid),
                BUFFER_CONSUME_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), buffer_consume),
                CRYPTO_UTILS_BLS12381_V1_VERIFY_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), bls12381_v1_verify),
                CRYPTO_UTILS_KECCAK256_HASH_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), keccak256_hash),

                #[cfg(feature = "radix_engine_tests")]
                "test_host_read_memory" => Function::new_native_with_env(self.module.store(), env.clone(), host_read_memory),
                #[cfg(feature = "radix_engine_tests")]
                "test_host_write_memory" => Function::new_native_with_env(self.module.store(), env.clone(), host_write_memory),
                #[cfg(feature = "radix_engine_tests")]
                "test_host_check_memory_is_clean" => Function::new_native_with_env(self.module.store(), env.clone(), host_check_memory_is_clean),
            }
        };

        // instantiate
        let instance =
            Instance::new(&self.module, &import_object).expect("Failed to instantiate module");

        WasmerInstance {
            instance,
            runtime_ptr: env.runtime_ptr,
        }
    }
}

impl From<RuntimeError> for InvokeError<WasmRuntimeError> {
    fn from(error: RuntimeError) -> Self {
        let e_str = format!("{:?}", error);
        match error.downcast::<InvokeError<WasmRuntimeError>>() {
            Ok(e) => e,
            _ => InvokeError::SelfError(WasmRuntimeError::ExecutionError(e_str)),
        }
    }
}

impl WasmInstance for WasmerInstance {
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: Vec<Buffer>,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
        {
            // set up runtime pointer
            let mut guard = self.runtime_ptr.lock().expect("Runtime ptr unavailable");
            *guard = runtime as *mut _ as usize;
        }

        let input: Vec<Val> = args
            .into_iter()
            .map(|buffer| Val::I64(buffer.as_i64()))
            .collect();
        let return_data = self
            .instance
            .exports
            .get_function(func_name)
            .map_err(|_| {
                InvokeError::SelfError(WasmRuntimeError::UnknownExport(func_name.to_string()))
            })?
            .call(&input)
            .map_err(|e| {
                let err: InvokeError<WasmRuntimeError> = e.into();
                err
            });

        let result = match return_data {
            Ok(data) => {
                if let Some(v) = data.as_ref().get(0).and_then(|x| x.i64()) {
                    read_slice(&self.instance, Slice::transmute_i64(v))
                        .map_err(InvokeError::SelfError)
                } else {
                    Err(InvokeError::SelfError(WasmRuntimeError::InvalidWasmPointer))
                }
            }
            Err(err) => Err(err),
        };

        #[cfg(feature = "coverage")]
        if let Ok(dump_coverage) = self.instance.exports.get_function("dump_coverage") {
            if let Ok(blueprint_buffer) = runtime.actor_get_blueprint_name() {
                let blueprint_name =
                    String::from_utf8(runtime.buffer_consume(blueprint_buffer.id()).unwrap())
                        .unwrap();

                let mut ret = dump_coverage
                    .call(&[])
                    .unwrap()
                    .as_ref()
                    .get(0)
                    .and_then(|x| x.i64())
                    .unwrap();
                let coverage_data = read_slice(&self.instance, Slice::transmute_i64(ret)).unwrap();
                save_coverage_data(&blueprint_name, &coverage_data);
            }
        }

        result
    }
}

#[derive(Debug, Clone)]
pub struct WasmerEngineOptions {
    max_cache_size: usize,
}

impl Default for WasmerEngine {
    fn default() -> Self {
        Self::new(WasmerEngineOptions {
            max_cache_size: WASM_ENGINE_CACHE_SIZE,
        })
    }
}

impl WasmerEngine {
    pub fn new(options: WasmerEngineOptions) -> Self {
        let compiler = Singlepass::new();

        #[cfg(all(not(feature = "radix_engine_fuzzing"), not(feature = "moka")))]
        let modules_cache = RefCell::new(lru::LruCache::new(
            NonZeroUsize::new(options.max_cache_size).unwrap(),
        ));
        #[cfg(all(not(feature = "radix_engine_fuzzing"), feature = "moka"))]
        let modules_cache = moka::sync::Cache::builder()
            .weigher(
                |_metered_code_key: &CodeHash, _value: &Arc<WasmerModule>| -> u32 {
                    // No sophisticated weighing mechanism, just keep a fixed size cache
                    1u32
                },
            )
            .max_capacity(options.max_cache_size as u64)
            .build();
        #[cfg(feature = "radix_engine_fuzzing")]
        let modules_cache = options.max_cache_size;

        Self {
            store: Store::new(&Universal::new(compiler).engine()),
            modules_cache,
        }
    }
}

impl WasmEngine for WasmerEngine {
    type WasmInstance = WasmerInstance;

    fn instantiate(&self, code_hash: CodeHash, instrumented_code: &[u8]) -> WasmerInstance {
        #[cfg(not(feature = "radix_engine_fuzzing"))]
        {
            #[cfg(not(feature = "moka"))]
            {
                if let Some(cached_module) = self.modules_cache.borrow_mut().get(&code_hash) {
                    return cached_module.instantiate();
                }
            }
            #[cfg(feature = "moka")]
            if let Some(cached_module) = self.modules_cache.get(&code_hash) {
                return cached_module.instantiate();
            }
        }

        let new_module = Arc::new(WasmerModule {
            module: Module::new(&self.store, instrumented_code)
                .expect("Failed to parse WASM module"),
            code_size_bytes: instrumented_code.len(),
        });

        #[cfg(not(feature = "radix_engine_fuzzing"))]
        {
            #[cfg(not(feature = "moka"))]
            self.modules_cache
                .borrow_mut()
                .put(code_hash, new_module.clone());
            #[cfg(feature = "moka")]
            self.modules_cache.insert(code_hash, new_module.clone());
        }

        new_module.instantiate()
    }
}
