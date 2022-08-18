use core::fmt::Debug;

use crate::engine::*;
use crate::fee::{FeeReserve, FeeReserveError};
use crate::types::*;
use crate::wasm::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, TypeId, Encode, Decode, PartialEq, Eq)]
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
    CostingError(FeeReserveError),
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

    pub fn static_main<'s, Y, W, I, C>(
        package_fn: PackageFnIdentifier,
        call_data: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, PackageError>
    where
        Y: SystemApi<'s, W, I, C>,
        W: WasmEngine<I>,
        I: WasmInstance,
        C: FeeReserve,
    {
        match package_fn {
            PackageFnIdentifier::Publish => {
                let input: PackagePublishInput = scrypto_decode(&call_data.raw)
                    .map_err(|e| PackageError::InvalidRequestData(e))?;
                let package =
                    ValidatedPackage::new(input.package).map_err(PackageError::InvalidWasm)?;
                let node_id = system_api
                    .node_create(HeapRENode::Package(package))
                    .unwrap(); // FIXME: update all `create_value` calls to handle errors correctly
                system_api.node_globalize(node_id).map_err(|e| match e {
                    RuntimeError::CostingError(cost_unit_error) => {
                        PackageError::CostingError(cost_unit_error)
                    }
                    _ => panic!("Unexpected error {}", e),
                })?;
                let package_address: PackageAddress = node_id.into();
                Ok(ScryptoValue::from_typed(&package_address))
            }
        }
    }
}

impl Debug for ValidatedPackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatedPackage")
            .field("code_len", &self.code.len())
            .field("blueprint_abis", &self.blueprint_abis)
            .finish()
    }
}
