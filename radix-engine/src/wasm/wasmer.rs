use sbor::rust::ptr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::values::ScryptoValue;
use wasmer::*;

use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;

pub struct WasmerScryptoModule<'l> {
    module: Module,
    store: &'l Store,
}

#[derive(Clone, WasmerEnv)]
pub struct WasmerScryptoInstance {
    instance: Instance,
}

#[derive(Clone)]
pub struct WasmerScryptoInstanceEnv {
    instance: LazyInit<WasmerScryptoInstance>,
    runtime: usize,
}

pub struct WasmerEngine {
    store: Store,
}

impl WasmerEnv for WasmerScryptoInstanceEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.instance.initialize(WasmerScryptoInstance {
            instance: instance.clone(),
        });
        Ok(())
    }
}

impl<'l, 'r, R: ScryptoRuntime + 'static> ScryptoModule<'r, WasmerScryptoInstance, R>
    for WasmerScryptoModule<'l>
{
    fn instantiate(&self, runtime: &'r mut R) -> WasmerScryptoInstance {
        // native functions
        fn radix_engine<R: ScryptoRuntime>(
            env: &WasmerScryptoInstanceEnv,
            input_ptr: i32,
        ) -> Result<i32, RuntimeError> {
            let input = unsafe { env.instance.get_unchecked() }
                .read_value(input_ptr as usize)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            let runtime = unsafe { &mut *(env.runtime as *mut R) };
            let output = runtime
                .main(input)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            unsafe { env.instance.get_unchecked() }
                .send_value(&output)
                .map(|ptr| ptr as i32)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        fn use_tbd<R: ScryptoRuntime>(
            env: &WasmerScryptoInstanceEnv,
            tbd: i32,
        ) -> Result<(), RuntimeError> {
            let runtime = unsafe { &mut *(env.runtime as *mut R) };
            runtime
                .use_tbd(tbd as u32)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        // env
        let env = WasmerScryptoInstanceEnv {
            instance: LazyInit::new(),
            // TODO: this is very bad practice; it requires the caller to ensure the
            // runtime object lives longer than the intended wasm execution and it's not
            // thread-safe.
            runtime: runtime as *mut R as usize,
        };

        // imports
        let import_object = imports! {
            MODULE_ENV_NAME => {
                RADIX_ENGINE_FUNCTION_NAME => Function::new_native_with_env(&self.store, env.clone(), radix_engine::<R>),
                USE_TBD_FUNCTION_NAME => Function::new_native_with_env(&self.store, env, use_tbd::<R>),
            }
        };

        // instantiate
        let instance =
            Instance::new(&self.module, &import_object).expect("Failed to instantiate module");

        WasmerScryptoInstance { instance }
    }
}

impl WasmerScryptoInstance {
    pub fn send_value(&self, value: &ScryptoValue) -> Result<usize, InvokeError> {
        let slice = &value.raw;
        let n = slice.len();

        let result = self
            .instance
            .exports
            .get_function(EXPORT_SCRYPTO_ALLOC)
            .map_err(|_| InvokeError::MemoryAllocError)?
            .call(&[Val::I32(n as i32)])
            .map_err(|_| InvokeError::MemoryAllocError)?;

        if let Some(Value::I32(ptr)) = result.as_ref().get(0) {
            let ptr = *ptr as usize;
            let memory = self
                .instance
                .exports
                .get_memory(EXPORT_MEMORY)
                .map_err(|_| InvokeError::MemoryAllocError)?;
            let size = memory.size().bytes().0;
            if size > ptr && size - ptr >= n {
                unsafe {
                    let dest = memory.data_ptr().add(ptr);
                    ptr::copy(slice.as_ptr(), dest, n);
                }
                return Ok(ptr);
            }
        }

        Err(InvokeError::MemoryAllocError)
    }

    pub fn read_value(&self, ptr: usize) -> Result<ScryptoValue, InvokeError> {
        let memory = self
            .instance
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
}

impl ScryptoInstance for WasmerScryptoInstance {
    fn invoke_export(
        &mut self,
        export_name: &str,
        input: &ScryptoValue,
    ) -> Result<ScryptoValue, InvokeError> {
        let pointer = self.send_value(input)?;
        let result = self
            .instance
            .exports
            .get_function(export_name)
            .map_err(|_| InvokeError::FunctionNotFound)?
            .call(&[Val::I32(pointer as i32)]);

        println!("{:?}", export_name);
        println!("{:?}", input);
        println!("{:?}", result);

        match result {
            Ok(return_data) => {
                let ptr = return_data
                    .as_ref()
                    .get(0)
                    .ok_or(InvokeError::MissingReturnData)?
                    .i32()
                    .ok_or(InvokeError::InvalidReturnData)?;
                self.read_value(ptr as usize)
            }
            _ => Err(InvokeError::InvalidReturnData),
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
        Self {
            store: Store::default(),
        }
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

impl<'l, 'r, R: ScryptoRuntime + 'static>
    ScryptoLoader<'l, 'r, WasmerScryptoModule<'l>, WasmerScryptoInstance, R> for WasmerEngine
{
    fn load(&'l mut self, code: &[u8]) -> WasmerScryptoModule<'l> {
        let module = Module::new(&self.store, code).expect("Failed to parse wasm module");

        WasmerScryptoModule {
            module,
            store: &self.store,
        }
    }
}
