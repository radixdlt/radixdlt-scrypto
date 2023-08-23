use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use lazy_static::lazy_static;
use native_sdk::runtime::Runtime;
use num_traits::pow::Pow;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::{ClientApi, FieldValue, GenericArgs, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::*;

const DIVISIBILITY_MAXIMUM: u8 = 18;

lazy_static! {
    static ref MAX_MINT_AMOUNT: Decimal = Decimal(I192::from(2).pow(160)); // 2^160 subunits
}

declare_native_blueprint_state! {
    blueprint_ident: FungibleResourceManager,
    blueprint_snake_case: fungible_resource_manager,
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
        divisibility: {
            ident: Divisibility,
            field_type: {
                kind: StaticSingleVersioned,
            },
        },
        total_supply: {
            ident: TotalSupply,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::if_feature(FungibleResourceManagerFeature::TrackTotalSupply),
        },
    },
    collections: {}
}

pub type FungibleResourceManagerDivisibilityV1 = u8;
pub type FungibleResourceManagerTotalSupplyV1 = Decimal;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum FungibleResourceManagerError {
    InvalidRole(String),
    InvalidAmount(Decimal, u8),
    MaxMintAmountExceeded,
    InvalidDivisibility(u8),
    DropNonEmptyBucket,
    NotMintable,
    NotBurnable,
}

pub fn verify_divisibility(divisibility: u8) -> Result<(), RuntimeError> {
    if divisibility > DIVISIBILITY_MAXIMUM {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::FungibleResourceManagerError(
                FungibleResourceManagerError::InvalidDivisibility(divisibility),
            ),
        ));
    }

    Ok(())
}

fn check_mint_amount(divisibility: u8, amount: Decimal) -> Result<(), RuntimeError> {
    if !check_fungible_amount(&amount, divisibility) {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::FungibleResourceManagerError(
                FungibleResourceManagerError::InvalidAmount(amount, divisibility),
            ),
        ));
    }

    if amount > *MAX_MINT_AMOUNT {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::FungibleResourceManagerError(
                FungibleResourceManagerError::MaxMintAmountExceeded,
            ),
        ));
    }

    Ok(())
}

