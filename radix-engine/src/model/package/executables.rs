use super::{PackageRoyaltyAccumulatorSubstate, PackageRoyaltyConfigSubstate};
use crate::engine::*;
use crate::engine::{CallFrameUpdate, LockFlags, RuntimeError, SystemApi};
use crate::model::{GlobalAddressSubstate, MetadataSubstate, PackageInfoSubstate, Resource};
use crate::types::*;
use crate::wasm::*;
use core::fmt::Debug;
use radix_engine_interface::api::api::SysInvokableNative;
use radix_engine_interface::api::types::SubstateOffset;
use radix_engine_interface::api::types::{NativeFunction, PackageFunction, PackageId, RENodeId};
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::model::*;

pub struct Package;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum PackageError {
    InvalidRequestData(DecodeError),
    InvalidAbi(DecodeError),
    InvalidWasm(PrepareError),
    BlueprintNotFound,
    MethodNotFound(String),
    CouldNotEncodePackageAddress,
}

impl Package {
    fn new(
        code: Vec<u8>,
        abi: HashMap<String, BlueprintAbi>,
    ) -> Result<PackageInfoSubstate, PrepareError> {
        WasmValidator::default().validate(&code, &abi)?;

        Ok(PackageInfoSubstate {
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

impl NativeProcedure for PackagePublishNoOwnerInvocation {
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
        let package_royalty_config = PackageRoyaltyConfigSubstate {
            royalty_config: HashMap::new(), // TODO: add user interface
        };
        let package_royalty_accumulator = PackageRoyaltyAccumulatorSubstate {
            royalty: Resource::new_empty(RADIX_TOKEN, ResourceType::Fungible { divisibility: 18 }),
        };
        let metadata_substate = MetadataSubstate {
            metadata: self.metadata,
        };

        let node_id = api.allocate_node_id(RENodeType::Package)?;
        let node_id = api.create_node(
            node_id,
            RENode::Package(
                package,
                package_royalty_config,
                package_royalty_accumulator,
                metadata_substate,
            ),
        )?;
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

impl NativeProcedure for PackagePublishWithOwnerInvocation {
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
        let package_royalty_config = PackageRoyaltyConfigSubstate {
            royalty_config: HashMap::new(), // TODO: add user interface
        };
        let package_royalty_accumulator = PackageRoyaltyAccumulatorSubstate {
            royalty: Resource::new_empty(RADIX_TOKEN, ResourceType::Fungible { divisibility: 18 }),
        };
        let metadata_substate = MetadataSubstate {
            metadata: self.metadata,
        };

        let node_id = api.allocate_node_id(RENodeType::Package)?;
        let node_id = api.create_node(
            node_id,
            RENode::Package(
                package,
                package_royalty_config,
                package_royalty_accumulator,
                metadata_substate,
            ),
        )?;
        let package_id: PackageId = node_id.into();

        let node_id = api.allocate_node_id(RENodeType::GlobalPackage)?;
        let global_node_id = api.create_node(
            node_id,
            RENode::Global(GlobalAddressSubstate::Package(package_id)),
        )?;

        let package_address: PackageAddress = global_node_id.into();
        let bytes = scrypto_encode(&package_address).map_err(|_| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::CouldNotEncodePackageAddress,
            ))
        })?;

        let non_fungible_id = NonFungibleId::from_bytes(bytes);

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

impl ExecutableInvocation for PackageSetRoyaltyConfigInvocation {
    type Exec = NativeExecutor<PackageSetRoyaltyConfigExecutable>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let input = IndexedScryptoValue::from_typed(&self);
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Package(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::EpochManager(
                EpochManagerMethod::GetCurrentEpoch,
            )),
            resolved_receiver,
        );
        let executor = NativeExecutor(
            PackageSetRoyaltyConfigExecutable {
                receiver: resolved_receiver.receiver,
                royalty_config: self.royalty_config,
            },
            input,
        );

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for PackageSetRoyaltyConfigExecutable {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: auth check
        let node_id = self.receiver;
        let offset = SubstateOffset::Package(PackageOffset::RoyaltyConfig);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate = system_api.get_ref_mut(handle)?;
        substate.package_royalty_config().royalty_config = self.royalty_config;

        system_api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}
