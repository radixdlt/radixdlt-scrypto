use crate::blueprints::util::{PresecurifiedRoleAssignment, SecurifiedRoleAssignment};
use crate::errors::{ApplicationError, RuntimeError};
use crate::internal_prelude::*;
use crate::roles_template;
use radix_blueprint_schema_init::{
    BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, FunctionSchemaInit, ReceiverInfo,
    TypeRef,
};
use radix_blueprint_schema_init::{BlueprintSchemaInit, BlueprintStateSchemaInit};
use radix_engine_interface::api::{AttachedModuleId, SystemApi};
use radix_engine_interface::blueprints::hooks::{OnVirtualizeInput, OnVirtualizeOutput};
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::metadata_init;
use radix_engine_interface::object_modules::metadata::*;
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::modules::role_assignment::RoleAssignment;
use radix_native_sdk::modules::royalty::ComponentRoyalty;
use radix_native_sdk::runtime::Runtime;

pub const IDENTITY_ON_VIRTUALIZE_EXPORT_NAME: &str = "on_virtualize";

pub const IDENTITY_CREATE_PREALLOCATED_SECP256K1_ID: u8 = 0u8;
pub const IDENTITY_CREATE_PREALLOCATED_ED25519_ID: u8 = 1u8;

pub struct IdentityNativePackage;

