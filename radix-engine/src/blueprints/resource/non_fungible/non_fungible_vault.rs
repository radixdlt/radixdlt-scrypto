use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use radix_engine_interface::api::{
    FieldValue, LockFlags, SystemApi, ACTOR_STATE_OUTER_OBJECT, ACTOR_STATE_SELF,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use radix_native_sdk::resource::NativeBucket;
use radix_native_sdk::runtime::Runtime;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum NonFungibleVaultError {
    MissingId(NonFungibleLocalId),
    NotEnoughAmount,
    DecimalOverflow,
}

declare_native_blueprint_state! {
    blueprint_ident: NonFungibleVault,
    blueprint_snake_case: non_fungible_vault,
    fields: {
        balance: {
            ident: Balance,
            field_type: {
                kind: StaticSingleVersioned,
            },
        },
        locked_resource: {
            ident: LockedResource,
            field_type: {
                kind: StaticSingleVersioned,
            },
            transience: FieldTransience::TransientStatic {
                default_value: scrypto_encode(&NonFungibleVaultLockedResourceFieldPayload::from_content_source(LockedNonFungibleResource::default())).unwrap(),
            },
        },
        freeze_status: {
            ident: FreezeStatus,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::if_outer_feature(NonFungibleResourceManagerFeature::VaultFreeze),
        },
    },
    collections: {
        non_fungibles: Index {
            entry_ident: NonFungible,
            key_type: {
                kind: Static,
                content_type: NonFungibleLocalId,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
    },
}

type NonFungibleVaultBalanceV1 = LiquidNonFungibleVault;
type NonFungibleVaultLockedResourceV1 = LockedNonFungibleResource;
type NonFungibleVaultFreezeStatusV1 = VaultFrozenFlag;
type NonFungibleVaultNonFungibleV1 = ();

pub struct NonFungibleVaultBlueprint;

impl NonFungibleVaultBlueprint {
    pub fn get_definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let state = NonFungibleVaultStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();
        functions.insert(
            VAULT_TAKE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultTakeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultTakeOutput>(),
                ),
                export: NON_FUNGIBLE_VAULT_TAKE_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            VAULT_TAKE_ADVANCED_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultTakeAdvancedInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultTakeAdvancedOutput>(),
                ),
                export: NON_FUNGIBLE_VAULT_TAKE_ADVANCED_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultTakeNonFungiblesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultTakeNonFungiblesOutput>(),
                ),
                export: NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT.to_string(),
            },
        );
        functions.insert(
            VAULT_RECALL_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo {
                    receiver: Receiver::SelfRefMut,
                    ref_types: RefTypes::DIRECT_ACCESS,
                }),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultRecallInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultRecallOutput>(),
                ),
                export: NON_FUNGIBLE_VAULT_RECALL_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            VAULT_FREEZE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo {
                    receiver: Receiver::SelfRefMut,
                    ref_types: RefTypes::DIRECT_ACCESS,
                }),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultFreezeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultFreezeOutput>(),
                ),
                export: NON_FUNGIBLE_VAULT_FREEZE_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            VAULT_UNFREEZE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo {
                    receiver: Receiver::SelfRefMut,
                    ref_types: RefTypes::DIRECT_ACCESS,
                }),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultUnfreezeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultUnfreezeOutput>(),
                ),
                export: NON_FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo {
                    receiver: Receiver::SelfRefMut,
                    ref_types: RefTypes::DIRECT_ACCESS,
                }),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultRecallNonFungiblesInput>(
                        ),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultRecallNonFungiblesOutput>(
                        ),
                ),
                export: NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT.to_string(),
            },
        );
        functions.insert(
            VAULT_PUT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultPutInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultPutOutput>(),
                ),
                export: NON_FUNGIBLE_VAULT_PUT_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            VAULT_GET_AMOUNT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultGetAmountInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultGetAmountOutput>(),
                ),
                export: NON_FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleVaultGetNonFungibleLocalIdsInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleVaultGetNonFungibleLocalIdsOutput>()),
                export: NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_VAULT_CONTAINS_NON_FUNGIBLE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleVaultContainsNonFungibleInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleVaultContainsNonFungibleOutput>()),
                export: NON_FUNGIBLE_VAULT_CONTAINS_NON_FUNGIBLE_IDENT.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleVaultCreateProofOfNonFungiblesInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<NonFungibleVaultCreateProofOfNonFungiblesOutput>()),
                export: NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultLockNonFungiblesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultLockNonFungiblesOutput>(),
                ),
                export: NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultUnlockNonFungiblesInput>(
                        ),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultUnlockNonFungiblesOutput>(
                        ),
                ),
                export: NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            VAULT_BURN_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultBurnInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<VaultBurnOutput>(),
                ),
                export: NON_FUNGIBLE_VAULT_BURN_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultBurnNonFungiblesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultBurnNonFungiblesOutput>(),
                ),
                export: NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                non_fungible_vault::WithdrawEvent,
                non_fungible_vault::DepositEvent,
                non_fungible_vault::RecallEvent
            ]
        };

        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::Inner {
                outer_blueprint: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            },
            is_transient: false,
            dependencies: indexset!(),
            feature_set: NonFungibleVaultFeatureSet::all_features(),

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
                method_auth: MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
                    roles: RoleSpecification::UseOuter,
                    methods: method_auth_template! {
                        VAULT_GET_AMOUNT_IDENT => MethodAccessibility::Public;
                        NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => MethodAccessibility::Public;
                        NON_FUNGIBLE_VAULT_CONTAINS_NON_FUNGIBLE_IDENT => MethodAccessibility::Public;
                        NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => MethodAccessibility::Public;

                        VAULT_TAKE_IDENT => [WITHDRAWER_ROLE];
                        VAULT_TAKE_ADVANCED_IDENT => [WITHDRAWER_ROLE];
                        NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT => [WITHDRAWER_ROLE];
                        VAULT_RECALL_IDENT => [RECALLER_ROLE];
                        VAULT_FREEZE_IDENT => [FREEZER_ROLE];
                        VAULT_UNFREEZE_IDENT => [FREEZER_ROLE];
                        NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT => [RECALLER_ROLE];
                        VAULT_PUT_IDENT => [DEPOSITOR_ROLE];
                        VAULT_BURN_IDENT => [BURNER_ROLE];
                        NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT => [BURNER_ROLE];

                        NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT => MethodAccessibility::OwnPackageOnly;
                        NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT => MethodAccessibility::OwnPackageOnly;
                    },
                }),
            },
        }
    }

    pub fn take<Y: SystemApi<RuntimeError>>(
        amount: &Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::take_advanced(amount, WithdrawStrategy::Exact, api)
    }

    pub fn take_advanced<Y: SystemApi<RuntimeError>>(
        amount: &Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::assert_not_frozen(VaultFreezeFlags::WITHDRAW, api)?;

        let taken = {
            let amount = amount.for_withdrawal(0, withdraw_strategy).ok_or(
                RuntimeError::ApplicationError(ApplicationError::NonFungibleVaultError(
                    NonFungibleVaultError::DecimalOverflow,
                )),
            )?;

            let n = check_non_fungible_amount(&amount).map_err(|_| {
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::InvalidAmount(amount),
                ))
            })?;

            Self::internal_take_by_amount(n, api)?
        };

        // Create node
        let ids = taken.into_ids();
        let bucket = NonFungibleResourceManagerBlueprint::create_bucket(ids.clone(), api)?;

        Runtime::emit_event(api, events::non_fungible_vault::WithdrawEvent { ids })?;

        Ok(bucket)
    }

    pub fn take_non_fungibles<Y: SystemApi<RuntimeError>>(
        non_fungible_local_ids: &IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::assert_not_frozen(VaultFreezeFlags::WITHDRAW, api)?;

        // Take
        let taken = Self::internal_take_non_fungibles(non_fungible_local_ids, api)?;

        // Create node
        let ids = taken.into_ids();
        let bucket = NonFungibleResourceManagerBlueprint::create_bucket(ids.clone(), api)?;

        Runtime::emit_event(api, events::non_fungible_vault::WithdrawEvent { ids })?;

        Ok(bucket)
    }

    pub fn put<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::assert_not_frozen(VaultFreezeFlags::DEPOSIT, api)?;

        // Drop other bucket
        // This will fail if bucket is not an inner object of the current non-fungible resource
        let other_bucket = drop_non_fungible_bucket(bucket.0.as_node_id(), api)?;
        let ids = other_bucket.liquid.ids().clone();

        // Put
        Self::internal_put(other_bucket.liquid, api)?;

        Runtime::emit_event(api, events::non_fungible_vault::DepositEvent { ids })?;

        Ok(())
    }

    pub fn get_amount<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Decimal, RuntimeError> {
        Self::liquid_amount(api)?
            .checked_add(Self::locked_amount(api)?)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::DecimalOverflow),
            ))
    }

    pub fn contains_non_fungible<Y: SystemApi<RuntimeError>>(
        id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        let ids = Self::locked_non_fungible_local_ids(u32::MAX, api)?;
        if ids.contains(&id) {
            return Ok(true);
        }

        // TODO: Replace with better index api
        let key = scrypto_encode(&id).unwrap();
        let removed = api.actor_index_remove(
            ACTOR_STATE_SELF,
            NonFungibleVaultCollection::NonFungibleIndex.collection_index(),
            key.clone(),
        )?;
        let exists = removed.is_some();
        if let Some(removed) = removed {
            api.actor_index_insert(
                ACTOR_STATE_SELF,
                NonFungibleVaultCollection::NonFungibleIndex.collection_index(),
                key,
                removed,
            )?;
        }

        Ok(exists)
    }

    pub fn get_non_fungible_local_ids<Y: SystemApi<RuntimeError>>(
        limit: u32,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, RuntimeError> {
        let mut ids = Self::locked_non_fungible_local_ids(limit, api)?;
        let id_len: u32 = ids.len().try_into().unwrap();

        if id_len < limit {
            let locked_count = limit - id_len;
            ids.extend(Self::liquid_non_fungible_local_ids(locked_count, api)?);
        }

        Ok(ids)
    }

    pub fn recall<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::assert_recallable(api)?;

        let n = check_non_fungible_amount(&amount).map_err(|_| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::InvalidAmount(
                amount,
            )))
        })?;

        let taken = Self::internal_take_by_amount(n, api)?;

        let ids = taken.into_ids();
        let bucket = NonFungibleResourceManagerBlueprint::create_bucket(ids.clone(), api)?;

        Runtime::emit_event(api, events::non_fungible_vault::RecallEvent { ids })?;

        Ok(bucket)
    }

    pub fn freeze<Y: SystemApi<RuntimeError>>(
        to_freeze: VaultFreezeFlags,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::assert_freezable(api)?;

        let frozen_flag_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleVaultField::FreezeStatus.into(),
            LockFlags::MUTABLE,
        )?;

        let mut frozen = api
            .field_read_typed::<NonFungibleVaultFreezeStatusFieldPayload>(frozen_flag_handle)?
            .fully_update_and_into_latest_version();
        frozen.frozen.insert(to_freeze);
        api.field_write_typed(
            frozen_flag_handle,
            &NonFungibleVaultFreezeStatusFieldPayload::from_content_source(frozen),
        )?;

        Ok(())
    }

    pub fn unfreeze<Y: SystemApi<RuntimeError>>(
        to_unfreeze: VaultFreezeFlags,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::assert_freezable(api)?;

        let frozen_flag_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleVaultField::FreezeStatus.into(),
            LockFlags::MUTABLE,
        )?;
        let mut frozen = api
            .field_read_typed::<NonFungibleVaultFreezeStatusFieldPayload>(frozen_flag_handle)?
            .fully_update_and_into_latest_version();
        frozen.frozen.remove(to_unfreeze);
        api.field_write_typed(
            frozen_flag_handle,
            &NonFungibleVaultFreezeStatusFieldPayload::from_content_source(frozen),
        )?;

        Ok(())
    }

    pub fn recall_non_fungibles<Y: SystemApi<RuntimeError>>(
        non_fungible_local_ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::assert_recallable(api)?;

        let taken = Self::internal_take_non_fungibles(&non_fungible_local_ids, api)?;

        let ids = taken.into_ids();
        let bucket = NonFungibleResourceManagerBlueprint::create_bucket(ids.clone(), api)?;

        Runtime::emit_event(api, events::non_fungible_vault::RecallEvent { ids })?;

        Ok(bucket)
    }

    pub fn create_proof_of_non_fungibles<Y: SystemApi<RuntimeError>>(
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError> {
        Self::lock_non_fungibles(&ids, api)?;

        let proof_info = ProofMoveableSubstate { restricted: false };
        let receiver = Runtime::get_node_id(api)?;
        let proof_evidence = NonFungibleProofSubstate::new(
            ids.clone(),
            indexmap!(
                LocalRef::Vault(Reference(receiver.into()))=> ids
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ProofError(e)))
        })?;
        let proof_id = api.new_simple_object(
            NON_FUNGIBLE_PROOF_BLUEPRINT,
            indexmap! {
                NonFungibleProofField::Moveable.field_index() => FieldValue::new(&proof_info),
                NonFungibleProofField::ProofRefs.field_index() => FieldValue::new(&proof_evidence),
            },
        )?;
        Ok(Proof(Own(proof_id)))
    }

    pub fn burn<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::assert_not_frozen(VaultFreezeFlags::BURN, api)?;

        Self::take(&amount, api)?.package_burn(api)?;
        Ok(())
    }

    pub fn burn_non_fungibles<Y: SystemApi<RuntimeError>>(
        non_fungible_local_ids: &IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::assert_not_frozen(VaultFreezeFlags::BURN, api)?;

        Self::take_non_fungibles(non_fungible_local_ids, api)?.package_burn(api)?;
        Ok(())
    }

    //===================
    // Protected methods
    //===================

    pub fn lock_non_fungibles<Y: SystemApi<RuntimeError>>(
        ids: &IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleVaultField::LockedResource.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked = api
            .field_read_typed::<NonFungibleVaultLockedResourceFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        // Take from liquid if needed
        let delta: IndexSet<NonFungibleLocalId> = ids
            .iter()
            .cloned()
            .filter(|id| !locked.ids.contains_key(id))
            .collect();
        Self::internal_take_non_fungibles(&delta, api)?;

        // Increase lock count
        for id in ids {
            locked.ids.entry(id.clone()).or_default().add_assign(1);
        }

        api.field_write_typed(
            handle,
            &NonFungibleVaultLockedResourceFieldPayload::from_content_source(locked),
        )?;

        // Issue proof
        Ok(())
    }

    pub fn unlock_non_fungibles<Y: SystemApi<RuntimeError>>(
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleVaultField::LockedResource.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked = api
            .field_read_typed::<NonFungibleVaultLockedResourceFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        let mut liquid_non_fungibles: IndexSet<NonFungibleLocalId> = index_set_new();
        for id in ids {
            let cnt = locked
                .ids
                .swap_remove(&id)
                .expect("Attempted to unlock non-fungible that was not locked");
            if cnt > 1 {
                locked.ids.insert(id, cnt - 1);
            } else {
                liquid_non_fungibles.insert(id);
            }
        }

        api.field_write_typed(
            handle,
            &NonFungibleVaultLockedResourceFieldPayload::from_content_source(locked),
        )?;

        Self::internal_put(LiquidNonFungibleResource::new(liquid_non_fungibles), api)
    }

    //===================
    // Helper methods
    //===================

    fn assert_not_frozen<Y: SystemApi<RuntimeError>>(
        flags: VaultFreezeFlags,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if !api.actor_is_feature_enabled(
            ACTOR_STATE_OUTER_OBJECT,
            NonFungibleResourceManagerFeature::VaultFreeze.feature_name(),
        )? {
            return Ok(());
        }

        let frozen_flag_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleVaultField::FreezeStatus.into(),
            LockFlags::read_only(),
        )?;
        let frozen = api
            .field_read_typed::<NonFungibleVaultFreezeStatusFieldPayload>(frozen_flag_handle)?
            .fully_update_and_into_latest_version();
        api.field_close(frozen_flag_handle)?;

        if frozen.frozen.intersects(flags) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::VaultIsFrozen),
            ));
        }

        Ok(())
    }

    fn assert_freezable<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        if !api.actor_is_feature_enabled(
            ACTOR_STATE_OUTER_OBJECT,
            NonFungibleResourceManagerFeature::VaultFreeze.feature_name(),
        )? {
            // This should never be hit since the auth layer will prevent
            // any freeze call from even getting to this point but this is useful
            // if the Auth layer is ever disabled for whatever reason.
            // We still want to maintain these invariants.
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NotFreezable),
            ));
        }

        Ok(())
    }

    fn assert_recallable<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        if !api.actor_is_feature_enabled(
            ACTOR_STATE_OUTER_OBJECT,
            NonFungibleResourceManagerFeature::VaultRecall.feature_name(),
        )? {
            // This should never be hit since the auth layer will prevent
            // any recall call from even getting to this point but this is useful
            // if the Auth layer is ever disabled for whatever reason.
            // We still want to maintain these invariants.
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NotRecallable),
            ));
        }

        Ok(())
    }

    fn liquid_amount<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Decimal, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleVaultField::Balance.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref = api
            .field_read_typed::<NonFungibleVaultBalanceFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        let amount = substate_ref.amount;
        api.field_close(handle)?;
        Ok(amount)
    }

    fn locked_amount<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Decimal, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleVaultField::LockedResource.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref = api
            .field_read_typed::<NonFungibleVaultLockedResourceFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        let amount = substate_ref.amount();
        api.field_close(handle)?;
        Ok(amount)
    }

    fn liquid_non_fungible_local_ids<Y: SystemApi<RuntimeError>>(
        limit: u32,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, RuntimeError> {
        let items: Vec<NonFungibleLocalId> = api.actor_index_scan_keys_typed(
            ACTOR_STATE_SELF,
            NonFungibleVaultCollection::NonFungibleIndex.collection_index(),
            limit,
        )?;
        let ids = items.into_iter().collect();
        Ok(ids)
    }

    fn locked_non_fungible_local_ids<Y: SystemApi<RuntimeError>>(
        limit: u32,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleVaultField::LockedResource.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref = api
            .field_read_typed::<NonFungibleVaultLockedResourceFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        let limit: usize = limit.try_into().unwrap();
        let ids = substate_ref.ids().into_iter().take(limit).collect();
        api.field_close(handle)?;
        Ok(ids)
    }

    fn internal_take_by_amount<Y: SystemApi<RuntimeError>>(
        n: u32,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError> {
        // deduct from liquidity pool
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleVaultField::Balance.into(),
            LockFlags::MUTABLE,
        )?;
        let mut balance = api
            .field_read_typed::<NonFungibleVaultBalanceFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        if balance.amount < Decimal::from(n) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::NonFungibleVaultError(NonFungibleVaultError::NotEnoughAmount),
            ));
        }
        balance.amount = balance
            .amount
            .checked_sub(n)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::DecimalOverflow),
            ))?;

        let taken = {
            let ids: Vec<(NonFungibleLocalId, NonFungibleVaultNonFungibleEntryPayload)> = api
                .actor_index_drain_typed(
                    ACTOR_STATE_SELF,
                    NonFungibleVaultCollection::NonFungibleIndex.collection_index(),
                    n,
                )?;
            LiquidNonFungibleResource {
                ids: ids.into_iter().map(|(key, _value)| key).collect(),
            }
        };

        api.field_write_typed(
            handle,
            &NonFungibleVaultBalanceFieldPayload::from_content_source(balance),
        )?;
        api.field_close(handle)?;

        Ok(taken)
    }

    pub fn internal_take_non_fungibles<Y: SystemApi<RuntimeError>>(
        ids: &IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<LiquidNonFungibleResource, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleVaultField::Balance.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api
            .field_read_typed::<NonFungibleVaultBalanceFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        substate_ref.amount =
            substate_ref
                .amount
                .checked_sub(ids.len())
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::VaultError(VaultError::DecimalOverflow),
                ))?;

        // TODO: Batch remove
        for id in ids {
            let removed = api.actor_index_remove(
                ACTOR_STATE_SELF,
                NonFungibleVaultCollection::NonFungibleIndex.collection_index(),
                scrypto_encode(id).unwrap(),
            )?;

            if removed.is_none() {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::NonFungibleVaultError(NonFungibleVaultError::MissingId(
                        id.clone(),
                    )),
                ));
            }
        }

        api.field_write_typed(
            handle,
            &NonFungibleVaultBalanceFieldPayload::from_content_source(substate_ref),
        )?;
        api.field_close(handle)?;

        Ok(LiquidNonFungibleResource::new(ids.clone()))
    }

    pub fn internal_put<Y: SystemApi<RuntimeError>>(
        resource: LiquidNonFungibleResource,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if resource.is_empty() {
            return Ok(());
        }

        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            NonFungibleVaultField::Balance.into(),
            LockFlags::MUTABLE,
        )?;
        let mut vault = api
            .field_read_typed::<NonFungibleVaultBalanceFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        vault.amount =
            vault
                .amount
                .checked_add(resource.ids.len())
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::VaultError(VaultError::DecimalOverflow),
                ))?;

        // update liquidity
        // TODO: Batch update
        // TODO: Rather than insert, use create_unique?
        for id in resource.ids {
            api.actor_index_insert_typed(
                ACTOR_STATE_SELF,
                NonFungibleVaultCollection::NonFungibleIndex.collection_index(),
                id,
                NonFungibleVaultNonFungibleEntryPayload::from_content_source(()),
            )?;
        }

        api.field_write_typed(
            handle,
            &NonFungibleVaultBalanceFieldPayload::from_content_source(vault),
        )?;
        api.field_close(handle)?;

        Ok(())
    }
}
