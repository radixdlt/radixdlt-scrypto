use sbor::rust::boxed::Box;
use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::abi::{Function, Method};
use scrypto::buffer::scrypto_decode;
use scrypto::core::ScryptoActorInfo;
use scrypto::prelude::PackagePublishInput;
use scrypto::values::ScryptoValue;

use crate::engine::*;
use crate::model::PackageError::MethodNotFound;
use crate::wasm::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ValidatedPackage {
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

impl ValidatedPackage {
    /// Validates and creates a package
    pub fn new(package: scrypto::prelude::Package) -> Result<Self, WasmValidationError> {
        let mut wasm_engine = WasmiEngine::new();
        wasm_engine.validate(&package.code)?;

        // instrument wasm
        let instrumented_code = wasm_engine
            .instrument(&package.code)
            .map_err(|_| WasmValidationError::FailedToInstrumentCode)?;

        Ok(Self {
            code: package.code,
            instrumented_code,
            blueprint_abis: package.blueprints,
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
        method_name: &str,
        call_data: ScryptoValue,
        system_api: &mut S,
    ) -> Result<ScryptoValue, PackageError> {
        match method_name {
            "publish" => {
                let input: PackagePublishInput =
                    scrypto_decode(&call_data.raw).map_err(|e| PackageError::InvalidRequestData(e))?;
                let package =
                    ValidatedPackage::new(input.package).map_err(PackageError::WasmValidationError)?;
                let package_address = system_api.create_package(package);
                Ok(ScryptoValue::from_value(&package_address))
            }
            _ => Err(MethodNotFound(method_name.to_string()))
        }
    }

    pub fn invoke<S: SystemApi>(
        &self,
        actor: ScryptoActorInfo,
        export_name: String,
        call_data: ScryptoValue,
        system_api: &mut S,
    ) -> Result<ScryptoValue, RuntimeError> {
        #[cfg(feature = "wasmer")]
        let mut engine = WasmerEngine::new();
        #[cfg(not(feature = "wasmer"))]
        let mut engine = WasmiEngine::new();
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
