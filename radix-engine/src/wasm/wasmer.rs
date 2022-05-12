use std::sync::{Arc, Mutex};

use sbor::rust::boxed::Box;
use sbor::rust::ptr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::values::ScryptoValue;
use wasmer::*;
use wasmer_compiler_singlepass::Singlepass;

use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;

pub struct WasmerScryptoModule<'l> {
    module: Module,
    store: &'l Store,
}

pub struct WasmerScryptoInstance<'r> {
    instance: Instance,
    // This is to keep the trait object live so the runtime pointers
    // are valid as long as this instance is not dropped.
    #[allow(dead_code)]
    runtime: Box<dyn ScryptoRuntime + 'r>,
}

#[derive(Clone)]
pub struct WasmerScryptoInstanceEnv {
    instance: LazyInit<Instance>,
    runtime_ptr: Arc<Mutex<usize>>,
}

pub struct WasmerEngine {
    store: Store,
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

impl WasmerEnv for WasmerScryptoInstanceEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.instance.initialize(instance.clone());
        Ok(())
    }
}

impl<'l, 'r> ScryptoModule<'r, WasmerScryptoInstance<'r>> for WasmerScryptoModule<'l> {
    fn instantiate(&self, runtime: Box<dyn ScryptoRuntime + 'r>) -> WasmerScryptoInstance<'r> {
        // native functions
        fn radix_engine(
            env: &WasmerScryptoInstanceEnv,
            input_ptr: i32,
        ) -> Result<i32, RuntimeError> {
            let ptr = env.runtime_ptr.lock().unwrap();
            let runtime: &mut Box<dyn ScryptoRuntime> = unsafe { &mut *(*ptr as *mut _) };
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

        fn use_tbd(env: &WasmerScryptoInstanceEnv, tbd: i32) -> Result<(), RuntimeError> {
            let ptr = env.runtime_ptr.lock().unwrap();
            let runtime: &mut Box<dyn ScryptoRuntime> = unsafe { &mut *(*ptr as *mut _) };

            runtime
                .use_tbd(tbd as u32)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        // env
        let env = WasmerScryptoInstanceEnv {
            instance: LazyInit::new(),
            runtime_ptr: Arc::new(Mutex::new(&runtime as *const _ as usize)),
        };

        // imports
        let import_object = imports! {
            MODULE_ENV_NAME => {
                RADIX_ENGINE_FUNCTION_NAME => Function::new_native_with_env(&self.store, env.clone(), radix_engine),
                USE_TBD_FUNCTION_NAME => Function::new_native_with_env(&self.store, env, use_tbd),
            }
        };

        // instantiate
        let instance =
            Instance::new(&self.module, &import_object).expect("Failed to instantiate module");

        WasmerScryptoInstance { instance, runtime }
    }
}

impl<'r> ScryptoInstance for WasmerScryptoInstance<'r> {
    fn invoke_export(
        &mut self,
        export_name: &str,
        input: &ScryptoValue,
    ) -> Result<ScryptoValue, InvokeError> {
        let pointer = send_value(&self.instance, input)?;
        let result = self
            .instance
            .exports
            .get_function(export_name)
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
            .exports
            .iter()
            .filter(|e| matches!(e.1, Extern::Function(_)))
            .map(|e| e.0.to_string())
            .collect()
    }
}

impl WasmerEngine {
    pub fn new() -> Self {
        let compiler = Singlepass::new();
        let store = Store::new(&Universal::new(compiler).engine());
        Self { store }
    }
}

impl ScryptoValidator for WasmerEngine {
    fn validate(&mut self, _code: &[u8]) -> Result<(), WasmValidationError> {
        Ok(())
    }
}

impl ScryptoInstrumenter for WasmerEngine {
    fn instrument(&mut self, code: &[u8]) -> Result<Vec<u8>, InstrumentError> {
        Ok(code.to_vec())
    }
}

impl<'l, 'r> ScryptoLoader<'l, 'r, WasmerScryptoModule<'l>, WasmerScryptoInstance<'r>>
    for WasmerEngine
{
    fn load(&'l mut self, code: &[u8]) -> WasmerScryptoModule<'l> {
        let module = Module::new(&self.store, code).expect("Failed to parse wasm module");

        WasmerScryptoModule {
            module,
            store: &self.store,
        }
    }
}
