use super::{PackageRoyaltyAccumulatorSubstate, PackageRoyaltyConfigSubstate};
use crate::engine::*;
use crate::engine::{CallFrameUpdate, LockFlags, RuntimeError, SystemApi};
use crate::model::{
    AccessRulesChainSubstate, GlobalAddressSubstate, MetadataSubstate, PackageInfoSubstate,
};
use crate::types::*;
use crate::wasm::*;
use core::fmt::Debug;
use radix_engine_interface::api::types::SubstateOffset;
use radix_engine_interface::api::types::{NativeFn, PackageFn, PackageId, RENodeId};
use radix_engine_interface::api::InvokableModel;
use radix_engine_interface::model::*;

pub struct Package;

#[derive(Debug, Clone, PartialEq, Eq, Categorize, Encode, Decode)]
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
        abi: BTreeMap<String, BlueprintAbi>,
    ) -> Result<PackageInfoSubstate, PrepareError> {
        WasmValidator::default().validate(&code, &abi)?;

        Ok(PackageInfoSubstate {
            code: code,
            blueprint_abis: abi,
        })
    }
}

impl ExecutableInvocation for PackagePublishInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));
        let actor = ResolvedActor::function(NativeFn::Package(PackageFn::Publish));
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for PackagePublishInvocation {
    type Output = PackageAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(PackageAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        let royalty_vault_id = api
            .invoke(ResourceManagerCreateVaultInvocation {
                receiver: RADIX_TOKEN,
            })?
            .vault_id();

        let abi = scrypto_decode::<BTreeMap<String, BlueprintAbi>>(&self.abi).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidAbi(e),
            ))
        })?;
        let package = Package::new(self.code, abi).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(e),
            ))
        })?;
        let package_royalty_config = PackageRoyaltyConfigSubstate {
            royalty_config: self.royalty_config,
        };
        let package_royalty_accumulator = PackageRoyaltyAccumulatorSubstate {
            royalty: Own::Vault(royalty_vault_id),
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
            RENodeInit::Package(
                package,
                package_royalty_config,
                package_royalty_accumulator,
                metadata_substate,
                access_rules,
            ),
        )?;
        let package_id: PackageId = node_id.into();

        // Globalize
        let global_node_id = if let Some(address) = self.package_address {
            RENodeId::Global(GlobalAddress::Package(PackageAddress::Normal(address)))
        } else {
            api.allocate_node_id(RENodeType::GlobalPackage)?
        };

        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Package(package_id)),
        )?;

        let package_address: PackageAddress = global_node_id.into();
        Ok((package_address, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for PackageSetRoyaltyConfigInvocation {
    type Exec = PackageSetRoyaltyConfigExecutable;

    fn resolve<D: ResolverApi>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Package(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, api)?;

        let actor = ResolvedActor::method(
            NativeFn::Package(PackageFn::SetRoyaltyConfig),
            resolved_receiver,
        );
        let executor = PackageSetRoyaltyConfigExecutable {
            receiver: resolved_receiver.receiver,
            royalty_config: self.royalty_config,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for PackageSetRoyaltyConfigExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: auth check
        let node_id = self.receiver;
        let offset = SubstateOffset::Package(PackageOffset::RoyaltyConfig);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate = api.get_ref_mut(handle)?;
        substate.package_royalty_config().royalty_config = self.royalty_config;

        api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for PackageClaimRoyaltyInvocation {
    type Exec = PackageClaimRoyaltyExecutable;

    fn resolve<D: ResolverApi>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Package(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, api)?;

        let actor = ResolvedActor::method(
            NativeFn::Package(PackageFn::ClaimRoyalty),
            resolved_receiver,
        );
        let executor = PackageClaimRoyaltyExecutable {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for PackageClaimRoyaltyExecutable {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
    {
        // TODO: auth check
        let node_id = self.receiver;
        let offset = SubstateOffset::Package(PackageOffset::RoyaltyAccumulator);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = api.get_ref_mut(handle)?;
        let royalty_vault = substate_mut.package_royalty_accumulator().royalty.clone();

        let amount = api.invoke(VaultGetAmountInvocation {
            receiver: royalty_vault.vault_id(),
        })?;

        let bucket = api.invoke(VaultTakeInvocation {
            receiver: royalty_vault.vault_id(),
            amount,
        })?;
        let bucket_id = bucket.0;

        api.drop_lock(handle)?;

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}
