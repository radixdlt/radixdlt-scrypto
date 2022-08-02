use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::abi::BlueprintAbi;
use scrypto::buffer::scrypto_decode;
use scrypto::prelude::{PackageAddress, PackagePublishInput};
use scrypto::values::ScryptoValue;

use crate::engine::*;
use crate::fee::{CostUnitCounter, CostUnitCounterError};
use crate::wasm::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
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
    CostingError(CostUnitCounterError),
}

impl ValidatedPackage {
    pub fn new(package: scrypto::prelude::Package) -> Result<Self, PrepareError> {
        WasmValidator::default().validate(&package.code, &package.blueprints)?;

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

    pub fn static_main<'p, 's, Y, W, I, C>(
        method_name: &str,
        call_data: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, PackageError>
    where
        Y: SystemApi<'p, 's, W, I, C>,
        W: WasmEngine<I>,
        I: WasmInstance,
        C: CostUnitCounter,
    {
        match method_name {
            "publish" => {
                let input: PackagePublishInput = scrypto_decode(&call_data.raw)
                    .map_err(|e| PackageError::InvalidRequestData(e))?;
                let package =
                    ValidatedPackage::new(input.package).map_err(PackageError::InvalidWasm)?;
                let node_id = system_api.create_node(package).unwrap(); // FIXME: update all `create_value` calls to handle errors correctly
                system_api.node_globalize(&node_id).map_err(|e| match e {
                    RuntimeError::CostingError(cost_unit_error) => {
                        PackageError::CostingError(cost_unit_error)
                    }
                    _ => panic!("Unexpected error {}", e),
                })?;
                let package_address: PackageAddress = node_id.into();
                Ok(ScryptoValue::from_typed(&package_address))
            }
            _ => Err(PackageError::MethodNotFound(method_name.to_string())),
        }
    }
}
