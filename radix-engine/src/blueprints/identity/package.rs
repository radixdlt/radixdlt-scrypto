use crate::errors::InterpreterError;
use crate::errors::RuntimeError;
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRulesObject;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::node_modules::metadata::METADATA_SET_IDENT;
use radix_engine_interface::api::types::ClientCostingReason;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::node_modules::auth::{ACCESS_RULES_SET_GROUP_ACCESS_RULE_AND_MUTABILITY_IDENT, AccessRulesSetGroupAccessRuleAndMutabilityInput};
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::BlueprintSchema;
use radix_engine_interface::schema::FunctionSchema;
use radix_engine_interface::schema::PackageSchema;

pub const OWNER_GROUP_NAME: &str = "owner";

pub struct IdentityNativePackage;

impl IdentityNativePackage {
    pub fn schema() -> PackageSchema {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let substates = Vec::new();

        let mut functions = BTreeMap::new();
        functions.insert(
            IDENTITY_CREATE_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<IdentityCreateInput>(),
                output: aggregator.add_child_type_and_descendents::<IdentityCreateOutput>(),
                export_name: IDENTITY_CREATE_IDENT.to_string(),
            },
        );

        // TODO: Make these not visible to client (should only be called by virtualization)
        functions.insert(
            IDENTITY_CREATE_VIRTUAL_ECDSA_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<VirtualLazyLoadInput>(),
                output: aggregator.add_child_type_and_descendents::<VirtualLazyLoadOutput>(),
                export_name: IDENTITY_CREATE_VIRTUAL_ECDSA_IDENT.to_string(),
            },
        );
        functions.insert(
            IDENTITY_CREATE_VIRTUAL_EDDSA_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<VirtualLazyLoadInput>(),
                output: aggregator.add_child_type_and_descendents::<VirtualLazyLoadOutput>(),
                export_name: IDENTITY_CREATE_VIRTUAL_EDDSA_IDENT.to_string(),
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
        Y: ClientApi<RuntimeError>,
    {
        match export_name {
            IDENTITY_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: IdentityCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create(input.access_rule, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_CREATE_VIRTUAL_ECDSA_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create_ecdsa_virtual(input.id, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_CREATE_VIRTUAL_EDDSA_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create_eddsa_virtual(input.id, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}

pub struct IdentityBlueprint;

impl IdentityBlueprint {
    pub fn create<Y>(access_rule: AccessRule, api: &mut Y) -> Result<Address, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (object, modules) = Self::create_object(access_rule.clone(), access_rule, api)?;
        let modules = modules
            .into_iter()
            .map(|(id, own)| (id, own.id()))
            .collect();
        let address = api.globalize(RENodeId::Object(object.id()), modules)?;
        Ok(address)
    }

    pub fn create_ecdsa_virtual<Y>(
        id: [u8; 26],
        api: &mut Y,
    ) -> Result<(Own, BTreeMap<NodeModuleId, Own>), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let non_fungible_global_id = NonFungibleGlobalId::new(
            ECDSA_SECP256K1_TOKEN,
            NonFungibleLocalId::bytes(id.to_vec()).unwrap(),
        );
        let access_rule = rule!(require(non_fungible_global_id));

        let non_fungible_global_id = NonFungibleGlobalId::new(
            PACKAGE_TOKEN,
            NonFungibleLocalId::bytes(scrypto_encode(&IDENTITY_PACKAGE).unwrap()).unwrap(),
        );
        let mutability_rule = rule!(require(non_fungible_global_id));
        Self::create_object(access_rule, mutability_rule, api)
    }

    pub fn create_eddsa_virtual<Y>(
        id: [u8; 26],
        api: &mut Y,
    ) -> Result<(Own, BTreeMap<NodeModuleId, Own>), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let non_fungible_global_id = NonFungibleGlobalId::new(
            EDDSA_ED25519_TOKEN,
            NonFungibleLocalId::bytes(id.to_vec()).unwrap(),
        );
        let access_rule = rule!(require(non_fungible_global_id));

        let non_fungible_global_id = NonFungibleGlobalId::new(
            PACKAGE_TOKEN,
            NonFungibleLocalId::bytes(scrypto_encode(&IDENTITY_PACKAGE).unwrap()).unwrap(),
        );
        let mutability_rule = rule!(require(non_fungible_global_id));

        Self::create_object(access_rule, mutability_rule, api)
    }

    fn create_object<Y>(
        access_rule: AccessRule,
        mutability: AccessRule,
        api: &mut Y,
    ) -> Result<(Own, BTreeMap<NodeModuleId, Own>), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mut access_rules = AccessRulesConfig::new();
        access_rules.set_group_access_rule_and_mutability(
            OWNER_GROUP_NAME.to_string(),
            access_rule,
            mutability,
        );
        access_rules.set_method_access_rule_to_group(
            MethodKey::new(NodeModuleId::Metadata, METADATA_SET_IDENT.to_string()),
            OWNER_GROUP_NAME.to_string(),
        );

        let access_rules = AccessRulesObject::sys_new(access_rules, api)?;
        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(api, RoyaltyConfig::default())?;

        let object_id = api.new_object(IDENTITY_BLUEPRINT, vec![])?;

        let modules = btreemap!(
            NodeModuleId::AccessRules => access_rules,
            NodeModuleId::Metadata => metadata,
            NodeModuleId::ComponentRoyalty => royalty,
        );

        Ok((Own::Object(object_id), modules))
    }

    fn replace_access_rules_with_generated_owner_badge<Y>(
        receiver: RENodeId,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
        where
            Y: ClientApi<RuntimeError>,
    {
        let owner_token = ResourceManager(IDENTITY_OWNER_TOKEN);
        let (bucket, local_id) = owner_token.mint_non_fungible_single_uuid((), api)?;
        let global_id = NonFungibleGlobalId::new(IDENTITY_OWNER_TOKEN, local_id);

        let _rtn = api.call_module_method(
            receiver,
            NodeModuleId::AccessRules,
            ACCESS_RULES_SET_GROUP_ACCESS_RULE_AND_MUTABILITY_IDENT,
            scrypto_encode(&AccessRulesSetGroupAccessRuleAndMutabilityInput {
                name: OWNER_GROUP_NAME.to_string(),
                rule: rule!(require(global_id)),
                mutability: AccessRule::DenyAll,
            }).unwrap(),
        )?;

        Ok(bucket)
    }
}
