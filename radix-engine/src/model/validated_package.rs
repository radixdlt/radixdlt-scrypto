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
pub enum PackageError {
    InvalidRequestData(DecodeError),
    InvalidWasm(PrepareError),
    BlueprintNotFound,
    MethodNotFound(String),
}

impl ValidatedPackage {
    pub fn new(code: Vec<u8>, abi: HashMap<String, BlueprintAbi>) -> Result<Self, PrepareError> {
        WasmValidator::default().validate(&code, &abi)?;

        Ok(Self {
            code: code,
            blueprint_abis: abi,
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
    ) -> Result<ScryptoValue, InvokeError<PackageError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match package_fn {
            PackageFnIdentifier::Publish => {
                let input: PackagePublishInput = scrypto_decode(&call_data.raw)
                    .map_err(|e| InvokeError::Error(PackageError::InvalidRequestData(e)))?;
                let package = ValidatedPackage::new(input.code, input.abi)
                    .map_err(|e| InvokeError::Error(PackageError::InvalidWasm(e)))?;
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
