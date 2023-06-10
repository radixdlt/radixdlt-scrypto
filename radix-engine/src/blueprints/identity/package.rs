use crate::blueprints::util::{
    PresecurifiedAccessRules, SecurifiedAccessRules, SecurifiedRoleEntry,
};
use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::method_auth_template;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::kernel_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::node_modules::royalty::{
    COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::{
    BlueprintSetup, BlueprintTemplate, PackageSetup,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::ReceiverInfo;
use radix_engine_interface::schema::{BlueprintSchema, SchemaMethodKey, SchemaMethodPermission};
use radix_engine_interface::schema::{FunctionSchema, VirtualLazyLoadSchema};
use resources_tracker_macro::trace_resources;

const IDENTITY_CREATE_VIRTUAL_SECP256K1_EXPORT_NAME: &str = "create_virtual_secp256k1";
const IDENTITY_CREATE_VIRTUAL_ED25519_EXPORT_NAME: &str = "create_virtual_ed25519";

pub struct IdentityNativePackage;

impl IdentityNativePackage {
    pub fn definition() -> PackageSetup {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let fields = Vec::new();

        let mut functions = BTreeMap::new();
        functions.insert(
            IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<IdentityCreateAdvancedInput>(),
                output: aggregator.add_child_type_and_descendents::<IdentityCreateAdvancedOutput>(),
                export: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            },
        );
        functions.insert(
            IDENTITY_CREATE_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<IdentityCreateInput>(),
                output: aggregator.add_child_type_and_descendents::<IdentityCreateOutput>(),
                export: IDENTITY_CREATE_IDENT.to_string(),
            },
        );
        functions.insert(
            IDENTITY_SECURIFY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<IdentitySecurifyToSingleBadgeInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<IdentitySecurifyToSingleBadgeOutput>(),
                export: IDENTITY_SECURIFY_IDENT.to_string(),
            },
        );

        let virtual_lazy_load_functions = btreemap!(
            IDENTITY_CREATE_VIRTUAL_SECP256K1_ID => VirtualLazyLoadSchema {
                export_name: IDENTITY_CREATE_VIRTUAL_SECP256K1_EXPORT_NAME.to_string(),
            },
            IDENTITY_CREATE_VIRTUAL_ED25519_ID => VirtualLazyLoadSchema {
                export_name: IDENTITY_CREATE_VIRTUAL_ED25519_EXPORT_NAME.to_string(),
            }
        );

        let method_auth_template = method_auth_template! {
            SchemaMethodKey::metadata(METADATA_GET_IDENT) => SchemaMethodPermission::Public;
            SchemaMethodKey::metadata(METADATA_SET_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::metadata(METADATA_REMOVE_IDENT) => [OWNER_ROLE];

            SchemaMethodKey::royalty(COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::royalty(COMPONENT_ROYALTY_SET_ROYALTY_IDENT) => [OWNER_ROLE];

            SchemaMethodKey::main(IDENTITY_SECURIFY_IDENT) => [SECURIFY_ROLE];
        };

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            IDENTITY_BLUEPRINT.to_string() => BlueprintSetup {
                schema: BlueprintSchema {
                    outer_blueprint: None,
                    schema,
                    fields,
                    collections: vec![],
                    functions,
                    virtual_lazy_load_functions,
                    event_schema: [].into(),
                    dependencies: btreeset!(
                        SECP256K1_SIGNATURE_VIRTUAL_BADGE.into(),
                        ED25519_SIGNATURE_VIRTUAL_BADGE.into(),
                        IDENTITY_OWNER_BADGE.into(),
                        PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                    ),
                    features: btreeset!(),
                },
                function_auth: btreemap!(
                    IDENTITY_CREATE_IDENT.to_string() => rule!(allow_all),
                    IDENTITY_CREATE_ADVANCED_IDENT.to_string() => rule!(allow_all),
                ),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template,
                    outer_method_auth_template: btreemap!(),
                }
            }
        );

        PackageSetup { blueprints }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<&NodeId>,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match export_name {
            IDENTITY_CREATE_ADVANCED_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: IdentityCreateAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create_advanced(input.owner_rule, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let _input: IdentityCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_SECURIFY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: IdentitySecurifyToSingleBadgeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::securify(receiver, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_CREATE_VIRTUAL_SECP256K1_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create_virtual_secp256k1(input, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_CREATE_VIRTUAL_ED25519_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create_virtual_ed25519(input, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}

const SECURIFY_ROLE: &'static str = "securify";

struct SecurifiedIdentity;

impl SecurifiedAccessRules for SecurifiedIdentity {
    const OWNER_BADGE: ResourceAddress = IDENTITY_OWNER_BADGE;
    const SECURIFY_ROLE: Option<&'static str> = Some(SECURIFY_ROLE);

    fn role_definitions() -> BTreeMap<RoleKey, SecurifiedRoleEntry> {
        btreemap!()
    }
}

impl PresecurifiedAccessRules for SecurifiedIdentity {}

pub struct IdentityBlueprint;

impl IdentityBlueprint {
    pub fn create_advanced<Y>(
        owner_rule: OwnerRole,
        api: &mut Y,
    ) -> Result<GlobalAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let access_rules = SecurifiedIdentity::create_advanced(owner_rule, api)?;

        let modules = Self::create_object(access_rules, api)?;
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();
        let address = api.globalize(modules)?;
        Ok(address)
    }

    pub fn create<Y>(api: &mut Y) -> Result<(GlobalAddress, Bucket), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (access_rules, bucket) = SecurifiedIdentity::create_securified(api)?;

        let modules = Self::create_object(access_rules, api)?;
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();
        let address = api.globalize(modules)?;
        Ok((address, bucket))
    }

    pub fn create_virtual_secp256k1<Y>(
        input: VirtualLazyLoadInput,
        api: &mut Y,
    ) -> Result<BTreeMap<ObjectModuleId, Own>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let public_key_hash = PublicKeyHash::Secp256k1(Secp256k1PublicKeyHash(input.id));
        Self::create_virtual(public_key_hash, api)
    }

    pub fn create_virtual_ed25519<Y>(
        input: VirtualLazyLoadInput,
        api: &mut Y,
    ) -> Result<BTreeMap<ObjectModuleId, Own>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let public_key_hash = PublicKeyHash::Ed25519(Ed25519PublicKeyHash(input.id));
        Self::create_virtual(public_key_hash, api)
    }

    fn create_virtual<Y>(
        public_key_hash: PublicKeyHash,
        api: &mut Y,
    ) -> Result<BTreeMap<ObjectModuleId, Own>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let owner_id = NonFungibleGlobalId::from_public_key_hash(public_key_hash);
        let access_rules = SecurifiedIdentity::create_presecurified(owner_id, api)?;

        let modules = Self::create_object(access_rules, api)?;

        {
            // Set up metadata
            // TODO: Improve this when the Metadata module API is nicer
            let metadata = modules.get(&ObjectModuleId::Metadata).unwrap();
            // NOTE:
            // This is the owner key for ROLA.
            // We choose to set this explicitly to simplify the security-critical logic off-ledger.
            // In particular, we want an owner to be able to explicitly delete the owner keys.
            // If we went with a "no metadata = assume default public key hash", then this could cause unexpeted
            // security-critical behaviour if a user expected that deleting the metadata removed the owner keys.
            api.call_method(
                &metadata.0,
                METADATA_SET_IDENT,
                scrypto_encode(&MetadataSetInput {
                    key: "owner_keys".to_string(),
                    value: MetadataValue::PublicKeyHashArray(vec![public_key_hash]),
                })
                .unwrap(),
            )?;
        }

        Ok(modules)
    }

    fn securify<Y>(receiver: &NodeId, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        SecurifiedIdentity::securify(receiver, api)
    }

    fn create_object<Y>(
        access_rules: AccessRules,
        api: &mut Y,
    ) -> Result<BTreeMap<ObjectModuleId, Own>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let metadata = Metadata::create(api)?;
        let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

        let object_id = api.new_simple_object(IDENTITY_BLUEPRINT, vec![])?;

        let modules = btreemap!(
            ObjectModuleId::Main => Own(object_id),
            ObjectModuleId::AccessRules => access_rules.0,
            ObjectModuleId::Metadata => metadata,
            ObjectModuleId::Royalty => royalty,
        );

        Ok(modules)
    }
}
