use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::values::ScryptoValue;
use wasm_instrument::{gas_metering, inject_stack_limiter, parity_wasm};
use wasmi::*;

pub struct WasmiScryptoModule {
    module_ref: ModuleRef,
    memory_ref: MemoryRef,
}
pub struct WasmiScryptoModuleExternals<'a, T: ScryptoRuntime> {
    module: &'a WasmiScryptoModule,
    runtime: &'a mut T,
}

pub struct WasmiEngine {}

pub struct WasmiEnvModule;

impl ModuleImportResolver for WasmiEnvModule {
    fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
        match field_name {
            ENGINE_FUNCTION_NAME => {
                if signature.params() != [ValueType::I32]
                    || signature.return_type() != Some(ValueType::I32)
                {
                    return Err(Error::Instantiation(
                        "Function signature does not match".into(),
                    ));
                }
                Ok(FuncInstance::alloc_host(
                    signature.clone(),
                    ENGINE_FUNCTION_INDEX,
                ))
            }
            TBD_FUNCTION_NAME => {
                if signature.params() != [ValueType::I32] || signature.return_type() != None {
                    return Err(Error::Instantiation(
                        "Function signature does not match".into(),
                    ));
                }
                Ok(FuncInstance::alloc_host(
                    signature.clone(),
                    TBD_FUNCTION_INDEX,
                ))
            }
            _ => Err(Error::Instantiation(format!(
                "Function {} not found",
                field_name
            ))),
        }
    }
}

impl WasmiScryptoModule {
    pub fn send_value<T: ScryptoRuntime>(
        &self,
        value: &ScryptoValue,
        externals: &mut WasmiScryptoModuleExternals<T>,
    ) -> Result<RuntimeValue, InvokeError> {
        let result = self.module_ref.invoke_export(
            EXPORT_SCRYPTO_ALLOC,
            &[RuntimeValue::I32((value.raw.len()) as i32)],
            externals,
        );

        if let Ok(Some(RuntimeValue::I32(ptr))) = result {
            if self.memory_ref.set((ptr + 4) as u32, &value.raw).is_ok() {
                return Ok(RuntimeValue::I32(ptr));
            }
        }

        Err(InvokeError::MemoryAllocError)
    }

    pub fn read_value(&self, ptr: u32) -> Result<ScryptoValue, InvokeError> {
        let len: u32 = self
            .memory_ref
            .get_value(ptr)
            .map_err(|_| InvokeError::MemoryAccessError)?;

        let start = ptr.checked_add(4).ok_or(InvokeError::MemoryAccessError)?;
        let end = start
            .checked_add(len)
            .ok_or(InvokeError::MemoryAccessError)?;
        let range = start as usize..end as usize;
        let direct = self.memory_ref.direct_access();
        let buffer = direct.as_ref();

        if end > buffer.len().try_into().unwrap() {
            return Err(InvokeError::MemoryAccessError);
        }

        ScryptoValue::from_slice(&buffer[range]).map_err(InvokeError::InvalidScryptoValue)
    }
}

impl<'a, T: ScryptoRuntime> Externals for WasmiScryptoModuleExternals<'a, T> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            ENGINE_FUNCTION_INDEX => {
                let input_ptr: u32 = args.nth_checked(0)?;
                let input = self.module.read_value(input_ptr)?;

                let output = self.runtime.main(input)?;
                self.module
                    .send_value(&output, self)
                    .map(Option::Some)
                    .map_err(|e| e.into())
            }
            TBD_FUNCTION_INDEX => {
                let amount: u32 = args.nth_checked(0)?;
                self.runtime
                    .use_tbd(amount)
                    .map(|_| Option::None)
                    .map_err(|e| e.into())
            }
            _ => Err(InvokeError::FunctionNotFound.into()),
        }
    }
}

