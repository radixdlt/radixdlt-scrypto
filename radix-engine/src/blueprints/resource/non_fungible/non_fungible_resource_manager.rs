use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use core::ops::Neg;
use radix_common::math::Decimal;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::{
    FieldValue, GenericArgs, KVEntry, SystemApi, ACTOR_REF_GLOBAL, ACTOR_STATE_SELF,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::MetadataInit;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_native_sdk::component::{globalize_object, globalize_object_with_inner_object_and_event};
use radix_native_sdk::runtime::Runtime;

declare_native_blueprint_state! {
    blueprint_ident: NonFungibleResourceManager,
    blueprint_snake_case: non_fungible_resource_manager,
    generics: {
        data: {
            ident: Data,
            description: "The non fungible data type, for a particular resource",
        }
    },
    features: {
        track_total_supply: {
            ident: TrackTotalSupply,
            description: "Enables total supply tracking of the resource",
        },
        vault_freeze: {
            ident: VaultFreeze,
            description: "Enabled if the resource can ever support freezing",
        },
        vault_recall: {
            ident: VaultRecall,
            description: "Enabled if the resource can ever support recall",
        },
        mint: {
            ident: Mint,
            description: "Enabled if the resource can ever support minting",
        },
        burn: {
            ident: Burn,
            description: "Enabled if the resource can ever support burning",
        },
    },
    fields: {
        id_type: {
            ident: IdType,
            field_type: {
                kind: StaticSingleVersioned,
            },
        },
        mutable_fields: {
            ident: MutableFields,
            field_type: {
                kind: StaticSingleVersioned,
            },
        },
        total_supply: {
            ident: TotalSupply,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::if_feature(NonFungibleResourceManagerFeature::TrackTotalSupply),
        },
    },
    collections: {
        data: KeyValue {
            entry_ident: Data,
            key_type: {
                kind: Static,
                content_type: NonFungibleLocalId,
            },
            value_type: {
                kind: Generic,
                ident: Data,
            },
            allow_ownership: false,
        },
    }
}

pub type NonFungibleResourceManagerIdTypeV1 = NonFungibleIdType;
pub type NonFungibleResourceManagerTotalSupplyV1 = Decimal;
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMutableFieldsV1 {
    pub mutable_field_index: IndexMap<String, usize>,
}

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NonFungibleResourceManagerError {
    NonFungibleAlreadyExists(Box<NonFungibleGlobalId>),
    NonFungibleNotFound(Box<NonFungibleGlobalId>),
    UnknownMutableFieldName(String),
    NonFungibleIdTypeDoesNotMatch(NonFungibleIdType, NonFungibleIdType),
    InvalidNonFungibleIdType,
    InvalidNonFungibleSchema(InvalidNonFungibleSchema),
    NonFungibleLocalIdProvidedForRUIDType,
    DropNonEmptyBucket,
    NotMintable,
    NotBurnable,
    UnexpectedDecimalComputationError,
}

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum InvalidNonFungibleSchema {
    SchemaValidationError(SchemaValidationError),
    InvalidLocalTypeId,
    NotATuple,
    MissingFieldNames,
    MutableFieldDoesNotExist(String),
}

fn create_non_fungibles<Y: SystemApi<RuntimeError>>(
    resource_address: ResourceAddress,
    id_type: NonFungibleIdType,
    entries: IndexMap<NonFungibleLocalId, ScryptoValue>,
    check_non_existence: bool,
    api: &mut Y,
) -> Result<IndexSet<NonFungibleLocalId>, RuntimeError> {
    let mut ids = index_set_new();
    for (non_fungible_local_id, value) in entries {
        if non_fungible_local_id.id_type() != id_type {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleIdTypeDoesNotMatch(
                        non_fungible_local_id.id_type(),
                        id_type,
                    ),
                ),
            ));
        }

        let non_fungible_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerCollection::DataKeyValue.collection_index(),
            &non_fungible_local_id.to_key(),
            LockFlags::MUTABLE,
        )?;

        if check_non_existence {
            let cur_non_fungible = api
                .key_value_entry_get_typed::<NonFungibleResourceManagerDataEntryPayload>(
                    non_fungible_handle,
                )?;

            if cur_non_fungible.is_some() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::NonFungibleAlreadyExists(Box::new(
                            NonFungibleGlobalId::new(resource_address, non_fungible_local_id),
                        )),
                    ),
                ));
            }
        }

        api.key_value_entry_set_typed(
            non_fungible_handle,
            NonFungibleResourceManagerDataEntryPayload::from_content_source(value),
        )?;
        api.key_value_entry_close(non_fungible_handle)?;
        ids.insert(non_fungible_local_id);
    }

    Ok(ids)
}

