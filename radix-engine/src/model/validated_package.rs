use sbor::rust::boxed::Box;
use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::abi::BlueprintAbi;
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
    blueprint_abis: HashMap<String, BlueprintAbi>,
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
        WasmValidator::validate(&package.code, &package.blueprints)?;

        Ok(Self {
            code: package.code,
            blueprint_abis: package.blueprints,
        })
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn blueprint_abi(&self, blueprint_name: &str) -> Option<&BlueprintAbi> {
        self.blueprint_abis.get(blueprint_name)
    }

    pub fn static_main<'s, S, W, I>(
        method_name: &str,
        call_data: ScryptoValue,
        system_api: &mut S,
    ) -> Result<ScryptoValue, PackageError>
    where
        S: SystemApi<W, I>,
        W: WasmEngine<I>,
        I: WasmInstance,
    {
        match method_name {
            "publish" => {
                let input: PackagePublishInput = scrypto_decode(&call_data.raw)
                    .map_err(|e| PackageError::InvalidRequestData(e))?;
                let package =
                    ValidatedPackage::new(input.package).map_err(PackageError::InvalidWasm)?;
                let package_address = system_api.create_package(package);
                Ok(ScryptoValue::from_typed(&package_address))
            }
            _ => Err(MethodNotFound(method_name.to_string())),
        }
    }

    pub fn invoke<'s, S, W, I>(
        &self,
        actor: &ScryptoActorInfo,
        fn_ident: &str,
        input: ScryptoValue,
        system_api: &mut S,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        S: SystemApi<W, I>,
        W: WasmEngine<I>,
        I: WasmInstance,
    {
        let wasm_metering_params = system_api.fee_table().wasm_metering_params();
        let instrumented_code = system_api
            .wasm_instrumenter()
            .instrument(&self.code, &wasm_metering_params);
        let mut instance = system_api.wasm_engine().instantiate(&instrumented_code);
        let blueprint_abi = self
            .blueprint_abi(actor.blueprint_name())
            .expect("Blueprint should exist");
        let export_name = &blueprint_abi.get_fn_abi(fn_ident).unwrap().export_name;
        let mut runtime: Box<dyn WasmRuntime> = Box::new(RadixEngineWasmRuntime::new(
            actor.clone(),
            blueprint_abi,
            system_api,
        ));
        let output = instance
            .invoke_export(export_name, &input, &mut runtime)
            .map_err(|e| match e {
                // Flatten error code for more readable transaction receipt
                InvokeError::RuntimeError(e) => e,
                e @ _ => RuntimeError::InvokeError(e.into()),
            })?;

        let fn_abi = blueprint_abi.get_fn_abi(fn_ident).unwrap();
        if !fn_abi.output.matches(&output.dom) {
            Err(RuntimeError::InvalidFnOutput {
                fn_ident: fn_ident.to_string(),
                output: output.dom,
            })
        } else {
            Ok(output)
        }
    }
}
