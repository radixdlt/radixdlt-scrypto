use super::{PackageRoyaltyAccumulatorSubstate, PackageRoyaltyConfigSubstate};
use crate::engine::*;
use crate::engine::{CallFrameUpdate, LockFlags, RuntimeError, SystemApi};
use crate::model::{
    AccessRulesChainSubstate, BucketSubstate, GlobalAddressSubstate, MetadataSubstate,
    PackageInfoSubstate, Resource,
};
use crate::types::*;
use crate::wasm::*;
use core::fmt::Debug;
use radix_engine_interface::api::api::SysInvokableNative;
use radix_engine_interface::api::types::SubstateOffset;
use radix_engine_interface::api::types::{NativeFunction, PackageFunction, PackageId, RENodeId};
use radix_engine_interface::model::*;
use radix_engine_interface::rule;

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

impl<W: WasmEngine> ExecutableInvocation<W> for PackagePublishInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let call_frame_update = CallFrameUpdate::empty();
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::Package(
            PackageFunction::Publish,
        )));
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for PackagePublishInvocation {
    type Output = PackageAddress;

    fn main<Y>(self, api: &mut Y) -> Result<(PackageAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
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
            royalty_config: self.royalty_config,
        };
        let package_royalty_accumulator = PackageRoyaltyAccumulatorSubstate {
            royalty: Resource::new_empty(RADIX_TOKEN, ResourceType::Fungible { divisibility: 18 }),
        };
        let metadata_substate = MetadataSubstate {
            metadata: self.metadata,
        };
        let access_rules = AccessRulesChainSubstate {
            access_rules_chain: vec![self.access_rules],
        };

        // TODO: Can we trust developers enough to add protection for
        // - `metadata::set`
        // - `access_rules_chain::add_access_rules`
        // - `royalty::set_royalty_config`
        // - `royalty::claim_royalty`

        // Create package node
        let node_id = api.allocate_node_id(RENodeType::Package)?;
        api.create_node(
            node_id,
            RENode::Package(
                package,
                package_royalty_config,
                package_royalty_accumulator,
                metadata_substate,
                access_rules,
            ),
        )?;
        let package_id: PackageId = node_id.into();

        // Globalize
        let global_node_id = api.allocate_node_id(RENodeType::GlobalPackage)?;
        api.create_node(
            global_node_id,
            RENode::Global(GlobalAddressSubstate::Package(package_id)),
        )?;

        let package_address: PackageAddress = global_node_id.into();
        Ok((package_address, CallFrameUpdate::empty()))
    }
}

impl<W: WasmEngine> ExecutableInvocation<W> for PackagePublishWithOwnerInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let call_frame_update = CallFrameUpdate::empty();
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::Package(
            PackageFunction::PublishWithOwner,
        )));
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for PackagePublishWithOwnerInvocation {
    type Output = PackageAddress;

    fn main<Y>(self, api: &mut Y) -> Result<(PackageAddress, CallFrameUpdate), RuntimeError>
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
            royalty_config: self.royalty_config,
        };
        let package_royalty_accumulator = PackageRoyaltyAccumulatorSubstate {
            royalty: Resource::new_empty(RADIX_TOKEN, ResourceType::Fungible { divisibility: 18 }),
        };
        let metadata_substate = MetadataSubstate {
            metadata: self.metadata,
        };

        let global_node_id = api.allocate_node_id(RENodeType::GlobalPackage)?;
        let package_address: PackageAddress = global_node_id.into();

        // Add protection for metadata/royalties
        let mut access_rules = AccessRules::new().default(AccessRule::DenyAll, AccessRule::DenyAll);
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Metadata(
                MetadataMethod::Get,
            ))),
            AccessRule::AllowAll,
            rule!(require(self.owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Metadata(
                MetadataMethod::Set,
            ))),
            rule!(require(self.owner_badge.clone())),
            rule!(require(self.owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Package(
                PackageMethod::SetRoyaltyConfig,
            ))),
            rule!(require(self.owner_badge.clone())),
            rule!(require(self.owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Package(
                PackageMethod::ClaimRoyalty,
            ))),
            rule!(require(self.owner_badge.clone())),
            rule!(require(self.owner_badge.clone())),
        );

        // Create package node
        let node_id = api.allocate_node_id(RENodeType::Package)?;
        api.create_node(
            node_id,
            RENode::Package(
                package,
                package_royalty_config,
                package_royalty_accumulator,
                metadata_substate,
                AccessRulesChainSubstate {
                    access_rules_chain: vec![access_rules],
                },
            ),
        )?;
        let package_id: PackageId = node_id.into();

        // Globalize
        api.create_node(
            global_node_id,
            RENode::Global(GlobalAddressSubstate::Package(package_id)),
        )?;

        Ok((
            package_address,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Package(package_address))),
        ))
    }
}

impl<W: WasmEngine> ExecutableInvocation<W> for PackageSetRoyaltyConfigInvocation {
    type Exec = NativeExecutor<PackageSetRoyaltyConfigExecutable>;

    fn resolve<D: ResolveApi<W>>(
        self,
        api: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Package(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, api)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Package(PackageMethod::SetRoyaltyConfig)),
            resolved_receiver,
        );
        let executor = NativeExecutor(PackageSetRoyaltyConfigExecutable {
            receiver: resolved_receiver.receiver,
            royalty_config: self.royalty_config,
        });

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

impl<W: WasmEngine> ExecutableInvocation<W> for PackageClaimRoyaltyInvocation {
    type Exec = NativeExecutor<PackageClaimRoyaltyExecutable>;

    fn resolve<D: ResolveApi<W>>(
        self,
        api: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Package(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, api)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Package(PackageMethod::ClaimRoyalty)),
            resolved_receiver,
        );
        let executor = NativeExecutor(PackageClaimRoyaltyExecutable {
            receiver: resolved_receiver.receiver,
        });

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for PackageClaimRoyaltyExecutable {
    type Output = Bucket;

    fn main<Y>(self, system_api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: auth check
        let node_id = self.receiver;
        let offset = SubstateOffset::Package(PackageOffset::RoyaltyAccumulator);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        let resource = substate_mut
            .package_royalty_accumulator()
            .royalty
            .take_all();
        let bucket_node_id = system_api.allocate_node_id(RENodeType::Bucket)?;
        system_api.create_node(
            bucket_node_id,
            RENode::Bucket(BucketSubstate::new(resource)),
        )?;
        let bucket_id = bucket_node_id.into();

        system_api.drop_lock(handle)?;

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}
