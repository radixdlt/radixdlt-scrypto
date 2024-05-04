use crate::blueprints::pool::v1::constants::*;
use crate::blueprints::pool::v1::errors::one_resource_pool::*;
use crate::blueprints::pool::v1::events::one_resource_pool::*;
use crate::blueprints::pool::v1::substates::one_resource_pool::*;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::*;
use native_sdk::modules::metadata::*;
use native_sdk::modules::role_assignment::*;
use native_sdk::modules::royalty::*;
use native_sdk::resource::*;
use native_sdk::runtime::*;
use radix_engine_interface::blueprints::component::*;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::prelude::*;
use radix_engine_interface::*;

pub struct OneResourcePoolBlueprint;
impl OneResourcePoolBlueprint {
    pub fn instantiate<Y>(
        resource_address: ResourceAddress,
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<OneResourcePoolInstantiateOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi,
    {
        // Validate that the resource is a fungible resource - a pool can't be created with non
        // fungible resources.
        let resource_manager = ResourceManager(resource_address);
        if let ResourceType::NonFungible { .. } = resource_manager.resource_type(api)? {
            Err(Error::NonFungibleResourcesAreNotAccepted { resource_address })?
        }

        // Allocating the component address of the pool - this will be used later for the component
        // caller badge.
        let (address_reservation, address) = {
            if let Some(address_reservation) = address_reservation {
                let address = api.get_reservation_address(address_reservation.0.as_node_id())?;
                (address_reservation, address)
            } else {
                api.allocate_global_address(BlueprintId {
                    package_address: POOL_PACKAGE,
                    blueprint_name: ONE_RESOURCE_POOL_BLUEPRINT_IDENT.to_string(),
                })?
            }
        };

        let pool_unit_resource_manager = {
            let component_caller_badge = NonFungibleGlobalId::global_caller_badge(address);

            ResourceManager::new_fungible(
                owner_role.clone(),
                true,
                18,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(require(component_caller_badge.clone()));
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner => rule!(require(component_caller_badge.clone()));
                        burner_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata_init! {
                    "pool" => address, locked;
                },
                None,
                api,
            )?
        };

        let role_assignment = RoleAssignment::create(
            owner_role,
            indexmap! {
                ModuleId::Main => roles_init! {
                    RoleKey { key: POOL_MANAGER_ROLE.to_owned() } => pool_manager_rule;
                }
            },
            api,
        )?
        .0;
        let metadata = Metadata::create_with_data(
            metadata_init! {
                "pool_vault_number" => 1u8, locked;
                "pool_resources" => vec![GlobalAddress::from(resource_address)], locked;
                "pool_unit" => GlobalAddress::from(pool_unit_resource_manager.0), locked;
            },
            api,
        )?;
        let royalty = ComponentRoyalty::create(ComponentRoyaltyConfig::default(), api)?;
        let object_id = {
            let vault = Vault::create(resource_address, api)?;
            let substate = Substate {
                vault,
                pool_unit_resource_manager,
            };
            api.new_simple_object(
                ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
                indexmap! {
                    OneResourcePoolField::State.field_index() => FieldValue::immutable(OneResourcePoolStateFieldPayload::from_content_source(substate)),
                },
            )?
        };

        api.globalize(
            object_id,
            indexmap!(
                AttachedModuleId::RoleAssignment => role_assignment.0,
                AttachedModuleId::Metadata => metadata.0,
                AttachedModuleId::Royalty => royalty.0,
            ),
            Some(address_reservation),
        )?;

        Ok(Global::new(ComponentAddress::new_or_panic(
            address.as_node_id().0,
        )))
    }

    pub fn contribute<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<OneResourcePoolContributeOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if bucket.is_empty(api)? {
            return Err(Error::ContributionOfEmptyBucketError.into());
        }

        Self::with_state(api, |mut substate, api| {
            // Check if the provided resource belongs to the pool or not. If not, then error out.
            {
                let input_resource_address = bucket.resource_address(api)?;
                let pool_reserves_resource_address = substate.vault.resource_address(api)?;

                if input_resource_address != pool_reserves_resource_address {
                    return Err(Error::ResourceDoesNotBelongToPool {
                        resource_address: input_resource_address,
                    }
                    .into());
                }
            }

            /*
            There are four states that the pool could be in at this point of time depending on the
            total supply of the pool units and the total amount of reserves that the pool has.
            We can examine each of those states.

            Let PU denote the total supply of pool units where 0 means that none exists and 1 means
            that some amount exists. Let R denote the total amount of reserves that the pool has
            where 0 here means that no reserves exist in the pool and 1 means that some reserves
            exist in the pool.

            PU  R
            0   0 => This is a new pool - no pool units and no pool reserves.
            0   1 => This is a pool which has been used but has dried out and all of the pool units
                     have been burned. The first contribution to this pool gets whatever dust is
                     left behind.
            1   0 => This is an illegal state! Some amount of people own some % of zero which is
                     invalid. There is pretty much nothing we can do in this case because we can't
                     determine how much pool units to mint for this contribution. To signify that
                     the user has 100% ownership of the pool we must mint the maximum mint amount
                     which will dilute the worth of the pool units.
            1   1 => The pool is in normal operations.

            Thus depending on the supply of these resources the pool behaves differently and the
            amount of pool units to mint changes as well.
             */

            let initial_reserves_decimal = substate.vault.amount(api)?;
            let initial_pool_unit_total_supply_decimal = substate
                .pool_unit_resource_manager
                .total_supply(api)?
                .expect("Total supply is always enabled for pool unit resource.");
            let amount_of_contributed_resources_decimal = bucket.amount(api)?;

            let initial_reserves = PreciseDecimal::from(initial_reserves_decimal);
            let initial_pool_unit_total_supply =
                PreciseDecimal::from(initial_pool_unit_total_supply_decimal);
            let amount_of_contributed_resources =
                PreciseDecimal::from(amount_of_contributed_resources_decimal);

            let pool_units_to_mint = match (
                initial_pool_unit_total_supply > PreciseDecimal::ZERO,
                initial_reserves > PreciseDecimal::ZERO,
            ) {
                (false, false) => Ok(amount_of_contributed_resources),
                (false, true) => amount_of_contributed_resources
                    .checked_add(initial_reserves)
                    .ok_or(Error::DecimalOverflowError),
                (true, false) => Err(Error::NonZeroPoolUnitSupplyButZeroReserves),
                // Note: we do the division first to make it harder for the calculation to overflow.
                (true, true) => amount_of_contributed_resources
                    .checked_div(initial_reserves)
                    .and_then(|d| d.checked_mul(initial_pool_unit_total_supply))
                    .ok_or(Error::DecimalOverflowError),
            }?;
            let pool_units_to_mint =
                Decimal::try_from(pool_units_to_mint).map_err(|_| Error::DecimalOverflowError)?;
            if pool_units_to_mint == Decimal::ZERO {
                return Err(Error::ZeroPoolUnitsMinted.into());
            }
            Runtime::emit_event(
                api,
                ContributionEvent {
                    amount_of_resources_contributed: amount_of_contributed_resources_decimal,
                    pool_units_minted: pool_units_to_mint,
                },
            )?;
            substate.vault.put(bucket, api)?;

            substate
                .pool_unit_resource_manager
                .mint_fungible(pool_units_to_mint, api)
        })
    }

