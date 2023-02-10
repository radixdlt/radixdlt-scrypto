use crate::errors::RuntimeError;
use crate::kernel::actor::ResolvedActor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::ExecutableInvocation;
use crate::kernel::kernel_api::Executor;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientDerefApi;
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::resource::*;

impl ExecutableInvocation for IdentityCreateInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = ResolvedActor::function(NativeFn::Identity(IdentityFn::Create));
        let call_frame_update = CallFrameUpdate::empty();

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for IdentityCreateInvocation {
    type Output = ComponentAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let node_id = Identity::create(self.access_rule, api)?;
        let global_node_id = api.kernel_allocate_node_id(RENodeType::GlobalIdentity)?;
        api.kernel_create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Identity(node_id.into())),
            BTreeMap::new(),
        )?;

        let identity_address: ComponentAddress = global_node_id.into();
        let mut node_refs_to_copy = HashSet::new();
        node_refs_to_copy.insert(global_node_id);

        let update = CallFrameUpdate {
            node_refs_to_copy,
            nodes_to_move: vec![],
        };

        Ok((identity_address, update))
    }
}

pub struct Identity;

impl Identity {
    pub fn create<Y>(access_rule: AccessRule, api: &mut Y) -> Result<RENodeId, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let underlying_node_id = api.kernel_allocate_node_id(RENodeType::Identity)?;

        let mut access_rules = AccessRules::new();
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Set)),
            access_rule.clone(),
            access_rule,
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Get)),
            AccessRule::AllowAll,
            AccessRule::DenyAll,
        );

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(MetadataSubstate {
                metadata: BTreeMap::new(),
            }),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::AccessRulesChain(AccessRulesChainSubstate {
                access_rules_chain: vec![access_rules],
            }),
        );

        api.kernel_create_node(underlying_node_id, RENodeInit::Identity(), node_modules)?;

        Ok(underlying_node_id)
    }
}
