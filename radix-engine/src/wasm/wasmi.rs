use wasmi::*;

use crate::types::*;
use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;

pub struct WasmiModule {
    module: Module,
}

pub struct WasmiInstance {
    module_ref: ModuleRef,
    memory_ref: MemoryRef,
}

pub struct WasmiExternals<'a, 'b, 'r> {
    instance: &'a WasmiInstance,
    runtime: &'b mut Box<dyn WasmRuntime + 'r>,
}

pub struct WasmiEnvModule {}

pub struct WasmiEngine {
    modules: HashMap<Hash, WasmiModule>,
}

impl ModuleImportResolver for WasmiEnvModule {
    fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
        match field_name {
            RADIX_ENGINE_FUNCTION_NAME => {
                if signature.params() != [ValueType::I32]
                    || signature.return_type() != Some(ValueType::I32)
                {
                    return Err(Error::Instantiation(
                        "Function signature does not match".into(),
                    ));
                }
                Ok(FuncInstance::alloc_host(
                    signature.clone(),
                    RADIX_ENGINE_FUNCTION_INDEX,
                ))
            }
            CONSUME_COST_UNITS_FUNCTION_NAME => {
                if signature.params() != [ValueType::I32] || signature.return_type() != None {
                    return Err(Error::Instantiation(
                        "Function signature does not match".into(),
                    ));
                }
                Ok(FuncInstance::alloc_host(
                    signature.clone(),
                    CONSUME_COST_UNITS_FUNCTION_INDEX,
                ))
            }
            _ => Err(Error::Instantiation(format!(
                "Function {} not found",
                field_name
            ))),
        }
    }
}

impl WasmiModule {
    fn instantiate(&self) -> WasmiInstance {
        // link with env module
        let module_ref = ModuleInstance::new(
            &self.module,
            &ImportsBuilder::new().with_resolver(MODULE_ENV_NAME, &WasmiEnvModule {}),
        )
        .expect("Failed to instantiate WASM module")
        .assert_no_start();

        // find memory ref
        let memory_ref = match module_ref.export_by_name(EXPORT_MEMORY) {
            Some(ExternVal::Memory(memory)) => memory,
            _ => panic!("Failed to find memory export"),
        };

        WasmiInstance {
            module_ref,
            memory_ref,
        }
    }
}

impl<'a, 'b, 'r> WasmiExternals<'a, 'b, 'r> {
    pub fn send_value(&mut self, value: &ScryptoValue) -> Result<RuntimeValue, WasmInvokeError> {
        let result = self.instance.module_ref.clone().invoke_export(
            EXPORT_SCRYPTO_ALLOC,
            &[RuntimeValue::I32((value.raw.len()) as i32)],
            self,
        );

        if let Ok(Some(RuntimeValue::I32(ptr))) = result {
            if self
                .instance
                .memory_ref
                .set((ptr + 4) as u32, &value.raw)
                .is_ok()
            {
                return Ok(RuntimeValue::I32(ptr));
            }
        }

        Err(WasmInvokeError::MemoryAllocError)
    }

    pub fn read_value(&self, ptr: usize) -> Result<ScryptoValue, WasmInvokeError> {
        let len = self
            .instance
            .memory_ref
            .get_value::<u32>(ptr as u32)
            .map_err(|_| WasmInvokeError::MemoryAccessError)? as usize;

        let start = ptr
            .checked_add(4)
            .ok_or(WasmInvokeError::MemoryAccessError)?;
        let end = start
            .checked_add(len)
            .ok_or(WasmInvokeError::MemoryAccessError)?;

        let direct = self.instance.memory_ref.direct_access();
        let buffer = direct.as_ref();
        if end > buffer.len() {
            return Err(WasmInvokeError::MemoryAccessError);
        }

        ScryptoValue::from_slice(&buffer[start..end]).map_err(WasmInvokeError::InvalidScryptoValue)
    }
}

impl<'a, 'b, 'r> Externals for WasmiExternals<'a, 'b, 'r> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            RADIX_ENGINE_FUNCTION_INDEX => {
                let input_ptr = args.nth_checked::<u32>(0)? as usize;
                let input = self.read_value(input_ptr)?;
                let output = self.runtime.main(input)?;
                self.send_value(&output)
                    .map(Option::Some)
                    .map_err(|e| e.into())
            }
            CONSUME_COST_UNITS_FUNCTION_INDEX => {
                let n: u32 = args.nth_checked(0)?;
                self.runtime
                    .consume_cost_units(n)
                    .map(|_| Option::None)
                    .map_err(|e| e.into())
            }
            _ => Err(WasmInvokeError::FunctionNotFound.into()),
        }
    }
}

impl WasmInstance for WasmiInstance {
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: &ScryptoValue,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<ScryptoValue, WasmInvokeError> {
        let mut externals = WasmiExternals {
            instance: self,
            runtime,
        };

        let pointer = externals.send_value(args)?;
        let result = self
            .module_ref
            .clone()
            .invoke_export(func_name, &[pointer], &mut externals);

        let rtn = result
            .map_err(|e| {
                let e_str = format!("{:?}", e);
                match e.into_host_error() {
                    // Pass-through invoke errors
                    Some(host_error) => *host_error
                        .downcast::<WasmInvokeError>()
                        .expect("Failed to downcast error into WasmInvokeError"),
                    None => WasmInvokeError::WasmError(e_str),
                }
            })?
            .ok_or(WasmInvokeError::MissingReturnData)?;
        match rtn {
            RuntimeValue::I32(ptr) => externals.read_value(ptr as usize),
            _ => Err(WasmInvokeError::InvalidReturnData),
        }
    }
}

impl WasmiEngine {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }
}

impl WasmEngine<WasmiInstance> for WasmiEngine {
    fn instantiate(&mut self, code: &[u8]) -> WasmiInstance {
        let code_hash = hash(code);
        self.modules
            .entry(code_hash)
            .or_insert_with(|| WasmiModule {
                module: Module::from_buffer(code).expect("Failed to parse WASM module"),
            })
            .instantiate()
    }
}
