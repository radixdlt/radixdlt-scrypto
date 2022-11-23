use core::fmt::Debug;
use radix_engine_interface::api::api::SysInvokableNative;
use radix_engine_interface::api::types::{NativeFunction, PackageFunction, PackageId};

use crate::engine::*;
use crate::model::{GlobalAddressSubstate, MetadataSubstate, PackageSubstate};
use crate::types::*;
use crate::wasm::*;

pub struct Package;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum PackageError {
    InvalidRequestData(DecodeError),
    InvalidAbi(DecodeError),
    InvalidWasm(PrepareError),
    BlueprintNotFound,
    MethodNotFound(String),
}

impl Package {
    fn new(
        code: Vec<u8>,
        abi: HashMap<String, BlueprintAbi>,
    ) -> Result<PackageSubstate, PrepareError> {
        WasmValidator::default().validate(&code, &abi)?;

        Ok(PackageSubstate {
            code: code,
            blueprint_abis: abi,
        })
    }
}

impl NativeInvocation for PackagePublishNoOwnerInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Function(
            NativeFunction::Package(PackageFunction::PublishNoOwner),
            CallFrameUpdate::empty(),
        )
    }

    fn execute<Y>(
        invocation: Self,
        system_api: &mut Y,
    ) -> Result<(PackageAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation> + SysInvokableNative<RuntimeError>,
    {
        let code = system_api.read_blob(&invocation.code.0)?.to_vec();
        let blob = system_api.read_blob(&invocation.abi.0)?;
        let abi = scrypto_decode::<HashMap<String, BlueprintAbi>>(blob).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidAbi(e),
            ))
        })?;
        let package = Package::new(code, abi).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(e),
            ))
        })?;

        let metadata_substate = MetadataSubstate {
            metadata: invocation.metadata,
        };

        let node_id = system_api.create_node(RENode::Package(package, metadata_substate))?;
        let package_id: PackageId = node_id.into();

        let global_node_id =
            system_api.create_node(RENode::Global(GlobalAddressSubstate::Package(package_id)))?;

        let package_address: PackageAddress = global_node_id.into();

        Ok((package_address, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for PackagePublishWithOwnerInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Function(
            NativeFunction::Package(PackageFunction::PublishWithOwner),
            CallFrameUpdate::empty(),
        )
    }

    fn execute<Y>(
        invocation: Self,
        system_api: &mut Y,
    ) -> Result<((PackageAddress, Bucket), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation> + SysInvokableNative<RuntimeError>,
    {
        let code = system_api.read_blob(&invocation.code.0)?.to_vec();
        let blob = system_api.read_blob(&invocation.abi.0)?;
        let abi = scrypto_decode::<HashMap<String, BlueprintAbi>>(blob).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidAbi(e),
            ))
        })?;
        let package = Package::new(code, abi).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(e),
            ))
        })?;

        let metadata_substate = MetadataSubstate {
            metadata: invocation.metadata,
        };

        let node_id = system_api.create_node(RENode::Package(package, metadata_substate))?;
        let package_id: PackageId = node_id.into();

        let global_node_id =
            system_api.create_node(RENode::Global(GlobalAddressSubstate::Package(package_id)))?;

        let package_address: PackageAddress = global_node_id.into();

        let non_fungible_id = NonFungibleId::from_bytes(scrypto_encode(&package_address));

        let mut entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)> = HashMap::new();
        entries.insert(non_fungible_id, (vec![], vec![]));

        let mint_invocation = ResourceManagerMintInvocation {
            receiver: ENTITY_OWNER_TOKEN,
            mint_params: MintParams::NonFungible { entries },
        };

        let bucket = system_api.sys_invoke(mint_invocation)?;
        let bucket_node_id = RENodeId::Bucket(bucket.0);

        Ok((
            (package_address, bucket),
            CallFrameUpdate::move_node(bucket_node_id),
        ))
    }
}
