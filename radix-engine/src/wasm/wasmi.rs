use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;
use scrypto::rust::format;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::values::ScryptoValue;
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
            _ => Err(Error::Instantiation(format!(
                "Export {} not found",
                field_name
            ))),
        }
    }
}

impl WasmiScryptoModule {
    pub fn send_value(&self, value: &ScryptoValue) -> Result<RuntimeValue, InvokeError> {
        let result = self.module_ref.invoke_export(
            EXPORT_SCRYPTO_ALLOC,
            &[RuntimeValue::I32((value.raw.len()) as i32)],
            &mut NopExternals,
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
        if index != ENGINE_FUNCTION_INDEX {
            return Err(InvokeError::FunctionNotFound.into());
        }

        let input_ptr: u32 = args.nth_checked(0)?;
        let input = self.module.read_value(input_ptr)?;

        if let sbor::Value::Enum { name, fields } = input.dom {
            let output = self.runtime.main(
                name.as_str(),
                &[ScryptoValue::from_any(&fields[0]).expect("FIXME")],
            )?;

            self.module
                .send_value(&output)
                .map(Option::Some)
                .map_err(|e| e.into())
        } else {
            Err(InvokeError::InvalidCallData.into())
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
        let pointer = self.send_value(input)?;
        let mut externals = WasmiScryptoModuleExternals {
            module: self,
            runtime,
        };
        let result = self
            .module_ref
            .invoke_export(export_name, &[pointer], &mut externals);

        let rtn = result
            .map_err(|e| {
                match e.into_host_error() {
                    // Pass-through invoke errors
                    Some(host_error) => *host_error.downcast::<InvokeError>().unwrap(),
                    None => InvokeError::WasmError,
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

impl ScryptoWasmExecutor<WasmiScryptoModule> for WasmiEngine {
    fn instantiate(&mut self, code: &[u8]) -> WasmiScryptoModule {
        // parse wasm
        let module = Module::from_buffer(code).expect("Failed to parse wasm module");

        // link with env module
        let module_ref = ModuleInstance::new(
            &module,
            &ImportsBuilder::new().with_resolver(EXPORT_ENV, &WasmiEnvModule),
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
