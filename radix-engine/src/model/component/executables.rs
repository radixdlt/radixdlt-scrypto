use crate::engine::{deref_and_update, RENode, ResolvedFunction};
use crate::engine::{
    CallFrameUpdate, ExecutableInvocation, LockFlags, NativeExecutor, NativeProcedure, REActor,
    ResolveApi, ResolvedMethod, RuntimeError, SystemApi,
};
use crate::model::{BucketSubstate, GlobalAddressSubstate};
use crate::wasm::WasmEngine;
use radix_engine_interface::api::api::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::{constants::*, rule};

impl<W: WasmEngine> ExecutableInvocation<W> for ComponentGlobalizeInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::Component(
            ComponentFunction::Globalize,
        )));
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Component(self.component_id));
        let executor = NativeExecutor(self);

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ComponentGlobalizeInvocation {
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

impl<W: WasmEngine> ExecutableInvocation<W> for ComponentGlobalizeWithOwnerInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>>(
        self,
        _deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = REActor::Function(ResolvedFunction::Native(NativeFunction::Component(
            ComponentFunction::Globalize,
        )));
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Component(self.component_id));
        let executor = NativeExecutor(self);

        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for ComponentGlobalizeWithOwnerInvocation {
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

        // Add protection for metadata/royalties
        let mut access_rules =
            AccessRules::new().default(AccessRule::AllowAll, AccessRule::AllowAll);
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
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Component(
                ComponentMethod::SetRoyaltyConfig,
            ))),
            rule!(require(self.owner_badge.clone())),
            rule!(require(self.owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::Component(
                ComponentMethod::ClaimRoyalty,
            ))),
            rule!(require(self.owner_badge.clone())),
            rule!(require(self.owner_badge.clone())),
        );
        api.invoke(AccessRulesAddAccessCheckInvocation {
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

impl<W: WasmEngine> ExecutableInvocation<W> for ComponentSetRoyaltyConfigInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>>(
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

impl<W: WasmEngine> ExecutableInvocation<W> for ComponentClaimRoyaltyInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolveApi<W>>(
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
