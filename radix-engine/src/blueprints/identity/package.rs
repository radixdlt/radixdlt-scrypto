use crate::blueprints::util::{PresecurifiedAccessRules, SecurifiedAccessRules};
use crate::errors::{ApplicationError, RuntimeError};
use crate::roles_template;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::system_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::metadata_init;
use radix_engine_interface::schema::{
    BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, FunctionSchemaInit, ReceiverInfo,
    TypeRef,
};
use radix_engine_interface::schema::{BlueprintSchemaInit, BlueprintStateSchemaInit};

const IDENTITY_CREATE_VIRTUAL_SECP256K1_EXPORT_NAME: &str = "create_virtual_secp256k1";
const IDENTITY_CREATE_VIRTUAL_ED25519_EXPORT_NAME: &str = "create_virtual_ed25519";

pub struct IdentityNativePackage;

impl IdentityNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let fields = Vec::new();

        let mut functions = BTreeMap::new();
        functions.insert(
            IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<IdentityCreateAdvancedInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<IdentityCreateAdvancedOutput>(),
                ),
                export: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            },
        );
        functions.insert(
            IDENTITY_CREATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<IdentityCreateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<IdentityCreateOutput>(),
                ),
                export: IDENTITY_CREATE_IDENT.to_string(),
            },
        );
        functions.insert(
            IDENTITY_SECURIFY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<IdentitySecurifyToSingleBadgeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<IdentitySecurifyToSingleBadgeOutput>(),
                ),
                export: IDENTITY_SECURIFY_IDENT.to_string(),
            },
        );

        let virtual_lazy_load_functions = btreemap!(
            IDENTITY_CREATE_VIRTUAL_SECP256K1_ID => IDENTITY_CREATE_VIRTUAL_SECP256K1_EXPORT_NAME.to_string(),
            IDENTITY_CREATE_VIRTUAL_ED25519_ID => IDENTITY_CREATE_VIRTUAL_ED25519_EXPORT_NAME.to_string(),
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            IDENTITY_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                feature_set: btreeset!(),
                dependencies: btreeset!(
                    SECP256K1_SIGNATURE_VIRTUAL_BADGE.into(),
                    ED25519_SIGNATURE_VIRTUAL_BADGE.into(),
                    IDENTITY_OWNER_BADGE.into(),
                    PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                ),
                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        virtual_lazy_load_functions,
                        functions,
                    },
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::StaticRoles(roles_template! {
                        roles {
                            SECURIFY_ROLE => updaters: [SELF_ROLE];
                        },
                        methods {
                            IDENTITY_SECURIFY_IDENT => [SECURIFY_ROLE];
                        }
                    }),
                },
            }
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match export_name {
            IDENTITY_CREATE_ADVANCED_IDENT => {
                let input: IdentityCreateAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create_advanced(input.owner_rule, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_CREATE_IDENT => {
                let _input: IdentityCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_SECURIFY_IDENT => {
                let receiver = Runtime::get_node_id(api)?;
                let _input: IdentitySecurifyToSingleBadgeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::securify(&receiver, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_CREATE_VIRTUAL_SECP256K1_EXPORT_NAME => {
                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create_virtual_secp256k1(input, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_CREATE_VIRTUAL_ED25519_EXPORT_NAME => {
                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create_virtual_ed25519(input, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}

const SECURIFY_ROLE: &'static str = "securify";

struct SecurifiedIdentity;

impl SecurifiedAccessRules for SecurifiedIdentity {
    type OwnerBadgeNonFungibleData = IdentityOwnerBadgeData;
    const OWNER_BADGE: ResourceAddress = IDENTITY_OWNER_BADGE;
    const SECURIFY_ROLE: Option<&'static str> = Some(SECURIFY_ROLE);
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

        let modules = Self::create_object(
            access_rules,
            metadata_init!(
                "owner_badge" => EMPTY, locked;
            ),
            api,
        )?;
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();
        let address = api.globalize(modules, None)?;
        Ok(address)
    }

    pub fn create<Y>(api: &mut Y) -> Result<(GlobalAddress, Bucket), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (address_reservation, address) = api.allocate_global_address(BlueprintId {
            package_address: IDENTITY_PACKAGE,
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
        })?;
        let (access_rules, bucket) = SecurifiedIdentity::create_securified(
            IdentityOwnerBadgeData {
                name: "Identity Owner Badge".to_string(),
                identity: address.try_into().expect("Impossible Case"),
            },
            Some(NonFungibleLocalId::bytes(address.as_node_id().0).unwrap()),
            api,
        )?;

        let modules = Self::create_object(
            access_rules,
            metadata_init! {
                "owner_badge" => NonFungibleLocalId::bytes(address.as_node_id().0).unwrap(), locked;
            },
            api,
        )?;
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();
        let address = api.globalize(modules, Some(address_reservation))?;
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
        let owner_badge = {
            let bytes = public_key_hash.get_hash_bytes();
            let entity_type = match public_key_hash {
                PublicKeyHash::Ed25519(..) => EntityType::GlobalVirtualEd25519Account,
                PublicKeyHash::Secp256k1(..) => EntityType::GlobalVirtualSecp256k1Account,
            };

            let mut id_bytes = vec![entity_type as u8];
            id_bytes.extend(bytes);

            NonFungibleLocalId::bytes(id_bytes).unwrap()
        };

        let owner_id = NonFungibleGlobalId::from_public_key_hash(public_key_hash);
        let access_rules = SecurifiedIdentity::create_presecurified(owner_id, api)?;

        let modules = Self::create_object(
            access_rules,
            metadata_init! {
                // NOTE:
                // This is the owner key for ROLA. We choose to set this explicitly to simplify the
                // security-critical logic off-ledger. In particular, we want an owner to be able to
                // explicitly delete the owner keys. If we went with a "no metadata = assume default
                // public key hash", then this could cause unexpected security-critical behavior if
                // a user expected that deleting the metadata removed the owner keys.
                "owner_keys" => vec![public_key_hash], updatable;
                "owner_badge" => owner_badge, locked;
            },
            api,
        )?;

        Ok(modules)
    }

    fn securify<Y>(receiver: &NodeId, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let owner_badge_data = IdentityOwnerBadgeData {
            name: "Identity Owner Badge".into(),
            identity: ComponentAddress::new_or_panic(receiver.0),
        };
        SecurifiedIdentity::securify(
            &receiver,
            owner_badge_data,
            Some(NonFungibleLocalId::bytes(receiver.0).unwrap()),
            api,
        )
    }

    fn create_object<Y>(
        access_rules: AccessRules,
        metadata_init: MetadataInit,
        api: &mut Y,
    ) -> Result<BTreeMap<ObjectModuleId, Own>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let metadata = Metadata::create_with_data(metadata_init, api)?;
        let royalty = ComponentRoyalty::create(ComponentRoyaltyConfig::default(), api)?;

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

#[derive(ScryptoSbor)]
pub struct IdentityOwnerBadgeData {
    pub name: String,
    pub identity: ComponentAddress,
}

impl NonFungibleData for IdentityOwnerBadgeData {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}
