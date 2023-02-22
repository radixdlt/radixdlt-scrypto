use crate::errors::InterpreterError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::global::GlobalSubstate;
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::access_rules::ObjectAccessRulesChainSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::types::*;
use radix_engine_interface::api::node_modules::metadata::{METADATA_GET_IDENT, METADATA_SET_IDENT};
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::{ClientApi, ClientSubstateApi};
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

pub struct IdentityNativePackage;
impl IdentityNativePackage {
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        match export_name {
            IDENTITY_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                Self::create(input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    fn create<Y>(input: ScryptoValue, api: &mut Y) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: IdentityCreateInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let node_id = Identity::create(input.access_rule, api)?;
        let global_node_id = api.kernel_allocate_node_id(RENodeType::GlobalIdentity)?;
        api.kernel_create_node(
            global_node_id,
            RENodeInit::Global(GlobalSubstate::Identity(node_id.into())),
            BTreeMap::new(),
        )?;
        let identity_address: ComponentAddress = global_node_id.into();

        Ok(IndexedScryptoValue::from_typed(&identity_address))
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
            AccessRuleKey::new(NodeModuleId::Metadata, METADATA_SET_IDENT.to_string()),
            access_rule.clone(),
            access_rule,
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::new(NodeModuleId::Metadata, METADATA_GET_IDENT.to_string()),
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
            RENodeModuleInit::ObjectAccessRulesChain(ObjectAccessRulesChainSubstate {
                access_rules_chain: vec![access_rules],
            }),
        );

        api.kernel_create_node(underlying_node_id, RENodeInit::Identity(), node_modules)?;

        Ok(underlying_node_id)
    }
}
