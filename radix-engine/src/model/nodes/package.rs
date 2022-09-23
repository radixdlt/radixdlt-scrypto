use core::fmt::Debug;

use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::PackageSubstate;
use crate::types::*;
use crate::wasm::*;

use super::TryIntoSubstates;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Package {
    code: Vec<u8>,
    blueprint_abis: HashMap<String, BlueprintAbi>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum PackageError {
    InvalidRequestData(DecodeError),
    InvalidAbi(DecodeError),
    InvalidWasm(PrepareError),
    BlueprintNotFound,
    MethodNotFound(String),
}

impl Package {
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
                let code = system_api
                    .read_blob(&input.code.0)
                    .map_err(InvokeError::Downstream)?
                    .to_vec();
                let abi = system_api
                    .read_blob(&input.abi.0)
                    .map_err(InvokeError::Downstream)
                    .and_then(|blob| {
                        scrypto_decode::<HashMap<String, BlueprintAbi>>(blob)
                            .map_err(|e| InvokeError::Error(PackageError::InvalidAbi(e)))
                    })?;
                let package = Package::new(code, abi)
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

impl Debug for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Package")
            .field("code_len", &self.code.len())
            .field("blueprint_abis", &self.blueprint_abis)
            .finish()
    }
}

impl TryIntoSubstates for Package {
    type Error = ();

    fn try_into_substates(self) -> Result<Vec<crate::model::Substate>, Self::Error> {
        Ok(vec![PackageSubstate {
            code: self.code,
            blueprint_abis: self.blueprint_abis,
        }
        .into()])
    }
}