impl IdentityNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let fields = Vec::new();

        let mut functions = index_map_new();
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

        let schema = generate_full_schema(aggregator);
        let blueprints = indexmap!(
            IDENTITY_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                is_transient: false,
                feature_set: indexset!(),
                dependencies: indexset!(
                    SECP256K1_SIGNATURE_RESOURCE.into(),
                    ED25519_SIGNATURE_RESOURCE.into(),
                    IDENTITY_OWNER_BADGE.into(),
                    PACKAGE_OF_DIRECT_CALLER_RESOURCE.into(),
                ),
                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    types: BlueprintTypeSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                    },
                    hooks: BlueprintHooksInit {
                        hooks: indexmap!(BlueprintHook::OnVirtualize => IDENTITY_ON_VIRTUALIZE_EXPORT_NAME.to_string())
                    }
                },
                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::StaticRoleDefinition(roles_template! {
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

    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        version: IdentityV1MinorVersion,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            IDENTITY_CREATE_ADVANCED_IDENT => {
                let input: IdentityCreateAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create_advanced(input.owner_role, version, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_CREATE_IDENT => {
                let _input: IdentityCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::create(version, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_SECURIFY_IDENT => {
                let _input: IdentitySecurifyToSingleBadgeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::securify(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            IDENTITY_ON_VIRTUALIZE_EXPORT_NAME => {
                let input: OnVirtualizeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = IdentityBlueprint::on_virtualize(input, version, api)?;

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

impl SecurifiedRoleAssignment for SecurifiedIdentity {
    type OwnerBadgeNonFungibleData = IdentityOwnerBadgeData;
    const OWNER_BADGE: ResourceAddress = IDENTITY_OWNER_BADGE;
    const SECURIFY_ROLE: Option<&'static str> = Some(SECURIFY_ROLE);
}

impl PresecurifiedRoleAssignment for SecurifiedIdentity {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Sbor)]
pub enum IdentityV1MinorVersion {
    Zero,
    One,
}

pub struct IdentityBlueprint;

impl IdentityBlueprint {
    pub fn create_advanced<Y: SystemApi<RuntimeError>>(
        owner_role: OwnerRole,
        version: IdentityV1MinorVersion,
        api: &mut Y,
    ) -> Result<GlobalAddress, RuntimeError> {
        let role_assignment = SecurifiedIdentity::create_advanced(owner_role, api)?;

        let (node_id, modules) = Self::create_object(
            role_assignment,
            metadata_init!(
                "owner_badge" => EMPTY, locked;
            ),
            version,
            api,
        )?;
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();
        let address = api.globalize(node_id, modules, None)?;
        Ok(address)
    }

    pub fn create<Y: SystemApi<RuntimeError>>(
        version: IdentityV1MinorVersion,
        api: &mut Y,
    ) -> Result<(GlobalAddress, Bucket), RuntimeError> {
        let (address_reservation, address) = api.allocate_global_address(BlueprintId {
            package_address: IDENTITY_PACKAGE,
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
        })?;
        let (role_assignment, bucket) = SecurifiedIdentity::create_securified(
            IdentityOwnerBadgeData {
                name: "Identity Owner Badge".to_string(),
                identity: address.try_into().expect("Impossible Case"),
            },
            Some(NonFungibleLocalId::bytes(address.as_node_id().0).unwrap()),
            api,
        )?;

        let (node_id, modules) = Self::create_object(
            role_assignment,
            metadata_init! {
                "owner_badge" => NonFungibleLocalId::bytes(address.as_node_id().0).unwrap(), locked;
            },
            version,
            api,
        )?;
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();
        let address = api.globalize(node_id, modules, Some(address_reservation))?;
        Ok((address, bucket))
    }

    pub fn on_virtualize<Y: SystemApi<RuntimeError>>(
        input: OnVirtualizeInput,
        version: IdentityV1MinorVersion,
        api: &mut Y,
    ) -> Result<OnVirtualizeOutput, RuntimeError> {
        match input.variant_id {
            IDENTITY_CREATE_PREALLOCATED_SECP256K1_ID => {
                let public_key_hash = PublicKeyHash::Secp256k1(Secp256k1PublicKeyHash(input.rid));
                Self::create_virtual(public_key_hash, input.address_reservation, version, api)
            }
            IDENTITY_CREATE_PREALLOCATED_ED25519_ID => {
                let public_key_hash = PublicKeyHash::Ed25519(Ed25519PublicKeyHash(input.rid));
                Self::create_virtual(public_key_hash, input.address_reservation, version, api)
            }
            x => Err(RuntimeError::ApplicationError(
                ApplicationError::PanicMessage(format!("Unexpected variant id: {:?}", x)),
            )),
        }
    }

    fn create_virtual<Y: SystemApi<RuntimeError>>(
        public_key_hash: PublicKeyHash,
        address_reservation: GlobalAddressReservation,
        version: IdentityV1MinorVersion,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let owner_badge = {
            let bytes = public_key_hash.get_hash_bytes();
            let entity_type = match public_key_hash {
                PublicKeyHash::Ed25519(..) => EntityType::GlobalPreallocatedEd25519Identity,
                PublicKeyHash::Secp256k1(..) => EntityType::GlobalPreallocatedSecp256k1Identity,
            };

            let mut id_bytes = vec![entity_type as u8];
            id_bytes.extend(bytes);

            NonFungibleLocalId::bytes(id_bytes).unwrap()
        };

        let owner_id = NonFungibleGlobalId::from_public_key_hash(public_key_hash);
        let role_assignment = SecurifiedIdentity::create_presecurified(owner_id, api)?;

        let (node_id, modules) = Self::create_object(
            role_assignment,
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
            version,
            api,
        )?;

        api.globalize(
            node_id,
            modules.into_iter().map(|(k, v)| (k, v.0)).collect(),
            Some(address_reservation),
        )?;
        Ok(())
    }

    fn securify<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Bucket, RuntimeError> {
        let receiver = Runtime::get_node_id(api)?;
        let owner_badge_data = IdentityOwnerBadgeData {
            name: "Identity Owner Badge".into(),
            identity: ComponentAddress::new_or_panic(receiver.0),
        };
        let bucket = SecurifiedIdentity::securify(
            &receiver,
            owner_badge_data,
            Some(NonFungibleLocalId::bytes(receiver.0).unwrap()),
            api,
        )?;
        Ok(bucket.into())
    }

    fn create_object<Y: SystemApi<RuntimeError>>(
        role_assignment: RoleAssignment,
        metadata_init: MetadataInit,
        version: IdentityV1MinorVersion,
        api: &mut Y,
    ) -> Result<(NodeId, IndexMap<AttachedModuleId, Own>), RuntimeError> {
        match version {
            IdentityV1MinorVersion::Zero => {
                let metadata = Metadata::create_with_data(metadata_init, api)?;
                let royalty = ComponentRoyalty::create(ComponentRoyaltyConfig::default(), api)?;
                let object_id = api.new_simple_object(IDENTITY_BLUEPRINT, indexmap!())?;
                let modules = indexmap!(
                    AttachedModuleId::RoleAssignment => role_assignment.0,
                    AttachedModuleId::Metadata => metadata,
                    AttachedModuleId::Royalty => royalty,
                );
                Ok((object_id, modules))
            }
            IdentityV1MinorVersion::One => {
                let metadata = Metadata::create_with_data(metadata_init, api)?;
                let object_id = api.new_simple_object(IDENTITY_BLUEPRINT, indexmap!())?;
                let modules = indexmap!(
                    AttachedModuleId::RoleAssignment => role_assignment.0,
                    AttachedModuleId::Metadata => metadata,
                );
                Ok((object_id, modules))
            }
        }
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
