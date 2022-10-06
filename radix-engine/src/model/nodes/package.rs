use core::fmt::Debug;

use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::PackageSubstate;
use crate::types::*;
use crate::wasm::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Package {
    pub info: PackageSubstate,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
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
            info: PackageSubstate {
                code: code,
                blueprint_abis: abi,
            },
        })
    }

    pub fn code(&self) -> &[u8] {
        &self.info.code
    }

    pub fn blueprint_abi(&self, blueprint_name: &str) -> Option<&BlueprintAbi> {
        self.info.blueprint_abis.get(blueprint_name)
    }

    pub fn static_main<'s, Y, W, I, R>(
        func: PackageFunction,
        call_data: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<PackageError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match func {
            PackageFunction::Publish => {
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
            .field("code_len", &self.info.code.len())
            .field("blueprint_abis", &self.info.blueprint_abis)
            .finish()
    }
}