fn to_features_and_roles(
    role_init: FungibleResourceRoles,
) -> (FungibleResourceManagerFeatureSet, RoleAssignmentInit) {
    let mut roles = RoleAssignmentInit::new();

    let features = FungibleResourceManagerFeatureSet {
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

    (features, roles)
}

pub struct FungibleResourceManagerBlueprint;

impl FungibleResourceManagerBlueprint {
    pub fn get_definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let state = FungibleResourceManagerStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = BTreeMap::new();
        functions.insert(
            FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateOutput>(),
                ),
                export: FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<FungibleResourceManagerCreateWithInitialSupplyInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<FungibleResourceManagerCreateWithInitialSupplyOutput>()),
                export: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<FungibleResourceManagerMintInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerMintOutput>(),
                ),
                export: FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_RESOURCE_MANAGER_AMOUNT_FOR_WITHDRAWAL_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME.to_string(),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                VaultCreationEvent,
                MintFungibleResourceEvent,
                BurnFungibleResourceEvent
            ]
        };

        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::Outer,
            is_transient: false,
            feature_set: FungibleResourceManagerFeatureSet::all_features(),
            dependencies: btreeset!(),
            schema: BlueprintSchemaInit {
                generics: vec![],
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
                    },
                    methods {
                        FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT => [MINTER_ROLE];
                        RESOURCE_MANAGER_BURN_IDENT => [BURNER_ROLE];
                        RESOURCE_MANAGER_PACKAGE_BURN_IDENT => MethodAccessibility::OwnPackageOnly;
                        RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT => MethodAccessibility::Public;
                        RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT => MethodAccessibility::Public;
                        RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT => MethodAccessibility::Public;
                        RESOURCE_MANAGER_GET_AMOUNT_FOR_WITHDRAWAL_IDENT => MethodAccessibility::Public;
                        RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT => MethodAccessibility::Public;
                        RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT => MethodAccessibility::Public;
                    }
                }),
            },
        }
    }

    pub(crate) fn create<Y>(
        owner_role: OwnerRole,
        track_total_supply: bool,
        divisibility: u8,
        resource_roles: FungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        verify_divisibility(divisibility)?;

        let address_reservation = match address_reservation {
            Some(address_reservation) => address_reservation,
            None => {
                let (reservation, _) = api.allocate_global_address(BlueprintId {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                })?;
                reservation
            }
        };

        let (mut features, roles) = to_features_and_roles(resource_roles);
        features.track_total_supply = track_total_supply;

        let total_supply_field = if features.mint || features.burn {
            FieldValue::new(
                &FungibleResourceManagerTotalSupplyFieldPayload::from_content_source(
                    Decimal::zero(),
                ),
            )
        } else {
            FieldValue::immutable(
                &FungibleResourceManagerTotalSupplyFieldPayload::from_content_source(
                    Decimal::zero(),
                ),
            )
        };

        let object_id = api.new_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            features.feature_names_str(),
            GenericArgs::default(),
            vec![
                FieldValue::immutable(
                    &FungibleResourceManagerDivisibilityFieldPayload::from_content_source(
                        divisibility,
                    ),
                ),
                total_supply_field,
            ],
            btreemap!(),
        )?;

        let resource_address = globalize_resource_manager(
            owner_role,
            object_id,
            address_reservation,
            roles,
            metadata,
            api,
        )?;

        Ok(resource_address)
    }

    pub(crate) fn create_with_initial_supply<Y>(
        owner_role: OwnerRole,
        track_total_supply: bool,
        divisibility: u8,
        initial_supply: Decimal,
        resource_roles: FungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        verify_divisibility(divisibility)?;

        let address_reservation = match address_reservation {
            Some(address_reservation) => address_reservation,
            None => {
                let (reservation, _) = api.allocate_global_address(BlueprintId {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                })?;
                reservation
            }
        };

        let (mut features, roles) = to_features_and_roles(resource_roles);
        features.track_total_supply = track_total_supply;

        let total_supply_field = if features.mint || features.burn {
            FieldValue::new(
                &FungibleResourceManagerTotalSupplyFieldPayload::from_content_source(
                    initial_supply,
                ),
            )
        } else {
            FieldValue::immutable(
                &FungibleResourceManagerTotalSupplyFieldPayload::from_content_source(
                    initial_supply,
                ),
            )
        };

        let object_id = api.new_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            features.feature_names_str(),
            GenericArgs::default(),
            vec![
                FieldValue::immutable(
                    &FungibleResourceManagerDivisibilityFieldPayload::from_content_source(
                        divisibility,
                    ),
                ),
                total_supply_field,
            ],
            btreemap!(),
        )?;

        check_mint_amount(divisibility, initial_supply)?;

        let (resource_address, bucket) = globalize_fungible_with_initial_supply(
            owner_role,
            object_id,
            address_reservation,
            roles,
            metadata,
            initial_supply,
            api,
        )?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn mint<Y>(amount: Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_mintable(api)?;

        let divisibility = {
            let divisibility_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                FungibleResourceManagerField::Divisibility,
                LockFlags::read_only(),
            )?;
            let divisibility: FungibleResourceManagerDivisibilityFieldPayload =
                api.field_read_typed(divisibility_handle)?;
            divisibility.into_latest()
        };

        // check amount
        check_mint_amount(divisibility, amount)?;

        let bucket = Self::create_bucket(amount, api)?;

        Runtime::emit_event(api, MintFungibleResourceEvent { amount })?;

        // Update total supply
        // TODO: Could be further cleaned up by using event
        if api.actor_is_feature_enabled(
            OBJECT_HANDLE_SELF,
            FungibleResourceManagerFeature::TrackTotalSupply,
        )? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                FungibleResourceManagerField::TotalSupply,
                LockFlags::MUTABLE,
            )?;
            let mut total_supply = api
                .field_read_typed::<FungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .into_latest();
            total_supply = total_supply.safe_add(amount).unwrap();
            api.field_write_typed(
                total_supply_handle,
                &FungibleResourceManagerTotalSupplyFieldPayload::from_content_source(total_supply),
            )?;
            api.field_close(total_supply_handle)?;
        }

        Ok(bucket)
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

        // Drop other bucket
        let other_bucket = drop_fungible_bucket(bucket.0.as_node_id(), api)?;

        // Construct the event and only emit it once all of the operations are done.
        Runtime::emit_event(
            api,
            BurnFungibleResourceEvent {
                amount: other_bucket.liquid.amount(),
            },
        )?;

        // Update total supply
        // TODO: Could be further cleaned up by using event
        if api.actor_is_feature_enabled(
            OBJECT_HANDLE_SELF,
            FungibleResourceManagerFeature::TrackTotalSupply,
        )? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                FungibleResourceManagerField::TotalSupply,
                LockFlags::MUTABLE,
            )?;
            let mut total_supply = api
                .field_read_typed::<FungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .into_latest();
            total_supply = total_supply.safe_sub(other_bucket.liquid.amount()).unwrap();
            api.field_write_typed(
                total_supply_handle,
                &FungibleResourceManagerTotalSupplyFieldPayload::from_content_source(total_supply),
            )?;
            api.field_close(total_supply_handle)?;
        }

        Ok(())
    }

    pub(crate) fn drop_empty_bucket<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let other_bucket = drop_fungible_bucket(bucket.0.as_node_id(), api)?;

        if other_bucket.liquid.amount().is_zero() {
            Ok(())
        } else {
            Err(RuntimeError::ApplicationError(
                ApplicationError::FungibleResourceManagerError(
                    FungibleResourceManagerError::DropNonEmptyBucket,
                ),
            ))
        }
    }

    pub(crate) fn create_empty_bucket<Y>(api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::create_bucket(0.into(), api)
    }

    pub(crate) fn create_bucket<Y>(amount: Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let bucket_id = api.new_simple_object(
            FUNGIBLE_BUCKET_BLUEPRINT,
            vec![
                FieldValue::new(&LiquidFungibleResource::new(amount)),
                FieldValue::new(&LockedFungibleResource::default()),
            ],
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub(crate) fn create_empty_vault<Y>(api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let vault_id = api.new_simple_object(
            FUNGIBLE_VAULT_BLUEPRINT,
            vec![
                FieldValue::new(&FungibleVaultBalanceFieldPayload::from_content_source(
                    LiquidFungibleResource::default(),
                )),
                FieldValue::new(
                    &FungibleVaultLockedBalanceFieldPayload::from_content_source(
                        LockedFungibleResource::default(),
                    ),
                ),
                FieldValue::new(&FungibleVaultFreezeStatusFieldPayload::from_content_source(
                    VaultFrozenFlag::default(),
                )),
            ],
        )?;

        Runtime::emit_event(api, VaultCreationEvent { vault_id })?;

        Ok(Own(vault_id))
    }

    pub(crate) fn get_resource_type<Y>(api: &mut Y) -> Result<ResourceType, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let divisibility_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleResourceManagerField::Divisibility,
            LockFlags::read_only(),
        )?;

        let divisibility = api
            .field_read_typed::<FungibleResourceManagerDivisibilityFieldPayload>(
                divisibility_handle,
            )?
            .into_latest();
        let resource_type = ResourceType::Fungible { divisibility };

        Ok(resource_type)
    }

    pub(crate) fn get_total_supply<Y>(api: &mut Y) -> Result<Option<Decimal>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if api.actor_is_feature_enabled(
            OBJECT_HANDLE_SELF,
            FungibleResourceManagerFeature::TrackTotalSupply,
        )? {
            let total_supply_handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                FungibleResourceManagerField::TotalSupply,
                LockFlags::read_only(),
            )?;
            let total_supply = api
                .field_read_typed::<FungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .into_latest();
            Ok(Some(total_supply))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn amount_for_withdrawal<Y>(
        api: &mut Y,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
    ) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let divisibility_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleResourceManagerField::Divisibility,
            LockFlags::read_only(),
        )?;

        let divisibility = api
            .field_read_typed::<FungibleResourceManagerDivisibilityFieldPayload>(
                divisibility_handle,
            )?
            .into_latest();

        Ok(amount.for_withdrawal(divisibility, withdraw_strategy))
    }

    fn assert_mintable<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api
            .actor_is_feature_enabled(OBJECT_HANDLE_SELF, FungibleResourceManagerFeature::Mint)?
        {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::FungibleResourceManagerError(
                    FungibleResourceManagerError::NotMintable,
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
            .actor_is_feature_enabled(OBJECT_HANDLE_SELF, FungibleResourceManagerFeature::Burn)?
        {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::FungibleResourceManagerError(
                    FungibleResourceManagerError::NotBurnable,
                ),
            ));
        }

        return Ok(());
    }
}
