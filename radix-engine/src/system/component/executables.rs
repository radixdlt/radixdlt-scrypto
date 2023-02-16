use crate::errors::RuntimeError;
use crate::kernel::actor::ResolvedActor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{
    ExecutableInvocation, Executor, KernelNodeApi, KernelSubstateApi,
};
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::RENodeInit;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::AccessRules;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;
use sbor::rust::collections::BTreeMap;

impl ExecutableInvocation for ComponentGlobalizeWithOwnerInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = ResolvedActor::function(NativeFn::Component(ComponentFn::GlobalizeWithOwner));
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
        Y: KernelNodeApi + KernelSubstateApi + ClientNativeInvokeApi<RuntimeError>,
    {
        let component_node_id = RENodeId::Component(self.component_id);
        let global_node_id = api.kernel_allocate_node_id(RENodeType::GlobalComponent)?;
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
        api.call_native(AccessRulesAddAccessCheckInvocation {
            receiver: component_node_id,
            access_rules,
        })?;

        api.kernel_create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Component(self.component_id)),
            BTreeMap::new(),
        )?;

        let call_frame_update = CallFrameUpdate::copy_ref(RENodeId::Global(
            GlobalAddress::Component(component_address),
        ));

        Ok((component_address, call_frame_update))
    }
}
