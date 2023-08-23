use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::CollectionDescriptor;
use radix_engine_interface::api::{
    ClientApi, CollectionIndex, FieldValue, GenericArgs, KVEntry, OBJECT_HANDLE_SELF,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::*;

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
    InvalidRole(String),
    UnknownMutableFieldName(String),
    NonFungibleIdTypeDoesNotMatch(NonFungibleIdType, NonFungibleIdType),
    InvalidNonFungibleIdType,
    InvalidNonFungibleSchema(InvalidNonFungibleSchema),
    NonFungibleLocalIdProvidedForRUIDType,
    DropNonEmptyBucket,
    NotMintable,
    NotBurnable,
}

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum InvalidNonFungibleSchema {
    SchemaValidationError(SchemaValidationError),
    InvalidLocalTypeIndex,
    NotATuple,
    MissingFieldNames,
    MutableFieldDoesNotExist(String),
}

fn create_non_fungibles<Y>(
    resource_address: ResourceAddress,
    id_type: NonFungibleIdType,
    entries: BTreeMap<NonFungibleLocalId, ScryptoValue>,
    check_non_existence: bool,
    api: &mut Y,
) -> Result<(), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let mut ids = BTreeSet::new();
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
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerCollection::DataKeyValue,
            &non_fungible_local_id.to_key(),
            LockFlags::MUTABLE,
        )?;

        if check_non_existence {
            let cur_non_fungible = api
                .key_value_entry_get_typed::<NonFungibleResourceManagerDataEntryPayload>(
                    non_fungible_handle,
                )?;

            if let Some(..) = cur_non_fungible {
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

    Ok(())
}

pub struct NonFungibleResourceManagerBlueprint;

impl NonFungibleResourceManagerBlueprint {
    pub fn get_definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let state = NonFungibleResourceManagerStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = BTreeMap::new();
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
            dependencies: btreeset!(),
            schema: BlueprintSchemaInit {
                generics: vec![GenericBound::Any],
                schema,
                state,
                events: event_schema,
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

    fn validate_non_fungible_schema(
        non_fungible_schema: &NonFungibleDataSchema,
    ) -> Result<IndexMap<String, usize>, RuntimeError> {
        let mut mutable_field_index = indexmap!();

        // Validate schema
        validate_schema(&non_fungible_schema.schema).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                    InvalidNonFungibleSchema::SchemaValidationError(e),
                ),
            ))
        })?;

        // Validate type kind
        let type_kind = non_fungible_schema
            .schema
            .resolve_type_kind(non_fungible_schema.non_fungible)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                        InvalidNonFungibleSchema::InvalidLocalTypeIndex,
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
        let type_metadata = non_fungible_schema
            .schema
            .resolve_type_metadata(non_fungible_schema.non_fungible)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::InvalidNonFungibleSchema(
                        InvalidNonFungibleSchema::InvalidLocalTypeIndex,
                    ),
                ),
            ))?;
        match &type_metadata.child_names {
            Some(ChildNames::NamedFields(names)) => {
                let allowed_names: IndexMap<_, _> = names
                    .iter()
                    .enumerate()
                    .map(|(i, x)| (x.as_ref(), i))
                    .collect();
                for f in &non_fungible_schema.mutable_fields {
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
                if !non_fungible_schema.mutable_fields.is_empty() {
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

    pub(crate) fn create<Y>(
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        track_total_supply: bool,
        non_fungible_schema: NonFungibleDataSchema,
        resource_roles: NonFungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mutable_field_index = Self::validate_non_fungible_schema(&non_fungible_schema)?;

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

        let mutable_fields = NonFungibleResourceManagerMutableFields {
            mutable_field_index,
        };

        let schema_hash = non_fungible_schema.schema.generate_schema_hash();

        let generic_args = GenericArgs {
            additional_schema: Some(non_fungible_schema.schema),
            generic_substitutions: vec![GenericSubstitution::Local(TypeIdentifier(
                schema_hash,
                non_fungible_schema.non_fungible,
            ))],
        };

        let (mut features, roles) = to_features_and_roles(resource_roles);
        features.track_total_supply = track_total_supply;

        let total_supply_field = if features.mint || features.burn {
            FieldValue::new(
                &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(
                    Decimal::zero(),
                ),
            )
        } else {
            FieldValue::immutable(
                &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(
                    Decimal::zero(),
                ),
            )
        };

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            features.feature_names_str(),
            generic_args,
            vec![
                FieldValue::immutable(
                    &NonFungibleResourceManagerIdTypeFieldPayload::from_content_source(id_type),
                ),
                FieldValue::immutable(
                    &NonFungibleResourceManagerMutableFieldsFieldPayload::from_content_source(
                        mutable_fields,
                    ),
                ),
                total_supply_field,
            ],
            btreemap!(),
        )?;

        globalize_resource_manager(
            owner_role,
            object_id,
            address_reservation,
            roles,
            metadata,
            api,
        )
    }

    pub(crate) fn create_with_initial_supply<Y>(
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        track_total_supply: bool,
        non_fungible_schema: NonFungibleDataSchema,
        entries: BTreeMap<NonFungibleLocalId, (ScryptoValue,)>,
        resource_roles: NonFungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let mutable_field_index = Self::validate_non_fungible_schema(&non_fungible_schema)?;

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

        // TODO: Do this check in a better way (e.g. via type check)
        if id_type == NonFungibleIdType::RUID {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleLocalIdProvidedForRUIDType,
                ),
            ));
        }

        let mutable_fields = NonFungibleResourceManagerMutableFields {
            mutable_field_index,
        };

        let supply: Decimal = Decimal::from(entries.len());

        let ids = entries.keys().cloned().collect();

        let mut non_fungibles = BTreeMap::new();
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

        let schema_hash = non_fungible_schema.schema.generate_schema_hash();
        let generic_args = GenericArgs {
            additional_schema: Some(non_fungible_schema.schema),
            generic_substitutions: vec![GenericSubstitution::Local(TypeIdentifier(
                schema_hash,
                non_fungible_schema.non_fungible,
            ))],
        };

        let (mut features, roles) = to_features_and_roles(resource_roles);
        features.track_total_supply = track_total_supply;

        let total_supply_field = if features.mint || features.burn {
            FieldValue::new(
                &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(supply),
            )
        } else {
            FieldValue::immutable(
                &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(supply),
            )
        };

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            features.feature_names_str(),
            generic_args,
            vec![
                FieldValue::immutable(&NonFungibleResourceManagerIdTypeFieldPayload::from_content_source(id_type)),
                FieldValue::immutable(&NonFungibleResourceManagerMutableFieldsFieldPayload::from_content_source(mutable_fields)),
                total_supply_field,
            ],
            btreemap!(NonFungibleResourceManagerCollection::DataKeyValue.collection_index() => non_fungibles),
        )?;
        let (resource_address, bucket) = globalize_non_fungible_with_initial_supply(
            owner_role,
            object_id,
            address_reservation,
            roles,
            metadata,
            ids,
            api,
        )?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn create_ruid_with_initial_supply<Y>(
        owner_role: OwnerRole,
        track_total_supply: bool,
        non_fungible_schema: NonFungibleDataSchema,
        entries: Vec<(ScryptoValue,)>,
        resource_roles: NonFungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let mutable_field_index = Self::validate_non_fungible_schema(&non_fungible_schema)?;

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

        let mut ids = BTreeSet::new();
        let mut non_fungibles = BTreeMap::new();
        let supply = Decimal::from(entries.len());
        for (entry,) in entries {
            let ruid = Runtime::generate_ruid(api)?;
            let id = NonFungibleLocalId::ruid(ruid);
            ids.insert(id.clone());
            let kv_entry = KVEntry {
                value: Some(scrypto_encode(&entry).unwrap()),
                locked: false,
            };
            non_fungibles.insert(scrypto_encode(&id).unwrap(), kv_entry);
        }

        let mutable_fields = NonFungibleResourceManagerMutableFields {
            mutable_field_index,
        };

        let schema_hash = non_fungible_schema.schema.generate_schema_hash();
        let generic_args = GenericArgs {
            additional_schema: Some(non_fungible_schema.schema),
            generic_substitutions: vec![GenericSubstitution::Local(TypeIdentifier(
                schema_hash,
                non_fungible_schema.non_fungible,
            ))],
        };

        let (mut features, roles) = to_features_and_roles(resource_roles);
        features.track_total_supply = track_total_supply;

        let total_supply_field = if features.mint || features.burn {
            FieldValue::new(
                &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(supply),
            )
        } else {
            FieldValue::immutable(
                &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(supply),
            )
        };

        let object_id = api.new_object(
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            features.feature_names_str(),
            generic_args,
            vec![
                FieldValue::immutable(&NonFungibleResourceManagerIdTypeFieldPayload::from_content_source(NonFungibleIdType::RUID)),
                FieldValue::immutable(&NonFungibleResourceManagerMutableFieldsFieldPayload::from_content_source(mutable_fields)),
                total_supply_field,
            ],
            btreemap!(NonFungibleResourceManagerCollection::DataKeyValue.collection_index() => non_fungibles),
        )?;
        let (resource_address, bucket) = globalize_non_fungible_with_initial_supply(
            owner_role,
            object_id,
            address_reservation,
            roles,
            metadata,
            ids,
            api,
        )?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn mint_non_fungible<Y>(
        entries: BTreeMap<NonFungibleLocalId, (ScryptoValue,)>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_mintable(api)?;

        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_global_address()?.into());
        let id_type = {
            let handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::IdType,
                LockFlags::read_only(),
            )?;
            let id_type = api
                .field_read_typed::<NonFungibleResourceManagerIdTypeFieldPayload>(handle)?
                .into_latest();
            api.field_close(handle)?;
            if id_type == NonFungibleIdType::RUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }
            id_type
        };

        // Update total supply
        // TODO: Could be further cleaned up by using event
        if api.actor_is_feature_enabled(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerFeature::TrackTotalSupply,
        )? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::TotalSupply,
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api
                .field_read_typed::<NonFungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .into_latest();
            total_supply = total_supply.safe_add(entries.len()).unwrap();
            api.field_write_typed(
                total_supply_handle,
                &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(
                    total_supply,
                ),
            )?;
        }

        let ids = {
            let ids: BTreeSet<NonFungibleLocalId> = entries.keys().cloned().collect();
            let non_fungibles = entries.into_iter().map(|(k, v)| (k, v.0)).collect();
            create_non_fungibles(resource_address, id_type, non_fungibles, true, api)?;

            ids
        };

        let bucket = Self::create_bucket(ids.clone(), api)?;
        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok(bucket)
    }

    pub(crate) fn mint_single_ruid_non_fungible<Y>(
        value: ScryptoValue,
        api: &mut Y,
    ) -> Result<(Bucket, NonFungibleLocalId), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_mintable(api)?;

        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_global_address()?.into());

        // Check id_type
        let id_type = {
            let id_type_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::IdType,
                LockFlags::read_only(),
            )?;
            let id_type: NonFungibleIdType = api
                .field_read_typed::<NonFungibleResourceManagerIdTypeFieldPayload>(id_type_handle)?
                .into_latest();
            api.field_close(id_type_handle)?;

            if id_type != NonFungibleIdType::RUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }

            id_type
        };

        // Update Total Supply
        // TODO: Could be further cleaned up by using event
        if api.actor_is_feature_enabled(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerFeature::TrackTotalSupply,
        )? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::TotalSupply,
                LockFlags::MUTABLE,
            )?;
            let mut total_supply = api
                .field_read_typed::<NonFungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .into_latest();
            total_supply = total_supply.safe_add(1).unwrap();
            api.field_write_typed(
                total_supply_handle,
                &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(
                    total_supply,
                ),
            )?;
        }

        let id = {
            let id = NonFungibleLocalId::ruid(Runtime::generate_ruid(api)?);
            let non_fungibles = btreemap!(id.clone() => value);

            create_non_fungibles(resource_address, id_type, non_fungibles, false, api)?;

            id
        };

        let ids = btreeset!(id.clone());
        let bucket = Self::create_bucket(ids.clone(), api)?;
        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok((bucket, id))
    }

    pub(crate) fn mint_ruid_non_fungible<Y>(
        entries: Vec<(ScryptoValue,)>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_mintable(api)?;

        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_global_address()?.into());

        // Check type
        let id_type = {
            let handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::IdType,
                LockFlags::read_only(),
            )?;
            let id_type = api
                .field_read_typed::<NonFungibleResourceManagerIdTypeFieldPayload>(handle)?
                .into_latest();
            api.field_close(handle)?;

            if id_type != NonFungibleIdType::RUID {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleResourceManagerError(
                        NonFungibleResourceManagerError::InvalidNonFungibleIdType,
                    ),
                ));
            }
            id_type
        };

        // Update total supply
        // TODO: there might be better for maintaining total supply, especially for non-fungibles
        if api.actor_is_feature_enabled(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerFeature::TrackTotalSupply,
        )? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::TotalSupply,
                LockFlags::MUTABLE,
            )?;
            let mut total_supply = api
                .field_read_typed::<NonFungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .into_latest();
            total_supply = total_supply.safe_add(entries.len()).unwrap();
            api.field_write_typed(
                total_supply_handle,
                &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(
                    total_supply,
                ),
            )?;
        }

        // Update data
        let ids = {
            let mut ids = BTreeSet::new();
            let mut non_fungibles = BTreeMap::new();
            for value in entries {
                let id = NonFungibleLocalId::ruid(Runtime::generate_ruid(api)?);
                ids.insert(id.clone());
                non_fungibles.insert(id, value.0);
            }
            create_non_fungibles(resource_address, id_type, non_fungibles, false, api)?;

            ids
        };

        let bucket = Self::create_bucket(ids.clone(), api)?;
        Runtime::emit_event(api, MintNonFungibleResourceEvent { ids })?;

        Ok(bucket)
    }

    pub(crate) fn update_non_fungible_data<Y>(
        id: NonFungibleLocalId,
        field_name: String,
        data: ScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_global_address()?.into());
        let data_schema_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerField::MutableFields,
            LockFlags::read_only(),
        )?;
        let mutable_fields = api
            .field_read_typed::<NonFungibleResourceManagerMutableFieldsFieldPayload>(
                data_schema_handle,
            )?
            .into_latest();

        let field_index = mutable_fields
            .mutable_field_index
            .get(&field_name)
            .cloned()
            .ok_or_else(|| {
                RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::UnknownMutableFieldName(field_name),
                ))
            })?;

        let non_fungible_handle = api.actor_open_key_value_entry(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerCollection::DataKeyValue,
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
            let non_fungible_global_id = NonFungibleGlobalId::new(resource_address, id);
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NonFungibleNotFound(Box::new(
                        non_fungible_global_id,
                    )),
                ),
            ));
        }

        api.key_value_entry_close(non_fungible_handle)?;

        Ok(())
    }

    pub(crate) fn non_fungible_exists<Y>(
        id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let non_fungible_handle = api.actor_open_key_value_entry(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerCollection::DataKeyValue,
            &id.to_key(),
            LockFlags::read_only(),
        )?;
        let non_fungible = api
            .key_value_entry_get_typed::<NonFungibleResourceManagerDataEntryPayload>(
                non_fungible_handle,
            )?;
        let exists = matches!(non_fungible, Option::Some(..));

        Ok(exists)
    }

    pub(crate) fn get_non_fungible<Y>(
        id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_global_address()?.into());

        let non_fungible_handle = api.actor_open_key_value_entry(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerCollection::DataKeyValue,
            &id.to_key(),
            LockFlags::read_only(),
        )?;
        let wrapper = api.key_value_entry_get_typed::<NonFungibleResourceManagerDataEntryPayload>(
            non_fungible_handle,
        )?;
        if let Some(non_fungible) = wrapper {
            Ok(non_fungible.into_content())
        } else {
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

    pub(crate) fn create_empty_bucket<Y>(api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::create_bucket(BTreeSet::new(), api)
    }

    pub(crate) fn create_bucket<Y>(
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let bucket_id = api.new_simple_object(
            NON_FUNGIBLE_BUCKET_BLUEPRINT,
            vec![
                FieldValue::new(&LiquidNonFungibleResource::new(ids)),
                FieldValue::new(&LockedNonFungibleResource::default()),
            ],
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub(crate) fn burn<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::burn_internal(bucket, api)
    }

    /// Only callable within this package - this is to allow the burning of tokens from a vault.
    pub(crate) fn package_burn<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::burn_internal(bucket, api)
    }

    fn burn_internal<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_burnable(api)?;

        // Drop the bucket
        let other_bucket = drop_non_fungible_bucket(bucket.0.as_node_id(), api)?;

        // Construct the event and only emit it once all of the operations are done.
        Runtime::emit_event(
            api,
            BurnNonFungibleResourceEvent {
                ids: other_bucket.liquid.ids().clone(),
            },
        )?;

        // Update total supply
        // TODO: there might be better for maintaining total supply, especially for non-fungibles
        if api.actor_is_feature_enabled(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerFeature::TrackTotalSupply,
        )? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::TotalSupply,
                LockFlags::MUTABLE,
            )?;
            let mut total_supply = api
                .field_read_typed::<NonFungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .into_latest();
            total_supply = total_supply.safe_sub(other_bucket.liquid.amount()).unwrap();
            api.field_write_typed(
                total_supply_handle,
                &NonFungibleResourceManagerTotalSupplyFieldPayload::from_content_source(
                    total_supply,
                ),
            )?;
        }

        // Update
        {
            for id in other_bucket.liquid.into_ids() {
                let handle = api.actor_open_key_value_entry(
                    OBJECT_HANDLE_SELF,
                    NonFungibleResourceManagerCollection::DataKeyValue,
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

    pub(crate) fn drop_empty_bucket<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
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

    pub(crate) fn create_empty_vault<Y>(api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let balance = LiquidNonFungibleVault {
            amount: Decimal::zero(),
        };

        let vault_id = api.new_simple_object(
            NON_FUNGIBLE_VAULT_BLUEPRINT,
            vec![
                FieldValue::new(&NonFungibleVaultBalanceFieldPayload::from_content_source(
                    balance,
                )),
                FieldValue::new(
                    &NonFungibleVaultLockedResourceFieldPayload::from_content_source(
                        LockedNonFungibleResource::default(),
                    ),
                ),
                FieldValue::new(
                    &NonFungibleVaultFreezeStatusFieldPayload::from_content_source(
                        VaultFrozenFlag::default(),
                    ),
                ),
            ],
        )?;

        Runtime::emit_event(api, VaultCreationEvent { vault_id })?;

        Ok(Own(vault_id))
    }

    pub(crate) fn get_resource_type<Y>(api: &mut Y) -> Result<ResourceType, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerField::IdType,
            LockFlags::read_only(),
        )?;

        let id_type = api
            .field_read_typed::<NonFungibleResourceManagerIdTypeFieldPayload>(handle)?
            .into_latest();
        let resource_type = ResourceType::NonFungible { id_type };

        Ok(resource_type)
    }

    pub(crate) fn get_total_supply<Y>(api: &mut Y) -> Result<Option<Decimal>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if api.actor_is_feature_enabled(
            OBJECT_HANDLE_SELF,
            NonFungibleResourceManagerFeature::TrackTotalSupply,
        )? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                NonFungibleResourceManagerField::TotalSupply,
                LockFlags::read_only(),
            )?;
            let total_supply = api
                .field_read_typed::<NonFungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .into_latest();
            Ok(Some(total_supply))
        } else {
            Ok(None)
        }
    }

    fn assert_mintable<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api
            .actor_is_feature_enabled(OBJECT_HANDLE_SELF, NonFungibleResourceManagerFeature::Mint)?
        {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NotMintable,
                ),
            ));
        }

        return Ok(());
    }

    fn assert_burnable<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api
            .actor_is_feature_enabled(OBJECT_HANDLE_SELF, NonFungibleResourceManagerFeature::Burn)?
        {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleResourceManagerError(
                    NonFungibleResourceManagerError::NotBurnable,
                ),
            ));
        }

        return Ok(());
    }

    pub(crate) fn amount_for_withdrawal<Y>(
        _api: &mut Y,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
    ) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Ok(amount.for_withdrawal(0, withdraw_strategy))
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
