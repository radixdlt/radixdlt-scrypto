use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::resource::NativeBucket;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::{
    ClientApi, FieldValue, ACTOR_REF_OUTER, ACTOR_STATE_OUTER_OBJECT, ACTOR_STATE_SELF,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;

declare_native_blueprint_state! {
    blueprint_ident: FungibleVault,
    blueprint_snake_case: fungible_vault,
    fields: {
        balance: {
            ident: Balance,
            field_type: {
                kind: StaticSingleVersioned,
            },
        },
        locked_balance: {
            ident: LockedBalance,
            field_type: {
                kind: StaticSingleVersioned,
            },
            transience: FieldTransience::TransientStatic {
                default_value: scrypto_encode(&FungibleVaultLockedBalanceFieldPayload::from_content_source(LockedFungibleResource::default())).unwrap(),
            },
        },
        freeze_status: {
            ident: FreezeStatus,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::if_outer_feature(FungibleResourceManagerFeature::VaultFreeze),
        },
    },
    collections: {}
}

type FungibleVaultBalanceV1 = LiquidFungibleResource;
type FungibleVaultLockedBalanceV1 = LockedFungibleResource;
type FungibleVaultFreezeStatusV1 = VaultFrozenFlag;

pub struct FungibleVaultBlueprint;

impl FungibleVaultBlueprint {
    pub fn get_definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let state = FungibleVaultStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = BTreeMap::new();
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
                export: FUNGIBLE_VAULT_TAKE_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_VAULT_TAKE_ADVANCED_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_VAULT_PUT_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            FUNGIBLE_VAULT_LOCK_FEE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<FungibleVaultLockFeeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<FungibleVaultLockFeeOutput>(),
                ),
                export: FUNGIBLE_VAULT_LOCK_FEE_IDENT.to_string(),
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
                export: FUNGIBLE_VAULT_RECALL_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_VAULT_FREEZE_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<FungibleVaultCreateProofOfAmountInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<FungibleVaultCreateProofOfAmountOutput>(),
                ),
                export: FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<FungibleVaultLockFungibleAmountInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<FungibleVaultLockFungibleAmountOutput>(),
                ),
                export: FUNGIBLE_VAULT_LOCK_AMOUNT_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<FungibleVaultUnlockFungibleAmountInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<FungibleVaultUnlockFungibleAmountOutput>(
                        ),
                ),
                export: FUNGIBLE_VAULT_UNLOCK_AMOUNT_EXPORT_NAME.to_string(),
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
                export: FUNGIBLE_VAULT_BURN_EXPORT_NAME.to_string(),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                fungible_vault::LockFeeEvent,
                fungible_vault::PayFeeEvent,
                fungible_vault::WithdrawEvent,
                fungible_vault::DepositEvent,
                fungible_vault::RecallEvent
            ]
        };

        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::Inner {
                outer_blueprint: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            },
            is_transient: false,
            dependencies: btreeset!(),
            feature_set: FungibleVaultFeatureSet::all_features(),
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
                method_auth: MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
                    roles: RoleSpecification::UseOuter,
                    methods: method_auth_template! {
                        VAULT_GET_AMOUNT_IDENT => MethodAccessibility::Public;
                        FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_IDENT => MethodAccessibility::Public;
                        VAULT_FREEZE_IDENT => [FREEZER_ROLE];
                        VAULT_UNFREEZE_IDENT => [FREEZER_ROLE];
                        VAULT_TAKE_IDENT => [WITHDRAWER_ROLE];
                        VAULT_TAKE_ADVANCED_IDENT => [WITHDRAWER_ROLE];
                        FUNGIBLE_VAULT_LOCK_FEE_IDENT => [WITHDRAWER_ROLE];
                        VAULT_RECALL_IDENT => [RECALLER_ROLE];
                        VAULT_PUT_IDENT => [DEPOSITOR_ROLE];
                        VAULT_BURN_IDENT => [BURNER_ROLE];
                        FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT => MethodAccessibility::OwnPackageOnly;
                        FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT => MethodAccessibility::OwnPackageOnly;
                    },
                }),
            },
        }
    }

    fn get_divisibility<Y>(api: &mut Y) -> Result<u8, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_OUTER_OBJECT,
            FungibleResourceManagerField::Divisibility.into(),
            LockFlags::read_only(),
        )?;
        let divisibility = api
            .field_read_typed::<FungibleResourceManagerDivisibilityFieldPayload>(handle)?
            .into_latest();
        api.field_close(handle)?;
        Ok(divisibility)
    }

    pub fn take<Y>(amount: &Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::take_advanced(amount, WithdrawStrategy::Exact, api)
    }

    pub fn take_advanced<Y>(
        amount: &Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::WITHDRAW, api)?;

        // Apply withdraw strategy
        let divisibility = Self::get_divisibility(api)?;
        let amount = amount.for_withdrawal(divisibility, withdraw_strategy);

        // Check amount
        if !check_fungible_amount(&amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        // Take
        let taken = Self::internal_take(amount, api)?;

        // Create node
        let bucket = FungibleResourceManagerBlueprint::create_bucket(taken.amount(), api)?;

        Runtime::emit_event(
            api,
            events::fungible_vault::WithdrawEvent {
                amount: taken.amount(),
            },
        )?;

        Ok(bucket)
    }

    pub fn put<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::DEPOSIT, api)?;

        // Drop other bucket
        let other_bucket = drop_fungible_bucket(bucket.0.as_node_id(), api)?;
        let amount = other_bucket.liquid.amount();

        // Put
        Self::internal_put(other_bucket.liquid, api)?;

        Runtime::emit_event(api, events::fungible_vault::DepositEvent { amount })?;

        Ok(())
    }

    pub fn get_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let amount = Self::liquid_amount(api)?
            .safe_add(Self::locked_amount(api)?)
            .unwrap();

        Ok(amount)
    }

    pub fn lock_fee<Y>(
        receiver: &NodeId,
        amount: Decimal,
        contingent: bool,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::WITHDRAW, api)?;

        // Check resource address and amount
        let resource_address =
            ResourceAddress::new_or_panic(api.actor_get_node_id(ACTOR_REF_OUTER)?.into());
        if resource_address != XRD {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::LockFeeNotRadixToken),
            ));
        }

        let divisibility = Self::get_divisibility(api)?;
        if !check_fungible_amount(&amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        // Lock the substate (with special flags)
        let vault_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleVaultField::Balance.into(),
            LockFlags::MUTABLE | LockFlags::UNMODIFIED_BASE | LockFlags::FORCE_WRITE,
        )?;

        // Take fee from the vault
        let mut vault = api
            .field_read_typed::<FungibleVaultBalanceFieldPayload>(vault_handle)?
            .into_latest();
        let fee = vault.take_by_amount(amount).map_err(|_| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(
                VaultError::LockFeeInsufficientBalance,
            ))
        })?;

        // Credit cost units
        let changes = api.credit_cost_units(receiver.clone().into(), fee, contingent)?;

        // Keep changes
        if !changes.is_empty() {
            vault.put(changes);
        }

        // Flush updates
        api.field_write_typed(
            vault_handle,
            &FungibleVaultBalanceFieldPayload::from_content_source(vault),
        )?;
        api.field_close(vault_handle)?;

        // Emitting an event once the fee has been locked
        Runtime::emit_event_no_revert(api, events::fungible_vault::LockFeeEvent { amount })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub fn recall<Y>(amount: Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_recallable(api)?;

        let divisibility = Self::get_divisibility(api)?;
        if !check_fungible_amount(&amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        let taken = Self::internal_take(amount, api)?;

        let bucket = FungibleResourceManagerBlueprint::create_bucket(taken.amount(), api)?;

        Runtime::emit_event(api, events::fungible_vault::RecallEvent { amount })?;

        Ok(bucket)
    }

    pub fn freeze<Y>(to_freeze: VaultFreezeFlags, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_freezable(api)?;

        let frozen_flag_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleVaultField::FreezeStatus.into(),
            LockFlags::MUTABLE,
        )?;

        let mut frozen = api
            .field_read_typed::<FungibleVaultFreezeStatusFieldPayload>(frozen_flag_handle)?
            .into_latest();
        frozen.frozen.insert(to_freeze);
        api.field_write_typed(
            frozen_flag_handle,
            &FungibleVaultFreezeStatusFieldPayload::from_content_source(frozen),
        )?;

        Ok(())
    }

    pub fn unfreeze<Y>(to_unfreeze: VaultFreezeFlags, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_freezable(api)?;

        let frozen_flag_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleVaultField::FreezeStatus.into(),
            LockFlags::MUTABLE,
        )?;
        let mut frozen = api
            .field_read_typed::<FungibleVaultFreezeStatusFieldPayload>(frozen_flag_handle)?
            .into_latest();
        frozen.frozen.remove(to_unfreeze);
        api.field_write_typed(
            frozen_flag_handle,
            &FungibleVaultFreezeStatusFieldPayload::from_content_source(frozen),
        )?;

        Ok(())
    }

    pub fn create_proof_of_amount<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let divisibility = Self::get_divisibility(api)?;
        if !check_fungible_amount(&amount, divisibility) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::InvalidAmount),
            ));
        }

        Self::lock_amount(amount, api)?;

        let proof_info = ProofMoveableSubstate { restricted: false };
        let proof_evidence = FungibleProofSubstate::new(
            amount,
            btreemap!(
                LocalRef::Vault(Reference(receiver.clone().into())) => amount
            ),
        )
        .map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ProofError(e)))
        })?;

        let proof_id = api.new_simple_object(
            FUNGIBLE_PROOF_BLUEPRINT,
            btreemap! {
                0u8 => FieldValue::new(&proof_info),
                1u8 => FieldValue::new(&proof_evidence),
            },
        )?;

        Ok(Proof(Own(proof_id)))
    }

    pub fn burn<Y>(amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        Self::assert_not_frozen(VaultFreezeFlags::BURN, api)?;

        Self::take(&amount, api)?.package_burn(api)?;
        Ok(())
    }

    //===================
    // Protected methods
    //===================

    pub fn lock_amount<Y>(amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleVaultField::LockedBalance.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked = api
            .field_read_typed::<FungibleVaultLockedBalanceFieldPayload>(handle)?
            .into_latest();
        let max_locked = locked.amount();

        // Take from liquid if needed
        if amount > max_locked {
            let delta = amount.safe_sub(max_locked).unwrap();
            Self::internal_take(delta, api)?;
        }

        // Increase lock count
        locked.amounts.entry(amount).or_default().add_assign(1);
        api.field_write_typed(
            handle,
            &FungibleVaultLockedBalanceFieldPayload::from_content_source(locked),
        )?;

        // Issue proof
        Ok(())
    }

    pub fn unlock_amount<Y>(amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleVaultField::LockedBalance.into(),
            LockFlags::MUTABLE,
        )?;
        let mut locked = api
            .field_read_typed::<FungibleVaultLockedBalanceFieldPayload>(handle)?
            .into_latest();

        let max_locked = locked.amount();
        let cnt = locked
            .amounts
            .remove(&amount)
            .expect("Attempted to unlock an amount that is not locked");
        if cnt > 1 {
            locked.amounts.insert(amount, cnt - 1);
        }

        let locked_amount = locked.amount();
        api.field_write_typed(
            handle,
            &FungibleVaultLockedBalanceFieldPayload::from_content_source(locked),
        )?;

        let delta = max_locked.safe_sub(locked_amount).unwrap();
        Self::internal_put(LiquidFungibleResource::new(delta), api)
    }

    //===================
    // Helper methods
    //===================

    fn assert_not_frozen<Y>(flags: VaultFreezeFlags, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api.actor_is_feature_enabled(
            ACTOR_STATE_OUTER_OBJECT,
            FungibleResourceManagerFeature::VaultFreeze.feature_name(),
        )? {
            return Ok(());
        }

        let frozen_flag_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleVaultField::FreezeStatus.into(),
            LockFlags::read_only(),
        )?;
        let frozen = api
            .field_read_typed::<FungibleVaultFreezeStatusFieldPayload>(frozen_flag_handle)?
            .into_latest();
        api.field_close(frozen_flag_handle)?;

        if frozen.frozen.intersects(flags) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::VaultIsFrozen),
            ));
        }

        Ok(())
    }

    fn assert_freezable<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api.actor_is_feature_enabled(
            ACTOR_STATE_OUTER_OBJECT,
            FungibleResourceManagerFeature::VaultFreeze.feature_name(),
        )? {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NotFreezable),
            ));
        }

        Ok(())
    }

    fn assert_recallable<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api.actor_is_feature_enabled(
            ACTOR_STATE_OUTER_OBJECT,
            FungibleResourceManagerFeature::VaultRecall.feature_name(),
        )? {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::VaultError(VaultError::NotRecallable),
            ));
        }

        Ok(())
    }

    fn liquid_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleVaultField::Balance.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref = api
            .field_read_typed::<FungibleVaultBalanceFieldPayload>(handle)?
            .into_latest();
        let amount = substate_ref.amount();
        api.field_close(handle)?;
        Ok(amount)
    }

    fn locked_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleVaultField::LockedBalance.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: LockedFungibleResource = api
            .field_read_typed::<FungibleVaultLockedBalanceFieldPayload>(handle)?
            .into_latest();
        let amount = substate_ref.amount();
        api.field_close(handle)?;
        Ok(amount)
    }

    fn internal_take<Y>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<LiquidFungibleResource, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleVaultField::Balance.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api
            .field_read_typed::<FungibleVaultBalanceFieldPayload>(handle)?
            .into_latest();
        let taken = substate_ref.take_by_amount(amount).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ResourceError(
                e,
            )))
        })?;
        api.field_write_typed(
            handle,
            &FungibleVaultBalanceFieldPayload::from_content_source(substate_ref),
        )?;
        api.field_close(handle)?;

        Ok(taken)
    }

    fn internal_put<Y>(resource: LiquidFungibleResource, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if resource.is_empty() {
            return Ok(());
        }

        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleVaultField::Balance.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api
            .field_read_typed::<FungibleVaultBalanceFieldPayload>(handle)?
            .into_latest();
        substate_ref.put(resource);
        api.field_write_typed(
            handle,
            &FungibleVaultBalanceFieldPayload::from_content_source(substate_ref),
        )?;
        api.field_close(handle)?;

        Ok(())
    }
}