pub struct NonFungibleResourceManagerBlueprint;

impl NonFungibleResourceManagerBlueprint {
    pub fn get_definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let state = NonFungibleResourceManagerStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();
        functions.insert(
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateOutput>(),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerCreateWithInitialSupplyInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerCreateWithInitialSupplyOutput>()),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerCreateRuidWithInitialSupplyInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerCreateRuidWithInitialSupplyOutput>()),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT.to_string(),
            },
        );

        functions.insert(
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintOutput>(),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerGetNonFungibleInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerGetNonFungibleOutput>()),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT.to_string(),
            },
        );

        functions.insert(
            NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerUpdateDataInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerUpdateDataOutput>()),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerExistsInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerExistsOutput>(),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT.to_string(),
            },
        );

        functions.insert(
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintRuidInput>(
                        ),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintRuidOutput>(
                        ),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_RUID_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerMintSingleRuidInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerMintSingleRuidOutput>()),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_RUID_IDENT.to_string(),
            },
        );

        functions.insert(
            RESOURCE_MANAGER_PACKAGE_BURN_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ResourceManagerPackageBurnInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ResourceManagerPackageBurnOutput>(),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            RESOURCE_MANAGER_BURN_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ResourceManagerBurnInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ResourceManagerBurnOutput>(),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultOutput>(),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketOutput>(),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetResourceTypeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetResourceTypeOutput>(),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyOutput>(),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            RESOURCE_MANAGER_GET_AMOUNT_FOR_WITHDRAWAL_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetAmountForWithdrawalInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetAmountForWithdrawalOutput>(
                        ),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_AMOUNT_FOR_WITHDRAWAL_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketOutput>(),
                ),
                export: NON_FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME.to_string(),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                VaultCreationEvent,
                MintNonFungibleResourceEvent,
                BurnNonFungibleResourceEvent
            ]
        };

        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::Outer,
            is_transient: false,
            feature_set: NonFungibleResourceManagerFeatureSet::all_features(),
            dependencies: indexset!(),
            schema: BlueprintSchemaInit {
                generics: vec![GenericBound::Any],
                schema,
                state,
                events: event_schema,
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit::default(),
            },

            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::StaticRoleDefinition(roles_template! {
                    roles {
                        MINTER_ROLE => updaters: [MINTER_UPDATER_ROLE];
                        MINTER_UPDATER_ROLE => updaters: [MINTER_UPDATER_ROLE];
                        BURNER_ROLE => updaters: [BURNER_UPDATER_ROLE];
                        BURNER_UPDATER_ROLE => updaters: [BURNER_UPDATER_ROLE];
                        WITHDRAWER_ROLE => updaters: [WITHDRAWER_UPDATER_ROLE];
                        WITHDRAWER_UPDATER_ROLE => updaters: [WITHDRAWER_UPDATER_ROLE];
                        DEPOSITOR_ROLE => updaters: [DEPOSITOR_UPDATER_ROLE];
                        DEPOSITOR_UPDATER_ROLE => updaters: [DEPOSITOR_UPDATER_ROLE];
                        RECALLER_ROLE => updaters: [RECALLER_UPDATER_ROLE];
                        RECALLER_UPDATER_ROLE => updaters: [RECALLER_UPDATER_ROLE];
                        FREEZER_ROLE => updaters: [FREEZER_UPDATER_ROLE];
                        FREEZER_UPDATER_ROLE => updaters: [FREEZER_UPDATER_ROLE];
                        NON_FUNGIBLE_DATA_UPDATER_ROLE => updaters: [NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE];
                        NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE => updaters: [NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE];
                    },
                    methods {
                        NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT => [MINTER_ROLE];
                        NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT => [MINTER_ROLE];
                        NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_RUID_IDENT => [MINTER_ROLE];
                        RESOURCE_MANAGER_BURN_IDENT => [BURNER_ROLE];
                        RESOURCE_MANAGER_PACKAGE_BURN_IDENT => MethodAccessibility::OwnPackageOnly;
                        NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT => [NON_FUNGIBLE_DATA_UPDATER_ROLE];
                        RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT => MethodAccessibility::Public;
                        RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT => MethodAccessibility::Public;
                        RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT => MethodAccessibility::Public;
                        RESOURCE_MANAGER_GET_AMOUNT_FOR_WITHDRAWAL_IDENT => MethodAccessibility::Public;
                        RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT => MethodAccessibility::Public;
                        RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT => MethodAccessibility::Public;
                        NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT => MethodAccessibility::Public;
                        NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT => MethodAccessibility::Public;
                    }
                }),
            },
        }
    }

    fn resolve_and_validate_non_fungible_schema<Y: SystemApi<RuntimeError>>(
        schema: &NonFungibleDataSchema,
        api: &mut Y,
    ) -> Result<(GenericArgs, IndexMap<String, usize>), RuntimeError> {
        match schema {
            NonFungibleDataSchema::Local(LocalNonFungibleDataSchema {
                schema,
                type_id,
                mutable_fields,
            }) => {
                let schema_hash = schema.generate_schema_hash();
                let mutable_indices =
                    Self::validate_non_fungible_schema(schema, *type_id, mutable_fields, true)?;
                Ok((
                    GenericArgs {
                        additional_schema: Some(schema.clone()),
                        generic_substitutions: vec![GenericSubstitution::Local(ScopedTypeId(
                            schema_hash,
                            *type_id,
                        ))],
                    },
                    mutable_indices,
                ))
            }
            NonFungibleDataSchema::Remote(RemoteNonFungibleDataSchema {
                type_id,
                mutable_fields,
            }) => {
                let (schema, scoped_type_id) = api.resolve_blueprint_type(&type_id)?;
                let mutable_indices = Self::validate_non_fungible_schema(
                    &schema,
                    scoped_type_id.1,
                    mutable_fields,
                    false,
                )?;
                Ok((
                    GenericArgs {
                        additional_schema: None,
                        generic_substitutions: vec![GenericSubstitution::Remote(type_id.clone())],
                    },
                    mutable_indices,
                ))
            }
        }
    }

    fn validate_non_fungible_schema(
        schema: &VersionedScryptoSchema,
        local_type_id: LocalTypeId,
        mutable_fields: &IndexSet<String>,
        should_validate_schema: bool,
    ) -> Result<IndexMap<String, usize>, RuntimeError> {
        let mut mutable_field_index = indexmap!();

        // Validate schema
        if should_validate_schema {
            validate_schema(schema.v1()).map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                        InvalidNonFungibleSchema::SchemaValidationError(e),
                    ),
                ))
            })?;
        }

        // Validate type kind
        let type_kind =
            schema
                .v1()
                .resolve_type_kind(local_type_id)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                            InvalidNonFungibleSchema::InvalidLocalTypeId,
                        ),
                    ),
                ))?;

        if !matches!(type_kind, TypeKind::Tuple { .. }) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                        InvalidNonFungibleSchema::NotATuple,
                    ),
                ),
            ));
        }

        // Validate names
        let type_metadata = schema.v1().resolve_type_metadata(local_type_id).ok_or(
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                    InvalidNonFungibleSchema::InvalidLocalTypeId,
                ),
            )),
        )?;
        match &type_metadata.child_names {
            Some(ChildNames::NamedFields(names)) => {
                let allowed_names: IndexMap<_, _> = names
                    .iter()
                    .enumerate()
                    .map(|(i, x)| (x.as_ref(), i))
                    .collect();
                for f in mutable_fields {
                    if let Some(index) = allowed_names.get(f.as_str()) {
                        mutable_field_index.insert(f.to_string(), *index);
                    } else {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::NonFungibleResourceManagerError(
                                NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                                    InvalidNonFungibleSchema::MutableFieldDoesNotExist(
                                        f.to_string(),
                                    ),
                                ),
                            ),
                        ));
                    }
                }
            }
            _ => {
                if !mutable_fields.is_empty() {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::NonFungibleResourceManagerError(
                            NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                                InvalidNonFungibleSchema::MissingFieldNames,
                            ),
                        ),
                    ));
                }
            }
        }

        Ok(mutable_field_index)
    }

    pub(crate) fn create<Y: SystemApi<RuntimeError>>(
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        track_total_supply: bool,
        non_fungible_schema: NonFungibleDataSchema,
        resource_roles: NonFungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError> {
        let (object_id, roles) = Self::create_object(
            id_type,
            indexmap!(),
            track_total_supply,
            non_fungible_schema,
            resource_roles,
            api,
        )?;

        let address_reservation = match address_reservation {
            Some(address_reservation) => address_reservation,
            None => {
                let (reservation, _) = api.allocate_global_address(BlueprintId {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                })?;
                reservation
            }
        };

        let address = globalize_object(
            object_id,
            owner_role,
            address_reservation,
            roles,
            metadata,
            api,
        )?;

        Ok(ResourceAddress::new_or_panic(address.into()))
    }

    pub(crate) fn create_with_initial_supply<Y: SystemApi<RuntimeError>>(
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        track_total_supply: bool,
        non_fungible_schema: NonFungibleDataSchema,
        entries: IndexMap<NonFungibleLocalId, (ScryptoValue,)>,
        resource_roles: NonFungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError> {
        // TODO: Do this check in a better way (e.g. via type check)
        if id_type == NonFungibleIdType::RUID {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleLocalIdProvidedForRUIDType,
                ),
            ));
        }

        let ids: IndexSet<NonFungibleLocalId> = entries.keys().cloned().collect();

        let (object_id, roles) = Self::create_object(
            id_type,
            entries,
            track_total_supply,
            non_fungible_schema,
            resource_roles,
            api,
        )?;

        let address_reservation = match address_reservation {
            Some(address_reservation) => address_reservation,
            None => {
                let (reservation, _) = api.allocate_global_address(BlueprintId {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                })?;
                reservation
            }
        };

        let (resource_address, bucket) = globalize_object_with_inner_object_and_event(
            object_id,
            owner_role,
            address_reservation,
            roles,
            metadata,
            NON_FUNGIBLE_BUCKET_BLUEPRINT,
            indexmap! {
                NonFungibleBucketField::Liquid.field_index() => FieldValue::new(&LiquidNonFungibleResource::new(ids.clone())),
                NonFungibleBucketField::Locked.field_index() => FieldValue::new(&LockedNonFungibleResource::default()),
            },
            MintNonFungibleResourceEvent::EVENT_NAME,
            MintNonFungibleResourceEvent { ids },
            api,
        )?;

        Ok((
            ResourceAddress::new_or_panic(resource_address.into()),
            Bucket(Own(bucket)),
        ))
    }

    pub(crate) fn create_ruid_with_initial_supply<Y: SystemApi<RuntimeError>>(
        owner_role: OwnerRole,
        track_total_supply: bool,
        non_fungible_schema: NonFungibleDataSchema,
        entries: Vec<(ScryptoValue,)>,
        resource_roles: NonFungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError> {
        let entries: Result<IndexMap<NonFungibleLocalId, (ScryptoValue,)>, RuntimeError> = entries
            .into_iter()
            .map(|e| {
                let ruid = Runtime::generate_ruid(api)?;
                let id = NonFungibleLocalId::ruid(ruid);
                Ok((id, e))
            })
            .collect();
        let entries = entries?;

        let ids: IndexSet<NonFungibleLocalId> = entries.keys().cloned().collect();

        let (object_id, roles) = Self::create_object(
            NonFungibleIdType::RUID,
            entries,
            track_total_supply,
            non_fungible_schema,
            resource_roles,
            api,
        )?;

        let address_reservation = match address_reservation {
            Some(address_reservation) => address_reservation,
            None => {
                let (reservation, _) = api.allocate_global_address(BlueprintId {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                })?;
                reservation
            }
        };

        let (resource_address, bucket) = globalize_object_with_inner_object_and_event(
            object_id,
            owner_role,
            address_reservation,
            roles,
            metadata,
            NON_FUNGIBLE_BUCKET_BLUEPRINT,
            indexmap! {
                NonFungibleBucketField::Liquid.field_index() => FieldValue::new(&LiquidNonFungibleResource::new(ids.clone())),
                NonFungibleBucketField::Locked.field_index() => FieldValue::new(&LockedNonFungibleResource::default()),
            },
            MintNonFungibleResourceEvent::EVENT_NAME,
            MintNonFungibleResourceEvent { ids },
            api,
        )?;

        Ok((
            ResourceAddress::new_or_panic(resource_address.into()),
            Bucket(Own(bucket)),
        ))
    }

    pub(crate) fn mint_non_fungible<Y: SystemApi<RuntimeError>>(
        entries: IndexMap<NonFungibleLocalId, (ScryptoValue,)>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::assert_mintable(api)?;
        let id_type = Self::assert_is_not_ruid(api)?;
        Self::update_total_supply(api, entries.len().into())?;

        let ids = {
            let resource_address =
                ResourceAddress::new_or_panic(api.actor_get_node_id(ACTOR_REF_GLOBAL)?.into());
            let non_fungibles = entries.into_iter().map(|(k, v)| (k, v.0)).collect();
            create_non_fungibles(resource_address, id_type, non_fungibles, true, api)?
        };

        let bucket = Self::create_bucket(ids.clone(), api)?;
        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok(bucket)
    }

    pub(crate) fn mint_ruid_non_fungible<Y: SystemApi<RuntimeError>>(
        entries: Vec<(ScryptoValue,)>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::assert_mintable(api)?;
        Self::assert_is_ruid(api)?;
        Self::update_total_supply(api, entries.len().into())?;

        let ids = {
            let resource_address =
                ResourceAddress::new_or_panic(api.actor_get_node_id(ACTOR_REF_GLOBAL)?.into());
            let mut non_fungibles = index_map_new();
            for value in entries {
                let id = NonFungibleLocalId::ruid(Runtime::generate_ruid(api)?);
                non_fungibles.insert(id, value.0);
            }
            create_non_fungibles(
                resource_address,
                NonFungibleIdType::RUID,
                non_fungibles,
                false,
                api,
            )?
        };

        let bucket = Self::create_bucket(ids.clone(), api)?;
        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok(bucket)
    }

    pub(crate) fn mint_single_ruid_non_fungible<Y: SystemApi<RuntimeError>>(
        value: ScryptoValue,
        api: &mut Y,
    ) -> Result<(Bucket, NonFungibleLocalId), RuntimeError> {
        Self::assert_mintable(api)?;
        Self::assert_is_ruid(api)?;
        Self::update_total_supply(api, Decimal::ONE)?;

        let (id, ids) = {
            let resource_address =
                ResourceAddress::new_or_panic(api.actor_get_node_id(ACTOR_REF_GLOBAL)?.into());
            let id = NonFungibleLocalId::ruid(Runtime::generate_ruid(api)?);
            let non_fungibles = indexmap!(id.clone() => value);

            let ids = create_non_fungibles(
                resource_address,
                NonFungibleIdType::RUID,
                non_fungibles,
                false,
                api,
            )?;

            (id, ids)
        };

        let bucket = Self::create_bucket(ids.clone(), api)?;
        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok((bucket, id))
    }

    pub(crate) fn update_non_fungible_data<Y: SystemApi<RuntimeError>>(
        id: NonFungibleLocalId,
        field_name: String,
        data: ScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let field_index = {
            let data_schema_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                NonFungibleResourceManagerField::MutableFields.into(),
                LockFlags::read_only(),
            )?;
            let mutable_fields = api
                .field_read_typed::<NonFungibleResourceManagerMutableFieldsFieldPayload>(
                    data_schema_handle,
                )?
                .fully_update_and_into_latest_version();
            mutable_fields
                .mutable_field_index
                .get(&field_name)
                .cloned()
                .ok_or_else(|| {
                    RuntimeError::ApplicationError(
                        ApplicationError::NonFungibleResourceManagerError(
                            NonFungibleResourceManagerError::UnknownMutableFieldName(field_name),
                        ),
                    )
                })?
        };

        let non_fungible_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerCollection::DataKeyValue.collection_index(),
            &id.to_key(),
            LockFlags::MUTABLE,
        )?;

        let mut non_fungible_entry = api
            .key_value_entry_get_typed::<NonFungibleResourceManagerDataEntryPayload>(
                non_fungible_handle,
            )?;

        if let Some(ref mut non_fungible_data_payload) = non_fungible_entry {
            match non_fungible_data_payload.as_mut() {
                Value::Tuple { fields } => fields[field_index] = data,
                _ => panic!("Non-tuple non-fungible created: id = {}", id),
            }
            let buffer = scrypto_encode(non_fungible_data_payload).unwrap();
            api.key_value_entry_set(non_fungible_handle, buffer)?;
        } else {
            let resource_address =
                ResourceAddress::new_or_panic(api.actor_get_node_id(ACTOR_REF_GLOBAL)?.into());
            let non_fungible_global_id = NonFungibleGlobalId::new(resource_address, id);
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleNotFound(Box::new(
                        non_fungible_global_id,
                    )),
                ),
            ));
        }

        Ok(())
    }

    pub(crate) fn non_fungible_exists<Y: SystemApi<RuntimeError>>(
        id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        let non_fungible_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerCollection::DataKeyValue.collection_index(),
            &id.to_key(),
            LockFlags::read_only(),
        )?;
        let non_fungible = api
            .key_value_entry_get_typed::<NonFungibleResourceManagerDataEntryPayload>(
                non_fungible_handle,
            )?;

        Ok(non_fungible.is_some())
    }

    pub(crate) fn get_non_fungible<Y: SystemApi<RuntimeError>>(
        id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError> {
        let non_fungible_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerCollection::DataKeyValue.collection_index(),
            &id.to_key(),
            LockFlags::read_only(),
        )?;
        let wrapper = api.key_value_entry_get_typed::<NonFungibleResourceManagerDataEntryPayload>(
            non_fungible_handle,
        )?;
        if let Some(non_fungible) = wrapper {
            Ok(non_fungible.into_content())
        } else {
            let resource_address =
                ResourceAddress::new_or_panic(api.actor_get_node_id(ACTOR_REF_GLOBAL)?.into());
            let non_fungible_global_id = NonFungibleGlobalId::new(resource_address, id.clone());
            Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleNotFound(Box::new(
                        non_fungible_global_id,
                    )),
                ),
            ))
        }
    }

    pub(crate) fn create_empty_bucket<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::create_bucket(index_set_new(), api)
    }

    pub(crate) fn create_bucket<Y: SystemApi<RuntimeError>>(
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let bucket_id = api.new_simple_object(
            NON_FUNGIBLE_BUCKET_BLUEPRINT,
            indexmap! {
                NonFungibleBucketField::Liquid.into() => FieldValue::new(&LiquidNonFungibleResource::new(ids)),
                NonFungibleBucketField::Locked.into() => FieldValue::new(&LockedNonFungibleResource::default()),
            },
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub(crate) fn burn<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::burn_internal(bucket, api)
    }

    /// Only callable within this package - this is to allow the burning of tokens from a vault.
    pub(crate) fn package_burn<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::burn_internal(bucket, api)
    }

    fn burn_internal<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::assert_burnable(api)?;

        // Drop the bucket
        // This will fail if bucket is not an inner object of the current non-fungible resource
        let other_bucket = drop_non_fungible_bucket(bucket.0.as_node_id(), api)?;

        Self::update_total_supply(api, other_bucket.liquid.amount().neg())?;

        Runtime::emit_event(
            api,
            BurnNonFungibleResourceEvent {
                ids: other_bucket.liquid.ids().clone(),
            },
        )?;

        // Update
        {
            for id in other_bucket.liquid.into_ids() {
                let handle = api.actor_open_key_value_entry(
                    ACTOR_STATE_SELF,
                    NonFungibleResourceManagerCollection::DataKeyValue.collection_index(),
                    &id.to_key(),
                    LockFlags::MUTABLE,
                )?;
                api.key_value_entry_remove(handle)?;
                // Tombstone the non fungible
                // TODO: RUID non fungibles with no data don't need to go through this process
                api.key_value_entry_lock(handle)?;
                api.key_value_entry_close(handle)?;
            }
        }

        Ok(())
    }

    pub(crate) fn drop_empty_bucket<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let other_bucket = drop_non_fungible_bucket(bucket.0.as_node_id(), api)?;

        if other_bucket.liquid.amount().is_zero() {
            Ok(())
        } else {
            Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::DropNonEmptyBucket,
                ),
            ))
        }
    }

    pub(crate) fn create_empty_vault<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Own, RuntimeError> {
        let balance = LiquidNonFungibleVault {
            amount: Decimal::zero(),
        };
        let mut fields = indexmap! {
            NonFungibleVaultField::Balance.into() => FieldValue::new(&NonFungibleVaultBalanceFieldPayload::from_content_source(
                    balance,
                )),
            NonFungibleVaultField::LockedResource.into() => FieldValue::new(
                    &NonFungibleVaultLockedResourceFieldPayload::from_content_source(
                        LockedNonFungibleResource::default(),
                    ),
                ),
        };

        if api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerFeature::VaultFreeze.feature_name(),
        )? {
            fields.insert(
                NonFungibleVaultField::FreezeStatus.into(),
                FieldValue::new(
                    &NonFungibleVaultFreezeStatusFieldPayload::from_content_source(
                        VaultFrozenFlag::default(),
                    ),
                ),
            );
        }

        let vault_id = api.new_simple_object(NON_FUNGIBLE_VAULT_BLUEPRINT, fields)?;

        Runtime::emit_event(api, VaultCreationEvent { vault_id })?;

        Ok(Own(vault_id))
    }

    pub(crate) fn get_resource_type<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<ResourceType, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerField::IdType.into(),
            LockFlags::read_only(),
        )?;

        let id_type = api
            .field_read_typed::<NonFungibleResourceManagerIdTypeFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        let resource_type = ResourceType::NonFungible { id_type };

        Ok(resource_type)
    }

    pub(crate) fn get_total_supply<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Option<Decimal>, RuntimeError> {
        if api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerFeature::TrackTotalSupply.feature_name(),
        )? {
            let total_supply_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                NonFungibleResourceManagerField::TotalSupply.into(),
                LockFlags::read_only(),
            )?;
            let total_supply = api
                .field_read_typed::<NonFungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .fully_update_and_into_latest_version();
            Ok(Some(total_supply))
        } else {
            Ok(None)
        }
    }

    fn create_object<Y: SystemApi<RuntimeError>>(
        id_type: NonFungibleIdType,
        entries: IndexMap<NonFungibleLocalId, (ScryptoValue,)>,
        track_total_supply: bool,
        non_fungible_schema: NonFungibleDataSchema,
        resource_roles: NonFungibleResourceRoles,
        api: &mut Y,
    ) -> Result<(NodeId, RoleAssignmentInit), RuntimeError> {
        let (generic_args, mutable_field_index) =
            Self::resolve_and_validate_non_fungible_schema(&non_fungible_schema, api)?;

        let mutable_fields = NonFungibleResourceManagerMutableFields {
            mutable_field_index,
        };

        let supply: Decimal = Decimal::from(entries.len());

        let mut non_fungibles = index_map_new();
        for (id, (value,)) in entries {
            if id.id_type() != id_type {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::NonFungibleIdTypeDoesNotMatch(
                            id.id_type(),
                            id_type,
                        ),
                    ),
                ));
            }

            let kv_entry = KVEntry {
                value: Some(scrypto_encode(&value).unwrap()),
                locked: false,
            };

            non_fungibles.insert(scrypto_encode(&id).unwrap(), kv_entry);
        }

        let (mut features, roles) = to_features_and_roles(resource_roles);
        features.track_total_supply = track_total_supply;

        let mut fields = indexmap! {
            NonFungibleResourceManagerField::IdType.into() => FieldValue::immutable(
                    &NonFungibleResourceManagerIdTypeFieldPayload::from_content_source(id_type),
                ),
            NonFungibleResourceManagerField::MutableFields.into() => FieldValue::immutable(
                    &NonFungibleResourceManagerMutableFieldsFieldPayload::from_content_source(
                        mutable_fields,
                    ),
                )
        };

        if track_total_supply {
            let total_supply_field = if features.mint || features.burn {
                FieldValue::new(
                    &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(supply),
                )
            } else {
                FieldValue::immutable(
                    &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(supply),
                )
            };

            fields.insert(
                NonFungibleResourceManagerField::TotalSupply.into(),
                total_supply_field,
            );
        }

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            features.feature_names_str(),
            generic_args,
            fields,
            indexmap!(NonFungibleResourceManagerCollection::DataKeyValue.collection_index() => non_fungibles),
        )?;

        Ok((object_id, roles))
    }

    fn assert_is_not_ruid<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<NonFungibleIdType, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerField::IdType.into(),
            LockFlags::read_only(),
        )?;
        let id_type = api
            .field_read_typed::<NonFungibleResourceManagerIdTypeFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        api.field_close(handle)?;
        if id_type == NonFungibleIdType::RUID {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                ),
            ));
        }
        Ok(id_type)
    }

    fn assert_is_ruid<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        // Check type
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerField::IdType.into(),
            LockFlags::read_only(),
        )?;
        let id_type = api
            .field_read_typed::<NonFungibleResourceManagerIdTypeFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        api.field_close(handle)?;

        if id_type != NonFungibleIdType::RUID {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                ),
            ));
        }

        Ok(())
    }

    fn assert_mintable<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        if !api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerFeature::Mint.feature_name(),
        )? {
            // This should never be hit since the auth layer will prevent
            // any mint call from even getting to this point but this is useful
            // if the Auth layer is ever disabled for whatever reason.
            // We still want to maintain these invariants.
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NotMintable,
                ),
            ));
        }

        return Ok(());
    }

    fn assert_burnable<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        if !api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerFeature::Burn.feature_name(),
        )? {
            // This should never be hit since the auth layer will prevent
            // any burn call from even getting to this point but this is useful
            // if the Auth layer is ever disabled for whatever reason.
            // We still want to maintain these invariants.
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NotBurnable,
                ),
            ));
        }

        return Ok(());
    }

    fn update_total_supply<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), RuntimeError> {
        if api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            NonFungibleResourceManagerFeature::TrackTotalSupply.feature_name(),
        )? {
            let total_supply_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                NonFungibleResourceManagerField::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply = api
                .field_read_typed::<NonFungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .fully_update_and_into_latest_version();
            total_supply =
                total_supply
                    .checked_add(amount)
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::NonFungibleResourceManagerError(
                            NonFungibleResourceManagerError::UnexpectedDecimalComputationError,
                        ),
                    ))?;
            api.field_write_typed(
                total_supply_handle,
                &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(
                    total_supply,
                ),
            )?;
        }

        Ok(())
    }

    pub(crate) fn amount_for_withdrawal<Y: SystemApi<RuntimeError>>(
        _api: &mut Y,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
    ) -> Result<Decimal, RuntimeError> {
        Ok(amount
            .for_withdrawal(0, withdraw_strategy)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::UnexpectedDecimalComputationError,
                ),
            ))?)
    }
}

