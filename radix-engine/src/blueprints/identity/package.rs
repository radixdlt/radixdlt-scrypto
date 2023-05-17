use crate::blueprints::util::{PresecurifiedAccessRules, SecurifiedAccessRules};
use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::kernel_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::BlueprintSchema;
use radix_engine_interface::schema::{FunctionSchema, VirtualLazyLoadSchema};
use radix_engine_interface::schema::{PackageSchema, ReceiverInfo};
use resources_tracker_macro::trace_resources;

const IDENTITY_CREATE_VIRTUAL_ECDSA_SECP256K1_EXPORT_NAME: &str = "create_virtual_ecdsa_secp256k1";
const IDENTITY_CREATE_VIRTUAL_EDDSA_ED25519_EXPORT_NAME: &str = "create_virtual_eddsa_ed25519";

pub struct IdentityNativePackage;

impl IdentityNativePackage {
    pub fn schema() -> PackageSchema {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let fields = Vec::new();

        let mut functions = BTreeMap::new();
        functions.insert(
            IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<IdentityCreateAdvancedInput>(),
                output: aggregator.add_child_type_and_descendents::<IdentityCreateAdvancedOutput>(),
                export_name: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            },
        );
        functions.insert(
            IDENTITY_CREATE_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<IdentityCreateInput>(),
                output: aggregator.add_child_type_and_descendents::<IdentityCreateOutput>(),
                export_name: IDENTITY_CREATE_IDENT.to_string(),
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
                export_name: IDENTITY_SECURIFY_IDENT.to_string(),
            },
        );

        let virtual_lazy_load_functions = btreemap!(
            IDENTITY_CREATE_VIRTUAL_ECDSA_SECP256K1_ID => VirtualLazyLoadSchema {
                export_name: IDENTITY_CREATE_VIRTUAL_ECDSA_SECP256K1_EXPORT_NAME.to_string(),
            },
            IDENTITY_CREATE_VIRTUAL_EDDSA_ED25519_ID => VirtualLazyLoadSchema {
                export_name: IDENTITY_CREATE_VIRTUAL_EDDSA_ED25519_EXPORT_NAME.to_string(),
            }
        );

        let schema = generate_full_schema(aggregator);
        PackageSchema {
            blueprints: btreemap!(
                IDENTITY_BLUEPRINT.to_string() => BlueprintSchema {
                    outer_blueprint: None,
                    schema,
                    fields,
                    collections: vec![],
                    functions,
                    virtual_lazy_load_functions,
                    event_schema: [].into()
                }
            ),
        }
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

                let rtn = IdentityBlueprint::create_advanced(input.authority_rules, api)?;

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
            IDENTITY_CREATE_VIRTUAL_ECDSA_SECP256K1_EXPORT_NAME => {
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
            IDENTITY_CREATE_VIRTUAL_EDDSA_ED25519_EXPORT_NAME => {
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

struct SecurifiedIdentity;

impl SecurifiedAccessRules for SecurifiedIdentity {
    const OWNER_BADGE: ResourceAddress = IDENTITY_OWNER_BADGE;
    const SECURIFY_AUTHORITY: Option<&'static str> = Some("securify");

    fn method_authorities() -> MethodAuthorities {
        let mut method_authorities = MethodAuthorities::new();
        method_authorities.set_main_method_authority(IDENTITY_SECURIFY_IDENT, "securify");
        method_authorities
    }

    fn authority_rules() -> AuthorityRules {
        let mut authority_rules = AuthorityRules::new();
        authority_rules.set_metadata_authority(rule!(require_owner()), rule!(deny_all));
        authority_rules.set_royalty_authority(rule!(require_owner()), rule!(deny_all));
        authority_rules
    }
}

impl PresecurifiedAccessRules for SecurifiedIdentity {
    const PACKAGE: PackageAddress = IDENTITY_PACKAGE;
}

pub struct IdentityBlueprint;

impl IdentityBlueprint {
    pub fn create_advanced<Y>(
        authority_rules: AuthorityRules,
        api: &mut Y,
    ) -> Result<GlobalAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let access_rules = SecurifiedIdentity::create_advanced(authority_rules, api)?;

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
        let public_key_hash = PublicKeyHash::EcdsaSecp256k1(EcdsaSecp256k1PublicKeyHash(input.id));
        Self::create_virtual(public_key_hash, api)
    }

    pub fn create_virtual_ed25519<Y>(
        input: VirtualLazyLoadInput,
        api: &mut Y,
    ) -> Result<BTreeMap<ObjectModuleId, Own>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let public_key_hash = PublicKeyHash::EddsaEd25519(EddsaEd25519PublicKeyHash(input.id));
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
                    value: scrypto_decode(
                        &scrypto_encode(&MetadataEntry::List(vec![MetadataValue::PublicKeyHash(
                            public_key_hash,
                        )]))
                        .unwrap(),
                    )
                    .unwrap(),
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
