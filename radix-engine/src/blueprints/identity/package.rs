use crate::errors::InterpreterError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::types::*;
use native_sdk::access_rules::AccessRulesObject;
use native_sdk::metadata::Metadata;
use radix_engine_interface::api::node_modules::metadata::{METADATA_GET_IDENT, METADATA_SET_IDENT};
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::{ClientApi, ClientSubstateApi};
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::BlueprintSchema;
use radix_engine_interface::schema::FunctionSchema;
use radix_engine_interface::schema::PackageSchema;

pub struct IdentityNativePackage;

impl IdentityNativePackage {
    pub fn schema() -> PackageSchema {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut substates = Vec::new();
        substates.push(aggregator.add_child_type_and_descendents::<AccessControllerSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateGlobalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateGlobalOutput>(),
                export_name: ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        PackageSchema {
            blueprints: btreemap!(
                IDENTITY_BLUEPRINT.to_string() => BlueprintSchema {
                    schema,
                    substates,
                    functions
                }
            ),
        }
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            IDENTITY_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

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

    fn create<Y>(
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: IdentityCreateInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let (node_id, access_rules) = Identity::create(input.access_rule, api)?;
        let access_rules = AccessRulesObject::sys_new(access_rules, api)?;
        let metadata = Metadata::sys_create(api)?;
        let address = api.globalize(
            node_id,
            btreemap!(
                NodeModuleId::AccessRules => access_rules.id(),
                NodeModuleId::Metadata => metadata.id(),
            ),
        )?;
        Ok(IndexedScryptoValue::from_typed(&address))
    }
}

pub struct Identity;

impl Identity {
    pub fn create<Y>(
        access_rule: AccessRule,
        api: &mut Y,
    ) -> Result<(RENodeId, AccessRules), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let mut access_rules = AccessRules::new();
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(NodeModuleId::Metadata, METADATA_SET_IDENT.to_string()),
            access_rule.clone(),
            access_rule,
        );
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(NodeModuleId::Metadata, METADATA_GET_IDENT.to_string()),
            AccessRule::AllowAll,
            AccessRule::DenyAll,
        );

        let component_id = api.new_object(IDENTITY_BLUEPRINT, vec![])?;

        Ok((RENodeId::Object(component_id), access_rules))
    }

    pub fn create_virtual<Y>(
        access_rule: AccessRule,
        api: &mut Y,
    ) -> Result<(RENodeId, AccessRules), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let node_id = api.kernel_allocate_node_id(RENodeType::Object)?;

        let mut access_rules = AccessRules::new();
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(NodeModuleId::Metadata, METADATA_SET_IDENT.to_string()),
            access_rule.clone(),
            access_rule,
        );
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(NodeModuleId::Metadata, METADATA_GET_IDENT.to_string()),
            AccessRule::AllowAll,
            AccessRule::DenyAll,
        );

        api.kernel_create_node(
            node_id,
            RENodeInit::Object(btreemap!()),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                    package_address: IDENTITY_PACKAGE,
                    blueprint_name: IDENTITY_BLUEPRINT.to_string(),
                    global: false,
                })
            ),
        )?;

        Ok((node_id, access_rules))
    }
}