fn to_features_and_roles(
    role_init: NonFungibleResourceRoles,
) -> (NonFungibleResourceManagerFeatureSet, RoleAssignmentInit) {
    let mut roles = RoleAssignmentInit::new();

    let features = NonFungibleResourceManagerFeatureSet {
        track_total_supply: false, // Will be set later
        vault_freeze: role_init.freeze_roles.is_some(),
        vault_recall: role_init.recall_roles.is_some(),
        mint: role_init.mint_roles.is_some(),
        burn: role_init.burn_roles.is_some(),
    };

    roles
        .data
        .extend(role_init.mint_roles.unwrap_or_default().to_role_init().data);
    roles
        .data
        .extend(role_init.burn_roles.unwrap_or_default().to_role_init().data);
    roles.data.extend(
        role_init
            .recall_roles
            .unwrap_or_default()
            .to_role_init()
            .data,
    );
    roles.data.extend(
        role_init
            .freeze_roles
            .unwrap_or_default()
            .to_role_init()
            .data,
    );
    roles.data.extend(
        role_init
            .deposit_roles
            .unwrap_or_default()
            .to_role_init()
            .data,
    );
    roles.data.extend(
        role_init
            .withdraw_roles
            .unwrap_or_default()
            .to_role_init()
            .data,
    );
    roles.data.extend(
        role_init
            .non_fungible_data_update_roles
            .unwrap_or_default()
            .to_role_init()
            .data,
    );

    (features, roles)
}
