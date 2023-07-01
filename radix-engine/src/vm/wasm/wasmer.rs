use crate::errors::InvokeError;
use crate::types::*;
use crate::vm::wasm::constants::*;
use crate::vm::wasm::errors::*;
use crate::vm::wasm::traits::*;
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
    modules_cache: RefCell<lru::LruCache<Hash, Arc<WasmerModule>>>,
    #[cfg(all(not(feature = "radix_engine_fuzzing"), feature = "moka"))]
    modules_cache: moka::sync::Cache<Hash, Arc<WasmerModule>>,
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
        pub fn consume_buffer(
            env: &WasmerInstanceEnv,
            buffer_id: BufferId,
            destination_ptr: u32,
        ) -> Result<(), RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let slice = runtime
                .consume_buffer(buffer_id)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            write_memory(&instance, destination_ptr, &slice)?;

            Ok(())
        }

        pub fn actor_call_module_method(
            env: &WasmerInstanceEnv,
            object_handle: u32,
            module_id: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let ident = read_memory(&instance, ident_ptr, ident_len)?;
            let args = read_memory(&instance, args_ptr, args_len)?;

            let buffer = runtime
                .actor_call_module_method(object_handle, module_id, ident, args)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn call_method(
            env: &WasmerInstanceEnv,
            receiver_ptr: u32,
            receiver_len: u32,
            direct_access: u32,
            module_id: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let receiver = read_memory(&instance, receiver_ptr, receiver_len)?;
            let ident = read_memory(&instance, ident_ptr, ident_len)?;
            let args = read_memory(&instance, args_ptr, args_len)?;

            let buffer = runtime
                .call_method(receiver, direct_access, module_id, ident, args)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn call_function(
            env: &WasmerInstanceEnv,
            package_address_ptr: u32,
            package_address_len: u32,
            blueprint_ident_ptr: u32,
            blueprint_ident_len: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let package_address = read_memory(&instance, package_address_ptr, package_address_len)?;
            let blueprint_ident = read_memory(&instance, blueprint_ident_ptr, blueprint_ident_len)?;
            let ident = read_memory(&instance, ident_ptr, ident_len)?;
            let args = read_memory(&instance, args_ptr, args_len)?;

            let buffer = runtime
                .call_function(package_address, blueprint_ident, ident, args)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn new_object(
            env: &WasmerInstanceEnv,
            blueprint_ident_ptr: u32,
            blueprint_ident_len: u32,
            object_states_ptr: u32,
            object_states_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .new_object(
                    read_memory(&instance, blueprint_ident_ptr, blueprint_ident_len)?,
                    read_memory(&instance, object_states_ptr, object_states_len)?,
                )
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn allocate_global_address(
            env: &WasmerInstanceEnv,
            blueprint_ident_ptr: u32,
            blueprint_ident_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .allocate_global_address(read_memory(
                    &instance,
                    blueprint_ident_ptr,
                    blueprint_ident_len,
                )?)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn cost_unit_limit(env: &WasmerInstanceEnv) -> Result<u32, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime
                .cost_unit_limit()
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        pub fn cost_unit_price(env: &WasmerInstanceEnv) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .cost_unit_price()
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn tip_percentage(env: &WasmerInstanceEnv) -> Result<u32, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime
                .tip_percentage()
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        pub fn fee_balance(env: &WasmerInstanceEnv) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .fee_balance()
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn globalize_object(
            env: &WasmerInstanceEnv,
            modules_ptr: u32,
            modules_len: u32,
            address_ptr: u32,
            address_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .globalize_object(
                    read_memory(&instance, modules_ptr, modules_len)?,
                    read_memory(&instance, address_ptr, address_len)?,
                )
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn get_type_info(
            env: &WasmerInstanceEnv,
            component_id_ptr: u32,
            component_id_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .get_object_info(read_memory(&instance, component_id_ptr, component_id_len)?)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn key_value_store_new(
            env: &WasmerInstanceEnv,
            schema_id_ptr: u32,
            schema_id_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .key_value_store_new(read_memory(&instance, schema_id_ptr, schema_id_len)?)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn key_value_store_open_entry(
            env: &WasmerInstanceEnv,
            node_id_ptr: u32,
            node_id_len: u32,
            key_ptr: u32,
            key_len: u32,
            flags: u32,
        ) -> Result<u32, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let handle = runtime
                .key_value_store_open_entry(
                    read_memory(&instance, node_id_ptr, node_id_len)?,
                    read_memory(&instance, key_ptr, key_len)?,
                    flags,
                )
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(handle)
        }

        pub fn key_value_store_remove_entry(
            env: &WasmerInstanceEnv,
            node_id_ptr: u32,
            node_id_len: u32,
            key_ptr: u32,
            key_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .key_value_store_remove_entry(
                    read_memory(&instance, node_id_ptr, node_id_len)?,
                    read_memory(&instance, key_ptr, key_len)?,
                )
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn key_value_entry_get(
            env: &WasmerInstanceEnv,
            handle: u32,
        ) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .key_value_entry_get(handle)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn key_value_entry_set(
            env: &WasmerInstanceEnv,
            handle: u32,
            data_ptr: u32,
            data_len: u32,
        ) -> Result<(), RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let data = read_memory(&instance, data_ptr, data_len)?;

            runtime
                .key_value_entry_set(handle, data)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(())
        }

        pub fn key_value_entry_release(
            env: &WasmerInstanceEnv,
            handle: u32,
        ) -> Result<(), RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime
                .key_value_entry_release(handle)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(())
        }

        pub fn drop_object(
            env: &WasmerInstanceEnv,
            node_id_ptr: u32,
            node_id_len: u32,
        ) -> Result<(), RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let node_id = read_memory(&instance, node_id_ptr, node_id_len)?;

            runtime
                .drop_object(node_id)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(())
        }

        pub fn actor_open_field(
            env: &WasmerInstanceEnv,
            object_handle: u32,
            field: u8,
            flags: u32,
        ) -> Result<u32, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let handle = runtime
                .actor_open_field(object_handle, field, flags)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(handle)
        }

        pub fn field_lock_read(env: &WasmerInstanceEnv, handle: u32) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .field_lock_read(handle)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn field_lock_write(
            env: &WasmerInstanceEnv,
            handle: u32,
            data_ptr: u32,
            data_len: u32,
        ) -> Result<(), RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let data = read_memory(&instance, data_ptr, data_len)?;

            runtime
                .field_lock_write(handle, data)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(())
        }

        pub fn field_lock_release(
            env: &WasmerInstanceEnv,
            handle: u32,
        ) -> Result<(), RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime
                .field_lock_release(handle)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(())
        }

        pub fn get_node_id(env: &WasmerInstanceEnv) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .get_node_id()
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn get_global_address(env: &WasmerInstanceEnv) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .get_global_address()
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn get_blueprint(env: &WasmerInstanceEnv) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .get_blueprint()
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn get_auth_zone(env: &WasmerInstanceEnv) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .get_auth_zone()
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn assert_access_rule(
            env: &WasmerInstanceEnv,
            rule_ptr: u32,
            rule_len: u32,
        ) -> Result<(), RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let rule = read_memory(&instance, rule_ptr, rule_len)?;

            runtime
                .assert_access_rule(rule)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        fn consume_wasm_execution_units(
            env: &WasmerInstanceEnv,
            n: u32,
        ) -> Result<(), RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);
            runtime
                .consume_wasm_execution_units(n)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        fn emit_event(
            env: &WasmerInstanceEnv,
            event_name_ptr: u32,
            event_name_len: u32,
            event_data_ptr: u32,
            event_data_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let event_name = read_memory(&instance, event_name_ptr, event_name_len)?;
            let event_data = read_memory(&instance, event_data_ptr, event_data_len)?;

            runtime.emit_event(event_name, event_data)
        }

        fn emit_log(
            env: &WasmerInstanceEnv,
            level_ptr: u32,
            level_len: u32,
            message_ptr: u32,
            message_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let level = read_memory(&instance, level_ptr, level_len)?;
            let message = read_memory(&instance, message_ptr, message_len)?;

            runtime.emit_log(level, message)
        }

        fn panic(
            env: &WasmerInstanceEnv,
            message_ptr: u32,
            message_len: u32,
        ) -> Result<(), InvokeError<WasmRuntimeError>> {
            let (instance, runtime) = grab_runtime!(env);

            let message = read_memory(&instance, message_ptr, message_len)?;

            runtime.panic(message)
        }

        pub fn get_transaction_hash(env: &WasmerInstanceEnv) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .get_transaction_hash()
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn generate_ruid(env: &WasmerInstanceEnv) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .generate_ruid()
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
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
                CONSUME_BUFFER_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), consume_buffer),
                CALL_METHOD_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), call_method),
                CALL_FUNCTION_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), call_function),
                NEW_OBJECT_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), new_object),
                ALLOCATE_GLOBAL_ADDRESS_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), allocate_global_address),
                COST_UNIT_LIMIT_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), cost_unit_limit),
                COST_UNIT_PRICE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), cost_unit_price),
                TIP_PERCENTAGE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), tip_percentage),
                FEE_BALANCE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), fee_balance),
                GLOBALIZE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), globalize_object),
                GET_OBJECT_INFO_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), get_type_info),
                DROP_OBJECT_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), drop_object),
                ACTOR_OPEN_FIELD_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), actor_open_field),
                ACTOR_CALL_MODULE_METHOD_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), actor_call_module_method),
                KEY_VALUE_STORE_NEW_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_store_new),
                KEY_VALUE_STORE_OPEN_ENTRY_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_store_open_entry),
                KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_store_remove_entry),
                KEY_VALUE_ENTRY_GET_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_entry_get),
                KEY_VALUE_ENTRY_SET_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_entry_set),
                KEY_VALUE_ENTRY_RELEASE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), key_value_entry_release),
                FIELD_LOCK_READ_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), field_lock_read),
                FIELD_LOCK_WRITE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), field_lock_write),
                FIELD_LOCK_RELEASE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), field_lock_release),
                GET_NODE_ID_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), get_node_id),
                GET_GLOBAL_ADDRESS_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), get_global_address),
                GET_BLUEPRINT_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), get_blueprint),
                GET_AUTH_ZONE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), get_auth_zone),
                ASSERT_ACCESS_RULE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), assert_access_rule),
                CONSUME_WASM_EXECUTION_UNITS_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), consume_wasm_execution_units),
                EMIT_EVENT_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), emit_event),
                EMIT_LOG_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), emit_log),
                PANIC_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), panic),
                GET_TRANSACTION_HASH_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), get_transaction_hash),
                GENERATE_RUID_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), generate_ruid),
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
            })?;

        if let Some(v) = return_data.as_ref().get(0).and_then(|x| x.i64()) {
            read_slice(&self.instance, Slice::transmute_i64(v)).map_err(InvokeError::SelfError)
        } else {
            Err(InvokeError::SelfError(WasmRuntimeError::InvalidWasmPointer))
        }
    }

    fn consumed_memory(&self) -> Result<usize, InvokeError<WasmRuntimeError>> {
        let memory = self
            .instance
            .exports
            .get_memory(EXPORT_MEMORY)
            .map_err(|_| WasmRuntimeError::MemoryAccessError)?;
        let memory_slice = unsafe { memory.data_unchecked_mut() };
        Ok(memory_slice.len())
    }
}

#[derive(Debug, Clone)]
pub struct WasmerEngineOptions {
    max_cache_size: usize,
}

impl Default for WasmerEngine {
    fn default() -> Self {
        Self::new(WasmerEngineOptions {
            max_cache_size: DEFAULT_WASM_ENGINE_CACHE_SIZE,
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
                |_metered_code_key: &Hash, _value: &Arc<WasmerModule>| -> u32 {
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

    fn instantiate(&self, code_hash: Hash, instrumented_code: &[u8]) -> WasmerInstance {
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
