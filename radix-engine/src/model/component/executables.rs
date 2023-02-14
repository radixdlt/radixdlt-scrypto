use crate::engine::{deref_and_update, Executor, RENodeInit};
use crate::engine::{
    CallFrameUpdate, ExecutableInvocation, LockFlags, ResolvedActor, ResolverApi, RuntimeError,
    SystemApi,
};
use crate::model::GlobalAddressSubstate;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::*;
use radix_engine_interface::{constants::*, rule};

impl ExecutableInvocation for ComponentGlobalizeInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = ResolvedActor::function(NativeFn::Component(ComponentFn::Globalize));
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Component(self.component_id));

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ComponentGlobalizeInvocation {
    type Output = ComponentAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(ComponentAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
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

        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Component(self.component_id)),
        )?;

        let call_frame_update = CallFrameUpdate::copy_ref(RENodeId::Global(
            GlobalAddress::Component(component_address),
        ));

        Ok((component_address, call_frame_update))
    }
}

impl ExecutableInvocation for ComponentGlobalizeWithOwnerInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = ResolvedActor::function(NativeFn::Component(ComponentFn::Globalize));
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Component(self.component_id));

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ComponentGlobalizeWithOwnerInvocation {
    type Output = ComponentAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(ComponentAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableModel<RuntimeError>,
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

        // Add protection for metadata/royalties
        let mut access_rules =
            AccessRules::new().default(AccessRule::AllowAll, AccessRule::AllowAll);
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Get)),
            AccessRule::AllowAll,
            rule!(require(self.owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Set)),
            rule!(require(self.owner_badge.clone())),
            rule!(require(self.owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Component(ComponentFn::SetRoyaltyConfig)),
            rule!(require(self.owner_badge.clone())),
            rule!(require(self.owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Component(ComponentFn::ClaimRoyalty)),
            rule!(require(self.owner_badge.clone())),
            rule!(require(self.owner_badge.clone())),
        );
        api.invoke(AccessRulesAddAccessCheckInvocation {
            receiver: component_node_id,
            access_rules,
        })?;

        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Component(self.component_id)),
        )?;

        let call_frame_update = CallFrameUpdate::copy_ref(RENodeId::Global(
            GlobalAddress::Component(component_address),
        ));

        Ok((component_address, call_frame_update))
    }
}

impl ExecutableInvocation for ComponentSetRoyaltyConfigInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = self.receiver;
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::Component(ComponentFn::SetRoyaltyConfig),
            resolved_receiver,
        );
        let executor = Self {
            receiver: resolved_receiver.receiver,
            royalty_config: self.royalty_config,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ComponentSetRoyaltyConfigInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: auth check
        let node_id = self.receiver;
        let offset = SubstateOffset::Component(ComponentOffset::RoyaltyConfig);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = api.get_ref_mut(handle)?;
        substate_mut.component_royalty_config().royalty_config = self.royalty_config;

        api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ComponentClaimRoyaltyInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = self.receiver;
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::Component(ComponentFn::ClaimRoyalty),
            resolved_receiver,
        );
        let executor = Self {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for ComponentClaimRoyaltyInvocation {
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
        let offset = SubstateOffset::Component(ComponentOffset::RoyaltyAccumulator);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = api.get_ref_mut(handle)?;
        let royalty_vault = substate_mut.component_royalty_accumulator().royalty.clone();

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
            bucket,
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}
