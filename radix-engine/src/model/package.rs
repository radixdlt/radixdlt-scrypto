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
    pub fn new<'w, W, I>(code: Vec<u8>, wasm_engine: &'w mut W) -> Result<Self, WasmValidationError>
    where
        W: WasmEngine<I>,
        I: WasmInstance,
    {
        // stateless runtime
        let runtime = NopWasmRuntime::new(EXPORT_ABI_TBD_LIMIT);
        let mut runtime_boxed: Box<dyn WasmRuntime> = Box::new(runtime);

        // validate wasm
        wasm_engine.validate(&code)?;

        // instrument wasm
        wasm_engine
            .instrument(&code)
            .map_err(|_| WasmValidationError::FailedToInstrumentCode)?;

        // TODO replace this with static ABIs
        // export blueprint ABI
        let mut blueprint_abis = HashMap::new();
        let exports: Vec<String> = wasm_engine
            .instantiate(&code)
            .function_exports()
            .into_iter()
            .filter(|e| e.ends_with("_abi") && e.len() > 4)
            .collect();
        for method_name in exports {
            let return_data = wasm_engine
                .instantiate(&code)
                .invoke_export(&method_name, &ScryptoValue::unit(), &mut runtime_boxed)
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
            blueprint_abis,
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
                let package = Package::new(bytes, system_api.wasm_engine())
                    .map_err(PackageError::WasmValidationError)?;
                let package_address = system_api.create_package(package);
                Ok(ScryptoValue::from_value(&package_address))
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
