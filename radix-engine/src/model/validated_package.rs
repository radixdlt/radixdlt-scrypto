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
pub struct ValidatedPackage {
    code: Vec<u8>,
    blueprint_abis: HashMap<String, (Type, Vec<Function>, Vec<Method>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageError {
    InvalidRequestData(DecodeError),
    InvalidWasm(PrepareError),
    BlueprintNotFound,
    MethodNotFound(String),
}

impl ValidatedPackage {
    pub fn new(package: scrypto::prelude::Package) -> Result<Self, PrepareError> {
        WasmModule::init(&package.code)?
            .reject_floating_point()?
            .reject_start_function()?
            .check_imports()?
            .check_memory()?
            .enforce_initial_memory_limit()?
            .enforce_functions_limit()?
            .enforce_locals_limit()?
            .inject_instruction_metering()?
            .inject_stack_metering()?
            .to_bytes()?;

        Ok(Self {
            code: package.code,
            blueprint_abis: package.blueprints,
        })
    }

    pub fn code(&self) -> &[u8] {
        &self.code
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

    pub fn static_main<'s, S, W, I>(
        call_data: ScryptoValue,
        system_api: &'s mut S,
    ) -> Result<ScryptoValue, PackageError>
    where
        S: SystemApi<W, I>,
        W: WasmEngine<I>,
        I: WasmInstance,
    {
        let function: PackageFunction =
            scrypto_decode(&call_data.raw).map_err(|e| PackageError::InvalidRequestData(e))?;
        match function {
            PackageFunction::Publish(bytes) => {
                let package = ValidatedPackage::new(bytes).map_err(PackageError::InvalidWasm)?;
                let package_address = system_api.create_package(package);
                Ok(ScryptoValue::from_typed(&package_address))
            }
        }
    }

    pub fn invoke<'s, S, W, I>(
        &self,
        actor: ScryptoActorInfo,
        export_name: String,
        call_data: ScryptoValue,
        system_api: &'s mut S,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        S: SystemApi<W, I>,
        W: WasmEngine<I>,
        I: WasmInstance,
    {
        let mut instance = system_api.wasm_engine().instantiate(self.code());
        let runtime = RadixEngineWasmRuntime::new(actor, system_api, CALL_FUNCTION_TBD_LIMIT);
        let mut runtime_boxed: Box<dyn WasmRuntime> = Box::new(runtime);
        instance
            .invoke_export(&export_name, &call_data, &mut runtime_boxed)
            .map_err(|e| match e {
                // Flatten error code for more readable transaction receipt
                InvokeError::RuntimeError(e) => e,
                e @ _ => RuntimeError::InvokeError(e.into()),
            })
    }
}
