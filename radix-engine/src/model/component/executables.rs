use crate::engine::{deref_and_update, RENode, ResolvedFunction};
use crate::engine::{
    CallFrameUpdate, ExecutableInvocation, LockFlags, MethodDeref, NativeExecutor, NativeProcedure,
    REActor, ResolvedMethod, RuntimeError, SystemApi,
};
use crate::model::{BucketSubstate, GlobalAddressSubstate};
use crate::types::*;
use radix_engine_interface::api::api::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::constants::*;
use radix_engine_interface::rule;

impl ExecutableInvocation for ComponentGlobalizeWithOwnerInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::Component(
            ComponentFunction::GlobalizeWithOwner,
        )));
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Component(self.component_id));
        let executor = NativeExecutor(self);

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ComponentGlobalizeWithOwnerInvocation {
    type Output = (ComponentAddress, Bucket);

    fn main<Y>(
        self,
        api: &mut Y,
    ) -> Result<((ComponentAddress, Bucket), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + SysInvokableNative<RuntimeError>,
    {
        let component_node_id = RENodeId::Component(self.component_id);
        let global_node_id = {
            let handle = api.lock_substate(
                component_node_id,
                SubstateOffset::Component(ComponentOffset::Info),
                LockFlags::read_only(),
            )?;
            let substate_ref = api.get_ref(handle)?;
            let node_id = if substate_ref
                .component_info()
                .package_address
                .eq(&ACCOUNT_PACKAGE)
            {
                api.allocate_node_id(RENodeType::GlobalAccount)?
            } else {
                api.allocate_node_id(RENodeType::GlobalComponent)?
            };
            api.drop_lock(handle)?;
            node_id
        };
        let component_address: ComponentAddress = global_node_id.into();

        // TODO: Cleanup package address + NonFungibleId integration
        let bytes = scrypto_encode(&component_address).unwrap();
        let non_fungible_id = NonFungibleId::Bytes(bytes);
        let non_fungible_address =
            NonFungibleAddress::new(ENTITY_OWNER_TOKEN, non_fungible_id.clone());

        let mut entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)> = HashMap::new();
        entries.insert(
            non_fungible_id,
            (scrypto_encode(&()).unwrap(), scrypto_encode(&()).unwrap()),
        );

        let mint_invocation = ResourceManagerMintInvocation {
            receiver: ENTITY_OWNER_TOKEN,
            mint_params: MintParams::NonFungible { entries },
        };
        let owner_badge_bucket: Bucket = api.sys_invoke(mint_invocation)?;

        let mut access_rules = AccessRules::new()
            .default(AccessRule::AllowAll)
            .default_mutability(AccessRule::AllowAll);
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Metadata(
                MetadataMethod::Set,
            ))),
            rule!(require(non_fungible_address.clone())),
            rule!(require(non_fungible_address)),
        );

        api.sys_invoke(AccessRulesAddAccessCheckInvocation {
            receiver: component_node_id,
            access_rules,
        })?;

        api.create_node(
            global_node_id,
            RENode::Global(GlobalAddressSubstate::Component(self.component_id)),
        )?;

        let mut call_frame_update =
            CallFrameUpdate::move_node(RENodeId::Bucket(owner_badge_bucket.0));
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Component(
                component_address,
            )));

        Ok(((component_address, owner_badge_bucket), call_frame_update))
    }
}

impl ExecutableInvocation for ComponentGlobalizeNoOwnerInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::Component(
            ComponentFunction::GlobalizeNoOwner,
        )));
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Component(self.component_id));
        let executor = NativeExecutor(self);

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ComponentGlobalizeNoOwnerInvocation {
    type Output = ComponentAddress;

    fn main<Y>(self, api: &mut Y) -> Result<(ComponentAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + SysInvokableNative<RuntimeError>,
    {
        let component_node_id = RENodeId::Component(self.component_id);
        let global_node_id = {
            let handle = api.lock_substate(
                component_node_id,
                SubstateOffset::Component(ComponentOffset::Info),
                LockFlags::read_only(),
            )?;
            let substate_ref = api.get_ref(handle)?;
            let node_id = if substate_ref
                .component_info()
                .package_address
                .eq(&ACCOUNT_PACKAGE)
            {
                api.allocate_node_id(RENodeType::GlobalAccount)?
            } else {
                api.allocate_node_id(RENodeType::GlobalComponent)?
            };
            api.drop_lock(handle)?;
            node_id
        };
        let component_address: ComponentAddress = global_node_id.into();
        let mut access_rules = AccessRules::new()
            .default(AccessRule::AllowAll)
            .default_mutability(AccessRule::AllowAll);
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Metadata(
                MetadataMethod::Set,
            ))),
            AccessRule::DenyAll,
            AccessRule::DenyAll,
        );

        api.sys_invoke(AccessRulesAddAccessCheckInvocation {
            receiver: component_node_id,
            access_rules,
        })?;

        api.create_node(
            global_node_id,
            RENode::Global(GlobalAddressSubstate::Component(self.component_id)),
        )?;

        let call_frame_update = CallFrameUpdate::copy_ref(RENodeId::Global(
            GlobalAddress::Component(component_address),
        ));

        Ok((component_address, call_frame_update))
    }
}

impl ExecutableInvocation for ComponentSetRoyaltyConfigInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = self.receiver;
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Component(ComponentMethod::SetRoyaltyConfig)),
            resolved_receiver,
        );
        let executor = NativeExecutor(Self {
            receiver: resolved_receiver.receiver,
            royalty_config: self.royalty_config,
        });

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ComponentSetRoyaltyConfigInvocation {
    type Output = ();

    fn main<Y>(self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: auth check
        let node_id = self.receiver;
        let offset = SubstateOffset::Component(ComponentOffset::RoyaltyConfig);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        substate_mut.component_royalty_config().royalty_config = self.royalty_config;

        system_api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ComponentClaimRoyaltyInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = self.receiver;
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::Component(ComponentMethod::ClaimRoyalty)),
            resolved_receiver,
        );
        let executor = NativeExecutor(Self {
            receiver: resolved_receiver.receiver,
        });

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ComponentClaimRoyaltyInvocation {
    type Output = Bucket;

    fn main<Y>(self, system_api: &mut Y) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: auth check
        let node_id = self.receiver;
        let offset = SubstateOffset::Component(ComponentOffset::RoyaltyAccumulator);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        let resource = substate_mut
            .component_royalty_accumulator()
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