    pub fn redeem<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<OneResourcePoolRedeemOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::with_state(api, |mut substate, api| {
            // Ensure that the passed pool resources are indeed pool resources
            let bucket_resource_address = bucket.resource_address(api)?;
            if bucket_resource_address != substate.pool_unit_resource_manager.0 {
                return Err(Error::InvalidPoolUnitResource {
                    expected: substate.pool_unit_resource_manager.0,
                    actual: bucket_resource_address,
                }
                .into());
            }

            // Calculating the amount owed based on the passed pool units.
            let pool_units_to_redeem = bucket.amount(api)?;
            let initial_pool_units_total_supply = substate
                .pool_unit_resource_manager
                .total_supply(api)?
                .expect("Total supply is always enabled for pool unit resource.");
            let initial_pool_resource_reserves = substate.vault.amount(api)?;
            let reserves_divisibility = substate.vault
            .resource_address(api)
            .and_then(|resource_address| ResourceManager(resource_address).resource_type(api))
            .map(|resource_type| {
                if let ResourceType::Fungible { divisibility } = resource_type {
                    divisibility
                } else {
                    panic!("Impossible case, we check for this in the constructor and have a test for this.")
                }
            })?;

            let amount_owed = Self::calculate_amount_owed(
                pool_units_to_redeem,
                initial_pool_units_total_supply,
                initial_pool_resource_reserves,
                reserves_divisibility,
            )?;

            // Return an error if the amount owed to them is zero. This is to guard from cases where
            // the amount owed is zero due to the divisibility. As an example. Imagine a pool with
            // reserves of 100.00 of a resource that has two divisibility and a pool unit total
            // supply of 100 pool units. Redeeming 10^-18 pool units from this pool would mean
            // redeeming 10^-18 tokens which is invalid for this resource's divisibility. Thus, this
            // calculation would round to 0.
            if amount_owed == Decimal::ZERO {
                return Err(Error::RedeemedZeroTokens.into());
            }

            Runtime::emit_event(
                api,
                RedemptionEvent {
                    pool_unit_tokens_redeemed: pool_units_to_redeem,
                    redeemed_amount: amount_owed,
                },
            )?;

            // Burn the pool units and take the owed resources from the bucket.
            bucket.burn(api)?;
            substate.vault.take(amount_owed, api)
        })
    }

    pub fn protected_deposit<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<OneResourcePoolProtectedDepositOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let bucket_amount = bucket.amount(api)?;

        Self::with_state(api, |mut substate, api| substate.vault.put(bucket, api))?;

        Runtime::emit_event(
            api,
            DepositEvent {
                amount: bucket_amount,
            },
        )?;

        Ok(())
    }

    pub fn protected_withdraw<Y>(
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<OneResourcePoolProtectedWithdrawOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let bucket = Self::with_state(api, |mut substate, api| {
            substate.vault.take_advanced(amount, withdraw_strategy, api)
        })?;

        let withdrawn_amount = bucket.amount(api)?;
        Runtime::emit_event(
            api,
            WithdrawEvent {
                amount: withdrawn_amount,
            },
        )?;

        Ok(bucket)
    }

    pub fn get_redemption_value<Y>(
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<OneResourcePoolGetRedemptionValueOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::with_state(api, |substate, api| {
            let pool_units_to_redeem = amount_of_pool_units;
            let pool_units_total_supply = substate
                .pool_unit_resource_manager
                .total_supply(api)?
                .expect("Total supply is always enabled for pool unit resource.");

            if amount_of_pool_units.is_negative()
                || amount_of_pool_units.is_zero()
                || amount_of_pool_units > pool_units_total_supply
            {
                return Err(Error::InvalidGetRedemptionAmount.into());
            }

            let pool_resource_reserves = substate.vault.amount(api)?;
            let pool_resource_divisibility = substate.vault
            .resource_address(api)
            .and_then(|resource_address| ResourceManager(resource_address).resource_type(api))
            .map(|resource_type| {
                if let ResourceType::Fungible { divisibility } = resource_type {
                    divisibility
                } else {
                    panic!("Impossible case, we check for this in the constructor and have a test for this.")
                }
            })?;

            Self::calculate_amount_owed(
                pool_units_to_redeem,
                pool_units_total_supply,
                pool_resource_reserves,
                pool_resource_divisibility,
            )
        })
    }

    pub fn get_vault_amount<Y>(
        api: &mut Y,
    ) -> Result<OneResourcePoolGetVaultAmountOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::with_state(api, |substate, api| substate.vault.amount(api))
    }

    //===================
    // Utility Functions
    //===================

    fn calculate_amount_owed(
        pool_units_to_redeem: Decimal,
        pool_units_total_supply: Decimal,
        reserves_amount: Decimal,
        reserves_divisibility: u8,
    ) -> Result<Decimal, RuntimeError> {
        let pool_units_to_redeem = PreciseDecimal::from(pool_units_to_redeem);
        let pool_units_total_supply = PreciseDecimal::from(pool_units_total_supply);
        let reserves_amount = PreciseDecimal::from(reserves_amount);

        let amount_owed = pool_units_to_redeem
            .checked_div(pool_units_total_supply)
            .and_then(|d| d.checked_mul(reserves_amount))
            .ok_or(Error::DecimalOverflowError)?;

        Decimal::try_from(amount_owed)
            .ok()
            .and_then(|value| {
                value.checked_round(reserves_divisibility, RoundingMode::ToNegativeInfinity)
            })
            .ok_or(Error::DecimalOverflowError.into())
    }

    /// Opens the substate, executes the callback, and closes the substate.
    fn with_state<Y, F, O>(api: &mut Y, callback: F) -> Result<O, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
        F: FnOnce(Substate, &mut Y) -> Result<O, RuntimeError>,
    {
        // Open
        let substate_key = OneResourcePoolField::State.into();
        let handle =
            api.actor_open_field(ACTOR_STATE_SELF, substate_key, LockFlags::read_only())?;
        let substate = api
            .field_read_typed::<VersionedOneResourcePoolState>(handle)?
            .into_latest();

        // Op
        let rtn = callback(substate, api);

        // Close
        if rtn.is_ok() {
            api.field_close(handle)?;
        }
        rtn
    }
}
