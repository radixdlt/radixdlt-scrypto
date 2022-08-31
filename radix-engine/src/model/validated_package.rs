use core::fmt::Debug;

use crate::engine::*;
use crate::fee::FeeReserve;
use crate::types::*;
use crate::wasm::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct ValidatedPackage {
    code: Vec<u8>,
    blueprint_abis: HashMap<String, BlueprintAbi>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ValidatedPackageError {
    InvalidRequestData(DecodeError),
    InvalidPackage(DecodeError),
    InvalidWasm(PrepareError),
    BlueprintNotFound,
    MethodNotFound(String),
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

    pub fn static_main<'s, Y, W, I, R>(
        package_fn: PackageFnIdentifier,
        call_data: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ValidatedPackageError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match package_fn {
            PackageFnIdentifier::Publish => {
                let input: PackagePublishInput = scrypto_decode(&call_data.raw).map_err(|e| {
                    InvokeError::Error(ValidatedPackageError::InvalidRequestData(e))
                })?;
                let package = system_api
                    .read_blob(&input.package_blob.0)
                    .map_err(InvokeError::Downstream)
                    .and_then(|blob| {
                        scrypto_decode::<Package>(blob).map_err(|e| {
                            InvokeError::Error(ValidatedPackageError::InvalidPackage(e))
                        })
                    })?;
                let package = ValidatedPackage::new(package)
                    .map_err(|e| InvokeError::Error(ValidatedPackageError::InvalidWasm(e)))?;
                let node_id = system_api
                    .node_create(HeapRENode::Package(package))
                    .map_err(InvokeError::Downstream)?;
                system_api
                    .node_globalize(node_id)
                    .map_err(InvokeError::Downstream)?;
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
