use sbor::rust::boxed::Box;
use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::abi::{Function, Method};
use scrypto::buffer::scrypto_decode;
use scrypto::component::PackageFunction;
use scrypto::core::ScryptoActorInfo;
use scrypto::values::ScryptoValue;

use crate::engine::*;
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
        let mut engine = WasmerEngine::new();
        let runtime = NopScryptoRuntime::new(EXPORT_BLUEPRINT_ABI_TBD_LIMIT); // stateless

        // validate wasm
        engine.validate(&code)?;

        // instrument wasm
        let instrumented_code = engine
            .instrument(&code)
            .map_err(|_| WasmValidationError::FailedToInstrumentCode)?;

        // TODO replace this with static ABIs
        // export blueprint ABI
        let mut blueprint_abis = HashMap::new();
        let module = engine.load(&code);
        let mut instance = module.instantiate(Box::new(runtime));
        let exports: Vec<String> = instance
            .function_exports()
            .into_iter()
            .filter(|e| e.ends_with("_abi") && e.len() > 4)
            .collect();
        for method_name in exports {
            let return_data = instance
                .invoke_export(&method_name, &ScryptoValue::unit())
                .map_err(|_| WasmValidationError::FailedToExportBlueprintAbi)?;

            let abi: (Type, Vec<Function>, Vec<Method>) = scrypto_decode(&return_data.raw)
                .map_err(|_| WasmValidationError::InvalidBlueprintAbi)?;

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

    pub fn invoke<S: SystemApi>(
        &self,
        actor: ScryptoActorInfo,
        export_name: String,
        call_data: ScryptoValue,
        system_api: &mut S,
    ) -> Result<ScryptoValue, RuntimeError> {
        let mut engine = WasmerEngine::new();
        let runtime = RadixEngineScryptoRuntime::new(actor, system_api, CALL_FUNCTION_TBD_LIMIT);
        let module = engine.load(self.instrumented_code());
        let mut instance = module.instantiate(Box::new(runtime));
        instance
            .invoke_export(&export_name, &call_data)
            .map_err(|e| match e {
                // Flatten error code for more readable transaction receipt
                InvokeError::RuntimeError(e) => e,
                e @ _ => RuntimeError::InvokeError(e.into()),
            })
    }
}
