use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::abi::{Function, Method};
use scrypto::buffer::scrypto_decode;
use scrypto::component::PackageFunction;
use scrypto::values::ScryptoValue;

use crate::engine::SystemApi;
use crate::wasm::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Package {
    code: Vec<u8>,
    instrumented_code: Vec<u8>,
    blueprint_abis: HashMap<String, (Type, Vec<Function>, Vec<Method>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageError {
    InvalidRequestData(DecodeError),
    BlueprintNotFound,
    WasmValidationError(WasmValidationError),
    MethodNotFound(String),
}

impl Package {
    /// Validates and creates a package
    pub fn new(code: Vec<u8>) -> Result<Self, WasmValidationError> {
        let mut wasm_engine = WasmiEngine::new(); // stateless

        // validate wasm
        wasm_engine.validate(&code)?;

        // instrument wasm
        let instrumented_code = wasm_engine
            .instrument(&code)
            .map_err(|_| WasmValidationError::FailedToInstrumentCode)?;

        // export blueprint ABI
        let mut blueprint_abis = HashMap::new();
        let module = wasm_engine.instantiate(&code);
        let exports: Vec<String> = module
            .function_exports()
            .into_iter()
            .filter(|e| e.ends_with("_abi") && e.len() > 4)
            .collect();
        for method_name in exports {
            let rtn = module
                .invoke_export(
                    &method_name,
                    &ScryptoValue::unit(),
                    &mut NopScryptoRuntime::new(EXPORT_BLUEPRINT_ABI_TBD_LIMIT),
                )
                .map_err(|_| WasmValidationError::FailedToExportBlueprintAbi)?;

            let abi: (Type, Vec<Function>, Vec<Method>) =
                scrypto_decode(&rtn.raw).map_err(|_| WasmValidationError::InvalidBlueprintAbi)?;

            if let Type::Struct { name, fields: _ } = &abi.0 {
                blueprint_abis.insert(name.clone(), abi);
            } else {
                return Err(WasmValidationError::InvalidBlueprintAbi);
            }
        }

        Ok(Self {
            code,
            instrumented_code,
            blueprint_abis,
        })
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn instrumented_code(&self) -> &[u8] {
        &self.instrumented_code
    }

    pub fn blueprint_abi(
        &self,
        blueprint_name: &str,
    ) -> Option<&(Type, Vec<Function>, Vec<Method>)> {
        self.blueprint_abis.get(blueprint_name)
    }

    pub fn contains_blueprint(&self, blueprint_name: &str) -> bool {
        self.blueprint_abis.contains_key(blueprint_name)
    }

    pub fn load_blueprint_schema(&self, blueprint_name: &str) -> Result<&Type, PackageError> {
        self.blueprint_abi(blueprint_name)
            .map(|v| &v.0)
            .ok_or(PackageError::BlueprintNotFound)
    }

    pub fn static_main<S: SystemApi>(
        call_data: ScryptoValue,
        system_api: &mut S,
    ) -> Result<ScryptoValue, PackageError> {
        let function: PackageFunction =
            scrypto_decode(&call_data.raw).map_err(|e| PackageError::InvalidRequestData(e))?;
        match function {
            PackageFunction::Publish(bytes) => {
                let package = Package::new(bytes).map_err(PackageError::WasmValidationError)?;
                let package_address = system_api.create_package(package);
                Ok(ScryptoValue::from_value(&package_address))
            }
        }
    }
}
