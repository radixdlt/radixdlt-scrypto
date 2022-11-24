use core::fmt::Debug;
use radix_engine_interface::api::api::SysInvokableNative;
use radix_engine_interface::api::types::{NativeFunction, PackageFunction, PackageId};
use radix_engine_interface::data::IndexedScryptoValue;

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

impl ExecutableInvocation for PackagePublishNoOwnerInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let call_frame_update = CallFrameUpdate::empty();
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::Package(
            PackageFunction::PublishNoOwner,
        )));
        let executor = NativeExecutor(self, input);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProgram for PackagePublishNoOwnerInvocation {
    type Output = PackageAddress;

    fn main<Y>(self, api: &mut Y) -> Result<(PackageAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation>,
    {
        let code = api.read_blob(&self.code.0)?.to_vec();
        let blob = api.read_blob(&self.abi.0)?;
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
            metadata: self.metadata,
        };

        let node_id = api.allocate_node_id(RENodeType::Package)?;
        let node_id = api.create_node(node_id, RENode::Package(package, metadata_substate))?;
        let package_id: PackageId = node_id.into();

        let node_id = api.allocate_node_id(RENodeType::GlobalPackage)?;
        let global_node_id = api.create_node(
            node_id,
            RENode::Global(GlobalAddressSubstate::Package(package_id)),
        )?;

        let package_address: PackageAddress = global_node_id.into();

        Ok((package_address, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for PackagePublishWithOwnerInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let call_frame_update = CallFrameUpdate::empty();
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::Package(
            PackageFunction::PublishWithOwner,
        )));
        let executor = NativeExecutor(self, input);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProgram for PackagePublishWithOwnerInvocation {
    type Output = (PackageAddress, Bucket);

    fn main<Y>(
        self,
        api: &mut Y,
    ) -> Result<((PackageAddress, Bucket), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + SysInvokableNative<RuntimeError>,
    {
        let code = api.read_blob(&self.code.0)?.to_vec();
        let blob = api.read_blob(&self.abi.0)?;
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
            metadata: self.metadata,
        };

        let node_id = api.allocate_node_id(RENodeType::Package)?;
        let node_id = api.create_node(node_id, RENode::Package(package, metadata_substate))?;
        let package_id: PackageId = node_id.into();

        let node_id = api.allocate_node_id(RENodeType::GlobalPackage)?;
        let global_node_id = api.create_node(
            node_id,
            RENode::Global(GlobalAddressSubstate::Package(package_id)),
        )?;

        let package_address: PackageAddress = global_node_id.into();

        let non_fungible_id = NonFungibleId::from_bytes(scrypto_encode(&package_address));

        let mut entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)> = HashMap::new();
        entries.insert(non_fungible_id, (vec![], vec![]));

        let mint_invocation = ResourceManagerMintInvocation {
            receiver: ENTITY_OWNER_TOKEN,
            mint_params: MintParams::NonFungible { entries },
        };

        let bucket = api.sys_invoke(mint_invocation)?;
        let bucket_node_id = RENodeId::Bucket(bucket.0);

        Ok((
            (package_address, bucket),
            CallFrameUpdate::move_node(bucket_node_id),
        ))
    }
}