impl ScryptoModule for WasmiScryptoModule {
    fn invoke_export<R: ScryptoRuntime>(
        &self,
        export_name: &str,
        input: &ScryptoValue,
        runtime: &mut R,
    ) -> Result<ScryptoValue, InvokeError> {
        let mut externals = WasmiScryptoModuleExternals {
            module: self,
            runtime,
        };
        let pointer = self.send_value(input, &mut externals)?;
        let result = self
            .module_ref
            .invoke_export(export_name, &[pointer], &mut externals);

        let rtn = result
            .map_err(|e| {
                let e_str = format!("{:?}", e);
                match e.into_host_error() {
                    // Pass-through invoke errors
                    Some(host_error) => *host_error.downcast::<InvokeError>().unwrap(),
                    None => InvokeError::WasmError(e_str),
                }
            })?
            .ok_or(InvokeError::MissingReturnData)?;
        match rtn {
            RuntimeValue::I32(ptr) => self.read_value(ptr as u32),
            _ => Err(InvokeError::InvalidReturnData),
        }
    }

    fn function_exports(&self) -> Vec<String> {
        self.module_ref
            .exports()
            .iter()
            .filter(|(_, val)| matches!(val, ExternVal::Func(_)))
            .map(|(name, _)| name.to_string())
            .collect()
    }
}

impl WasmiEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl ScryptoWasmValidator for WasmiEngine {
    fn validate(&mut self, code: &[u8]) -> Result<(), WasmValidationError> {
        // parse wasm module
        let module = Module::from_buffer(code).map_err(|_| WasmValidationError::FailedToParse)?;

        // check floating point
        module
            .deny_floating_point()
            .map_err(|_| WasmValidationError::FloatingPointNotAllowed)?;

        // Instantiate
        let instance = ModuleInstance::new(
            &module,
            &ImportsBuilder::new().with_resolver("env", &WasmiEnvModule),
        )
        .map_err(|e| WasmValidationError::FailedToInstantiate(e.to_string()))?;

        // Check start function
        if instance.has_start() {
            return Err(WasmValidationError::StartFunctionNotAllowed);
        }
        let module_ref = instance.assert_no_start();

        // Check memory export
        match module_ref.export_by_name(EXPORT_MEMORY) {
            Some(ExternVal::Memory(_)) => {}
            _ => {
                return Err(WasmValidationError::NoMemoryExport);
            }
        }

        // Check scrypto abi
        match module_ref.export_by_name(EXPORT_SCRYPTO_ALLOC) {
            Some(ExternVal::Func(_)) => {}
            _ => {
                return Err(WasmValidationError::NoScryptoAllocExport);
            }
        }
        match module_ref.export_by_name(EXPORT_SCRYPTO_FREE) {
            // TODO: check if this is indeed needed
            Some(ExternVal::Func(_)) => {}
            _ => {
                return Err(WasmValidationError::NoScryptoFreeExport);
            }
        }

        Ok(())
    }
}

impl ScryptoWasmInstrumenter for WasmiEngine {
    fn instrument(&mut self, code: &[u8]) -> Result<Vec<u8>, InstrumentError> {
        let mut module =
            parity_wasm::deserialize_buffer(code).expect("Unable to parse wasm module");

        // TODO reject wasm modules that call the `TBD_FUNCTION_INDEX` directly

        module = gas_metering::inject(
            module,
            &gas_metering::ConstantCostRules::new(INSTRUCTION_COST, MEMORY_GROW_COST),
            MODULE_ENV_NAME,
        )
        .map_err(|_| InstrumentError::FailedToInjectInstructionMetering)?;

        module = inject_stack_limiter(module, MAX_STACK_DEPTH)
            .map_err(|_| InstrumentError::FailedToInjectStackLimiter)?;

        module
            .to_bytes()
            .map_err(|_| InstrumentError::FailedToExportModule)
    }
}

impl ScryptoWasmExecutor<WasmiScryptoModule> for WasmiEngine {
    fn instantiate(&mut self, code: &[u8]) -> WasmiScryptoModule {
        // parse wasm
        let module = Module::from_buffer(code).expect("Failed to parse wasm module");

        // link with env module
        let module_ref = ModuleInstance::new(
            &module,
            &ImportsBuilder::new().with_resolver(MODULE_ENV_NAME, &WasmiEnvModule),
        )
        .expect("Failed to instantiate wasm module")
        .assert_no_start();

        // find memory ref
        let memory_ref = match module_ref.export_by_name(EXPORT_MEMORY) {
            Some(ExternVal::Memory(memory)) => memory,
            _ => panic!("Failed to find memory export"),
        };

        WasmiScryptoModule {
            module_ref,
            memory_ref,
        }
    }
}
