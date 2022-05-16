use std::sync::{Arc, Mutex};

use sbor::rust::boxed::Box;
use sbor::rust::collections::HashMap;
use sbor::rust::ptr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::crypto::{hash, Hash};
use scrypto::values::ScryptoValue;
use wasmer::*;
use wasmer_compiler_singlepass::Singlepass;

use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;

pub struct WasmerModule {
    module: Module,
}

pub struct WasmerInstance {
    instance: Instance,
    runtime_ptr: Arc<Mutex<usize>>,
}

#[derive(Clone)]
pub struct WasmerInstanceEnv {
    instance: LazyInit<Instance>,
    runtime_ptr: Arc<Mutex<usize>>,
}

pub struct WasmerEngine {
    store: Store,
    modules: HashMap<Hash, WasmerModule>,
}

pub fn send_value(instance: &Instance, value: &ScryptoValue) -> Result<usize, InvokeError> {
    let slice = &value.raw;
    let n = slice.len();

    let result = instance
        .exports
        .get_function(EXPORT_SCRYPTO_ALLOC)
        .map_err(|_| InvokeError::MemoryAllocError)?
        .call(&[Val::I32(n as i32)])
        .map_err(|_| InvokeError::MemoryAllocError)?;

    if let Some(Value::I32(ptr)) = result.as_ref().get(0) {
        let ptr = *ptr as usize;
        let memory = instance
            .exports
            .get_memory(EXPORT_MEMORY)
            .map_err(|_| InvokeError::MemoryAllocError)?;
        let size = memory.size().bytes().0;
        if size > ptr && size - ptr >= n {
            unsafe {
                let dest = memory.data_ptr().add(ptr + 4);
                ptr::copy(slice.as_ptr(), dest, n);
            }
            return Ok(ptr);
        }
    }

    Err(InvokeError::MemoryAllocError)
}

pub fn read_value(instance: &Instance, ptr: usize) -> Result<ScryptoValue, InvokeError> {
    let memory = instance
        .exports
        .get_memory(EXPORT_MEMORY)
        .map_err(|_| InvokeError::MemoryAccessError)?;
    let size = memory.size().bytes().0;
    if size > ptr && size - ptr >= 4 {
        // read len
        let mut temp = [0u8; 4];
        unsafe {
            let from = memory.data_ptr().add(ptr);
            ptr::copy(from, temp.as_mut_ptr(), 4);
        }
        let n = u32::from_le_bytes(temp) as usize;

        // read value
        if size - ptr - 4 >= (n as usize) {
            // TODO: avoid copying
            let mut temp = Vec::with_capacity(n);
            unsafe {
                let from = memory.data_ptr().add(ptr).add(4);
                ptr::copy(from, temp.as_mut_ptr(), n);
                temp.set_len(n);
            }

            return ScryptoValue::from_slice(&temp).map_err(InvokeError::InvalidScryptoValue);
        }
    }

    Err(InvokeError::MemoryAccessError)
}

impl WasmerEnv for WasmerInstanceEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.instance.initialize(instance.clone());
        Ok(())
    }
}

impl WasmerModule {
    fn instantiate(&self) -> WasmerInstance {
        // native functions
        fn radix_engine(env: &WasmerInstanceEnv, input_ptr: i32) -> Result<i32, RuntimeError> {
            let ptr = env.runtime_ptr.lock().unwrap();
            let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };
            let instance = unsafe { env.instance.get_unchecked() };

            let input = read_value(&instance, input_ptr as usize)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            let output = runtime
                .main(input)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            send_value(&instance, &output)
                .map(|ptr| ptr as i32)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        fn use_tbd(env: &WasmerInstanceEnv, tbd: i32) -> Result<(), RuntimeError> {
            let ptr = env.runtime_ptr.lock().unwrap();
            let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };

            runtime
                .use_tbd(tbd as u32)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        // env
        let env = WasmerInstanceEnv {
            instance: LazyInit::new(),
            runtime_ptr: Arc::new(Mutex::new(0)),
        };

        // imports
        let import_object = imports! {
            MODULE_ENV_NAME => {
                RADIX_ENGINE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), radix_engine),
                USE_TBD_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), use_tbd),
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

impl WasmInstance for WasmerInstance {
    fn invoke_export<'r>(
        &mut self,
        name: &str,
        input: &ScryptoValue,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<ScryptoValue, InvokeError> {
        {
            // set up runtime pointer
            let mut guard = self.runtime_ptr.lock().unwrap();
            *guard = runtime as *mut _ as usize;
        }

        let pointer = send_value(&self.instance, input)?;

        let result = self
            .instance
            .exports
            .get_function(name)
            .map_err(|_| InvokeError::FunctionNotFound)?
            .call(&[Val::I32(pointer as i32)]);

        match result {
            Ok(return_data) => {
                let ptr = return_data
                    .as_ref()
                    .get(0)
                    .ok_or(InvokeError::MissingReturnData)?
                    .i32()
                    .ok_or(InvokeError::InvalidReturnData)?;
                read_value(&self.instance, ptr as usize)
            }
            Err(e) => match e.downcast::<InvokeError>() {
                Ok(e) => Err(e),
                _ => Err(InvokeError::WasmError),
            },
        }
    }

    fn function_exports(&self) -> Vec<String> {
        self.instance
            .module()
            .exports()
            .filter(|e| matches!(e.ty(), ExternType::Function(_)))
            .map(|e| e.name().to_string())
            .collect()
    }
}

impl WasmerEngine {
    pub fn new() -> Self {
        let compiler = Singlepass::new();
        Self {
            store: Store::new(&Universal::new(compiler).engine()),
            modules: HashMap::new(),
        }
    }
}

impl WasmEngine<WasmerInstance> for WasmerEngine {
    fn validate(&mut self, _code: &[u8]) -> Result<(), WasmValidationError> {
        Ok(())
    }

    fn instrument(&mut self, code: &[u8]) -> Result<(), InstrumentError> {
        let code_hash = hash(code);

        let instrumented = code;

        self.modules.insert(
            code_hash,
            WasmerModule {
                module: Module::new(&self.store, instrumented)
                    .expect("Failed to parse wasm module"),
            },
        );

        Ok(())
    }

    fn instantiate(&mut self, code: &[u8]) -> WasmerInstance {
        let code_hash = hash(code);
        if self.modules.contains_key(&code_hash) {
            self.instrument(code)
                .expect("Failed to instrument the code");
        }
        let module = self.modules.get(&code_hash).unwrap();
        module.instantiate()
    }
}
