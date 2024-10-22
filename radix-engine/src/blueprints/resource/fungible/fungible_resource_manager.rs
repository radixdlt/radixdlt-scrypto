use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use lazy_static::lazy_static;
use num_traits::pow::Pow;
use radix_common::math::Decimal;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::{FieldValue, GenericArgs, SystemApi, ACTOR_STATE_SELF};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::MetadataInit;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_native_sdk::component::{globalize_object, globalize_object_with_inner_object_and_event};
use radix_native_sdk::runtime::Runtime;

const DIVISIBILITY_MAXIMUM: u8 = 18;

lazy_static! {
    static ref MAX_MINT_AMOUNT: Decimal = Decimal::from_attos(I192::from(2).pow(152)); // 2^152 subunits
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
    InvalidAmount(Decimal, u8),
    MaxMintAmountExceeded,
    InvalidDivisibility(u8),
    DropNonEmptyBucket,
    NotMintable,
    NotBurnable,
    UnexpectedDecimalComputationError,
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
    track_total_supply: bool,
    role_init: FungibleResourceRoles,
) -> (FungibleResourceManagerFeatureSet, RoleAssignmentInit) {
    let mut roles = RoleAssignmentInit::new();

    let features = FungibleResourceManagerFeatureSet {
        track_total_supply,
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

        let mut functions = index_map_new();
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
            dependencies: indexset!(),
            schema: BlueprintSchemaInit {
                generics: vec![],
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

    pub(crate) fn create<Y: SystemApi<RuntimeError>>(
        owner_role: OwnerRole,
        track_total_supply: bool,
        divisibility: u8,
        resource_roles: FungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError> {
        let (object_id, roles) = Self::create_object(
            Decimal::ZERO,
            track_total_supply,
            divisibility,
            resource_roles,
            api,
        )?;
        let address_reservation = Self::create_address_reservation(address_reservation, api)?;

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
        track_total_supply: bool,
        divisibility: u8,
        initial_supply: Decimal,
        resource_roles: FungibleResourceRoles,
        metadata: ModuleConfig<MetadataInit>,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError> {
        let (object_id, roles) = Self::create_object(
            initial_supply,
            track_total_supply,
            divisibility,
            resource_roles,
            api,
        )?;
        let address_reservation = Self::create_address_reservation(address_reservation, api)?;

        check_mint_amount(divisibility, initial_supply)?;

        let (resource_address, bucket) = {
            let (address, inner_object) = globalize_object_with_inner_object_and_event(
                object_id,
                owner_role,
                address_reservation,
                roles,
                metadata,
                FUNGIBLE_BUCKET_BLUEPRINT,
                indexmap! {
                    FungibleBucketField::Liquid.field_index() => FieldValue::new(&LiquidFungibleResource::new(initial_supply)),
                    FungibleBucketField::Locked.field_index() => FieldValue::new(&LockedFungibleResource::default()),
                },
                MintFungibleResourceEvent::EVENT_NAME,
                MintFungibleResourceEvent {
                    amount: initial_supply,
                },
                api,
            )?;

            (
                ResourceAddress::new_or_panic(address.into()),
                Bucket(Own(inner_object)),
            )
        };

        Ok((resource_address, bucket))
    }

    fn create_address_reservation<Y: SystemApi<RuntimeError>>(
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<GlobalAddressReservation, RuntimeError> {
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

        Ok(address_reservation)
    }

    fn create_object<Y: SystemApi<RuntimeError>>(
        initial_supply: Decimal,
        track_total_supply: bool,
        divisibility: u8,
        resource_roles: FungibleResourceRoles,
        api: &mut Y,
    ) -> Result<(NodeId, RoleAssignmentInit), RuntimeError> {
        verify_divisibility(divisibility)?;

        let mut fields = indexmap! {
            FungibleResourceManagerField::Divisibility.into() => FieldValue::immutable(
                    &FungibleResourceManagerDivisibilityFieldPayload::from_content_source(
                        divisibility,
                    ),
                )
        };

        let (features, roles) = to_features_and_roles(track_total_supply, resource_roles);

        if features.track_total_supply {
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

            fields.insert(
                FungibleResourceManagerField::TotalSupply.into(),
                total_supply_field,
            );
        }

        let object_id = api.new_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            features.feature_names_str(),
            GenericArgs::default(),
            fields,
            indexmap!(),
        )?;

        Ok((object_id, roles))
    }

    pub(crate) fn mint<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::assert_mintable(api)?;

        let divisibility = {
            let divisibility_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                FungibleResourceManagerField::Divisibility.into(),
                LockFlags::read_only(),
            )?;
            let divisibility: FungibleResourceManagerDivisibilityFieldPayload =
                api.field_read_typed(divisibility_handle)?;
            divisibility.fully_update_and_into_latest_version()
        };

        // check amount
        check_mint_amount(divisibility, amount)?;

        let bucket = Self::create_bucket(amount, api)?;

        Runtime::emit_event(api, MintFungibleResourceEvent { amount })?;

        // Update total supply
        if api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            FungibleResourceManagerFeature::TrackTotalSupply.feature_name(),
        )? {
            let total_supply_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                FungibleResourceManagerField::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply = api
                .field_read_typed::<FungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .fully_update_and_into_latest_version();
            // This should never overflow due to the 2^152 limit we place on mints.
            // Since Decimal have 2^192 max we would need to mint 2^40 times before
            // an overflow occurs.
            total_supply =
                total_supply
                    .checked_add(amount)
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::FungibleResourceManagerError(
                            FungibleResourceManagerError::UnexpectedDecimalComputationError,
                        ),
                    ))?;
            api.field_write_typed(
                total_supply_handle,
                &FungibleResourceManagerTotalSupplyFieldPayload::from_content_source(total_supply),
            )?;
            api.field_close(total_supply_handle)?;
        }

        Ok(bucket)
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

        // Drop other bucket
        // This will fail if bucket is not an inner object of the current fungible resource
        let other_bucket = drop_fungible_bucket(bucket.0.as_node_id(), api)?;

        // Construct the event and only emit it once all of the operations are done.
        Runtime::emit_event(
            api,
            BurnFungibleResourceEvent {
                amount: other_bucket.liquid.amount(),
            },
        )?;

        // Update total supply
        if api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            FungibleResourceManagerFeature::TrackTotalSupply.feature_name(),
        )? {
            let total_supply_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                FungibleResourceManagerField::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply = api
                .field_read_typed::<FungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .fully_update_and_into_latest_version();
            total_supply = total_supply
                .checked_sub(other_bucket.liquid.amount())
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::FungibleResourceManagerError(
                        FungibleResourceManagerError::UnexpectedDecimalComputationError,
                    ),
                ))?;
            api.field_write_typed(
                total_supply_handle,
                &FungibleResourceManagerTotalSupplyFieldPayload::from_content_source(total_supply),
            )?;
            api.field_close(total_supply_handle)?;
        }

        Ok(())
    }

    pub(crate) fn drop_empty_bucket<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
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

    pub(crate) fn create_empty_bucket<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::create_bucket(0.into(), api)
    }

    pub(crate) fn create_bucket<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let bucket_id = api.new_simple_object(
            FUNGIBLE_BUCKET_BLUEPRINT,
            indexmap! {
                FungibleBucketField::Liquid.into() => FieldValue::new(&LiquidFungibleResource::new(amount)),
                FungibleBucketField::Locked.into() => FieldValue::new(&LockedFungibleResource::default()),
            },
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub(crate) fn create_empty_vault<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Own, RuntimeError> {
        let mut fields: IndexMap<FieldIndex, FieldValue> = indexmap! {
            FungibleVaultField::Balance.into() => FieldValue::new(&FungibleVaultBalanceFieldPayload::from_content_source(
                    LiquidFungibleResource::default(),
                )),
            FungibleVaultField::LockedBalance.into() => FieldValue::new(&FungibleVaultLockedBalanceFieldPayload::from_content_source(LockedFungibleResource::default())),
        };

        if api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            FungibleResourceManagerFeature::VaultFreeze.feature_name(),
        )? {
            fields.insert(
                FungibleVaultField::FreezeStatus.into(),
                FieldValue::new(&FungibleVaultFreezeStatusFieldPayload::from_content_source(
                    VaultFrozenFlag::default(),
                )),
            );
        }

        let vault_id = api.new_simple_object(FUNGIBLE_VAULT_BLUEPRINT, fields)?;

        Runtime::emit_event(api, VaultCreationEvent { vault_id })?;

        Ok(Own(vault_id))
    }

    pub(crate) fn get_resource_type<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<ResourceType, RuntimeError> {
        let divisibility_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleResourceManagerField::Divisibility.into(),
            LockFlags::read_only(),
        )?;

        let divisibility = api
            .field_read_typed::<FungibleResourceManagerDivisibilityFieldPayload>(
                divisibility_handle,
            )?
            .fully_update_and_into_latest_version();
        let resource_type = ResourceType::Fungible { divisibility };

        Ok(resource_type)
    }

    pub(crate) fn get_total_supply<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Option<Decimal>, RuntimeError> {
        if api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            FungibleResourceManagerFeature::TrackTotalSupply.feature_name(),
        )? {
            let total_supply_handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                FungibleResourceManagerField::TotalSupply.into(),
                LockFlags::read_only(),
            )?;
            let total_supply = api
                .field_read_typed::<FungibleResourceManagerTotalSupplyFieldPayload>(
                    total_supply_handle,
                )?
                .fully_update_and_into_latest_version();
            Ok(Some(total_supply))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn amount_for_withdrawal<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
    ) -> Result<Decimal, RuntimeError> {
        let divisibility_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleResourceManagerField::Divisibility.into(),
            LockFlags::read_only(),
        )?;

        let divisibility = api
            .field_read_typed::<FungibleResourceManagerDivisibilityFieldPayload>(
                divisibility_handle,
            )?
            .fully_update_and_into_latest_version();

        Ok(amount
            .for_withdrawal(divisibility, withdraw_strategy)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::FungibleResourceManagerError(
                    FungibleResourceManagerError::UnexpectedDecimalComputationError,
                ),
            ))?)
    }

    fn assert_mintable<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        if !api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            FungibleResourceManagerFeature::Mint.feature_name(),
        )? {
            // This should never be hit since the auth layer will prevent
            // any mint call from even getting to this point but this is useful
            // if the Auth layer is ever disabled for whatever reason.
            // We still want to maintain these invariants.
            return Err(RuntimeError::ApplicationError(
                ApplicationError::FungibleResourceManagerError(
                    FungibleResourceManagerError::NotMintable,
                ),
            ));
        }

        return Ok(());
    }

    fn assert_burnable<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        if !api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            FungibleResourceManagerFeature::Burn.feature_name(),
        )? {
            // This should never be hit since the auth layer will prevent
            // any burn call from even getting to this point but this is useful
            // if the Auth layer is ever disabled for whatever reason.
            // We still want to maintain these invariants.
            return Err(RuntimeError::ApplicationError(
                ApplicationError::FungibleResourceManagerError(
                    FungibleResourceManagerError::NotBurnable,
                ),
            ));
        }

        return Ok(());
    }
}
