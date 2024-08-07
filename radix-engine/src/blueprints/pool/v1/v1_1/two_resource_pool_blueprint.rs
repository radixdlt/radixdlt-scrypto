use crate::blueprints::pool::v1::constants::*;
use crate::blueprints::pool::v1::errors::two_resource_pool::*;
use crate::blueprints::pool::v1::events::two_resource_pool::*;
use crate::blueprints::pool::v1::substates::two_resource_pool::*;
use crate::internal_prelude::*;
use radix_engine_interface::blueprints::component::*;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::prelude::*;
use radix_engine_interface::*;
use radix_native_sdk::modules::metadata::*;
use radix_native_sdk::modules::role_assignment::*;
use radix_native_sdk::modules::royalty::*;
use radix_native_sdk::resource::*;
use radix_native_sdk::runtime::*;

pub struct TwoResourcePoolBlueprint;
impl TwoResourcePoolBlueprint {
    pub fn instantiate<Y: SystemApi<RuntimeError>>(
        (resource_address1, resource_address2): (ResourceAddress, ResourceAddress),
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<TwoResourcePoolInstantiateOutput, RuntimeError> {
        // A pool can't be created between the same resources - error out if it's
        if resource_address1 == resource_address2 {
            return Err(Error::PoolCreationWithSameResource.into());
        }

        // A pool can't be created where one of the resources is non-fungible - error out if any of
        // them are
        for resource_address in [resource_address1, resource_address2] {
            let resource_manager = ResourceManager(resource_address);
            if let ResourceType::NonFungible { .. } = resource_manager.resource_type(api)? {
                return Err(Error::NonFungibleResourcesAreNotAccepted { resource_address }.into());
            }
        }

        // Allocating the address of the pool - this is going to be needed for the metadata of the
        // pool unit resource.
        let (address_reservation, address) = {
            if let Some(address_reservation) = address_reservation {
                let address = api.get_reservation_address(address_reservation.0.as_node_id())?;
                (address_reservation, address)
            } else {
                api.allocate_global_address(BlueprintId {
                    package_address: POOL_PACKAGE,
                    blueprint_name: TWO_RESOURCE_POOL_BLUEPRINT_IDENT.to_string(),
                })?
            }
        };

        // Creating the pool unit resource
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

        // Creating the pool nodes
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
                "pool_vault_number" => 2u8, locked;
                "pool_resources" => vec![
                    GlobalAddress::from(resource_address1),
                    GlobalAddress::from(resource_address2),
                ], locked;
                "pool_unit" => GlobalAddress::from(pool_unit_resource_manager.0), locked;
            },
            api,
        )?;
        let royalty = ComponentRoyalty::create(ComponentRoyaltyConfig::default(), api)?;
        let object_id = {
            let substate = Substate {
                vaults: [
                    (resource_address1, Vault::create(resource_address1, api)?),
                    (resource_address2, Vault::create(resource_address2, api)?),
                ],
                pool_unit_resource_manager,
            };
            api.new_simple_object(
                TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                indexmap! {
                    TwoResourcePoolField::State.field_index() => FieldValue::immutable(TwoResourcePoolStateFieldPayload::from_content_source(substate)),
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

    /// Contributes resources to the pool and mints pool units back representing the contributed
    /// resources.
    ///
    /// In much of a similar way to the single resource pool, there are a number of states that
    /// this pool can be in, there are more states than the single resource pool since this pool
    /// has the 2 resource reserves instead of one.
    ///
    /// To examine the behavior of this function, lets first define the terminology to use: Let PU
    /// denote the total supply of pool units where 0 means that none exists and 1 means that some
    /// amount exists. Let R1 denote the total amount of reserves of resource 1 that the pool has
    /// where 0 here means that no reserves exist in the pool and 1 means that some reserves exist
    /// in the pool and R2 be the equivalent of R1 but for resource 2.
    ///
    /// PU  R1  R2
    /// 0   0   0  => This a new pool since no pool units currently exist in circulation and there
    ///               are no reserves of either of the resources. This contribution is accepted in
    ///               full with no change and the geometric mean of the contribution amounts will be
    ///               minted as pool units.
    /// 0   0   1  => This is a new pool since no pool units exist in circulation but some dust is
    ///               left behind in the pool from it being used in the past.
    /// 0   1   0  => This is a new pool since no pool units exist in circulation but some dust is
    ///               left behind in the pool from it being used in the past.
    /// 0   1   1  => This is a new pool since no pool units exist in circulation but some dust is
    ///               left behind in the pool from it being used in the past.
    /// 1   0   0  => This is an illegal state and contributions in this state are not allowed! This
    ///               is due to some pool units existing in circulation but the pool has no reserves
    ///               of either of the assets. The contributor should have 100% pool ownership from
    ///               their contribution but some people already have some pool units. The pool will
    ///               not accept contributions when it is in this state.
    /// 1   0   1  => The pool accepts one sided liquidity of R2 and returns all of the given R1 as
    ///               change. The amount of pool units minted is equal to the ratio of the R2
    ///               contribution to the R2 reserves.
    /// 1   1   0  => The pool accepts one sided liquidity of R1 and returns all of the given R2 as
    ///               change. The amount of pool units minted is equal to the ratio of the R1
    ///               contribution to the R1 reserves.
    /// 1   1   1  => The pool is in normal operation and both resources are accepted where some of
    ///               them will be contributed and some of them will be change.
    ///
    /// There are common patterns that can be seen from the states above:
    ///
    /// * State 1: When PU = 0 the pool is considered new, the amount of reserves do not really
    /// matter as the amount of pool units that the pool mins is the same.
    /// * State 2: When PU = 1 and *all* reserves are empty then this is an illegal state that the
    /// pool can't really do anything about.
    /// * State 3: When PU = 1 and _some but not all_ of the reserves are empty then the resources
    /// with the empty reserves are not contributed to the pool and are returned as change.
    /// * State 4: When all is 1 then the pool is in normal operations.
    ///
    /// The above state names will be used in tests as its better than calling them out by name or
    /// by description. A simple state _x_ is a lot simpler.
    pub fn contribute<Y: SystemApi<RuntimeError>>(
        (bucket1, bucket2): (Bucket, Bucket),
        api: &mut Y,
    ) -> Result<TwoResourcePoolContributeOutput, RuntimeError> {
        Self::with_state(api, |mut substate, api| {
            // Sort the buckets and vaults in a predictable order.
            let (mut vault1, mut vault2, bucket1, bucket2) = {
                // Getting the vaults of the two resource pool - before getting them we sort them
                // according to a deterministic and predictable order. This helps make the code less
                // generalized and simple.
                let ((vault1, vault1_resource_address), (vault2, vault2_resource_address)) = {
                    let vault1 = Vault(substate.vaults[0].1 .0);
                    let vault2 = Vault(substate.vaults[1].1 .0);

                    let resource_address1 = substate.vaults[0].0;
                    let resource_address2 = substate.vaults[1].0;

                    if resource_address1 > resource_address2 {
                        ((vault1, resource_address1), (vault2, resource_address2))
                    } else {
                        ((vault2, resource_address2), (vault1, resource_address1))
                    }
                };

                // Getting the buckets of the two resource pool - before getting them we sort them
                // according to a deterministic and predictable order. This helps make the code less
                // generalized and simple.
                let ((bucket1, bucket1_resource_address), (bucket2, bucket2_resource_address)) = {
                    let resource_address1 = bucket1.resource_address(api)?;
                    let resource_address2 = bucket2.resource_address(api)?;

                    if resource_address1 > resource_address2 {
                        ((bucket1, resource_address1), (bucket2, resource_address2))
                    } else {
                        ((bucket2, resource_address2), (bucket1, resource_address1))
                    }
                };

                // Ensure that the two buckets given as arguments match the two vaults that the pool
                // has.
                if bucket1_resource_address != vault1_resource_address {
                    return Err(Error::ResourceDoesNotBelongToPool {
                        resource_address: bucket1_resource_address,
                    }
                    .into());
                }
                if bucket2_resource_address != vault2_resource_address {
                    return Err(Error::ResourceDoesNotBelongToPool {
                        resource_address: bucket2_resource_address,
                    }
                    .into());
                }

                (vault1, vault2, bucket1, bucket2)
            };

            let reserves1 = vault1.amount(api)?;
            let reserves2 = vault2.amount(api)?;

            // Determine the amount of pool units to mint and the amount of resource to contribute
            // to the pool based on the current state of the pool.
            let (amount1, amount2, pool_units_to_mint) = {
                let pool_unit_total_supply = substate
                    .pool_unit_resource_manager
                    .total_supply(api)?
                    .expect("Total supply is always enabled for pool unit resource.");

                let contribution1 = bucket1.amount(api)?;
                let contribution2 = bucket2.amount(api)?;
                let pool_unit_total_supply = PreciseDecimal::from(pool_unit_total_supply);
                let reserves1 = PreciseDecimal::from(reserves1);
                let reserves2 = PreciseDecimal::from(reserves2);
                let contribution1 = PreciseDecimal::from(contribution1);
                let contribution2 = PreciseDecimal::from(contribution2);

                let is_reserves1_not_empty = reserves1 > PreciseDecimal::ZERO;
                let is_reserves2_not_empty = reserves2 > PreciseDecimal::ZERO;
                let is_pool_units_in_circulation = pool_unit_total_supply > PreciseDecimal::ZERO;

                let (amount1, amount2, pool_units_to_mint) = match (
                    is_reserves1_not_empty,
                    is_reserves2_not_empty,
                    is_pool_units_in_circulation,
                ) {
                    // The total supply of pool units is zero and none of them are in circulation.
                    // This pool is currently in the "new pool" state and any amount can be
                    // contributed to it and we will mint for them the geometric average of their
                    // contribution in pool units.
                    (_, _, false) => {
                        let pool_units_to_mint = if contribution1 == PreciseDecimal::ZERO
                            || contribution2 == PreciseDecimal::ZERO
                        {
                            // Take C1 or C2 whichever of them is the largest to avoid minting zero
                            // pool units. If both are zero then zero pool units will be minted for
                            // zero contribution which is fine.
                            contribution1.max(contribution2)
                        } else {
                            // Pool units to mint = Geometric Average
                            //                    = sqrt(contribution1 * contribution2)
                            //                    = sqrt(contribution1) * sqrt(contribution2)
                            contribution1
                                .checked_sqrt()
                                .and_then(|c1| {
                                    contribution2
                                        .checked_sqrt()
                                        .and_then(|c2| c1.checked_mul(c2))
                                })
                                .and_then(|value| {
                                    value.checked_round(18, RoundingMode::ToPositiveInfinity)
                                })
                                .ok_or(Error::DecimalOverflowError)?
                        };
                        (contribution1, contribution2, pool_units_to_mint)
                    }
                    // One sided liquidity - one of the reserves is empty and contributions to it
                    // will be rejected whereas contributions to the other side will be accepted in
                    // full.
                    (false, true, true) => (
                        PreciseDecimal::ZERO,
                        contribution2,
                        contribution2
                            .checked_div(reserves2)
                            .and_then(|d| d.checked_mul(pool_unit_total_supply))
                            .ok_or(Error::DecimalOverflowError)?,
                    ),
                    (true, false, true) => (
                        contribution1,
                        PreciseDecimal::ZERO,
                        contribution1
                            .checked_div(reserves1)
                            .and_then(|d| d.checked_mul(pool_unit_total_supply))
                            .ok_or(Error::DecimalOverflowError)?,
                    ),
                    // Normal operations.
                    //
                    // We need to determine how much of the resources given for contribution can
                    // actually be contributed to keep the ratio of resources in the pool the same.
                    //
                    // This follows the following algorithm:
                    //
                    // * Calculate the amount of contribution 2 required to fulfill contribution 1
                    // in full.
                    // * Calculate the amount of contribution 1 required to fulfill contribution 2
                    // in full.
                    // * Collect this into an array of `[(contribution1, required_contribution2),
                    // (required_contribution1, contribution2)]`.
                    // * Eliminate any entry in the array that exceeds the amount of resources that
                    // were provided for liquidity.
                    (true, true, true) => [
                        contribution1
                            .checked_div(reserves1)
                            .and_then(|d| d.checked_mul(reserves2))
                            .map(|contribution2_required| (contribution1, contribution2_required)),
                        contribution2
                            .checked_div(reserves2)
                            .and_then(|d| d.checked_mul(reserves1))
                            .map(|contribution1_required| (contribution1_required, contribution2)),
                    ]
                    .into_iter()
                    .filter_map(|item| match item {
                        v @ Some((c1, c2)) if c1 <= contribution1 && c2 <= contribution2 => v,
                        _ => None,
                    })
                    .map(|(c1, c2)| -> Result<_, RuntimeError> {
                        let pool_units_to_mint = c1
                            .checked_div(reserves1)
                            .and_then(|d| d.checked_mul(pool_unit_total_supply))
                            .ok_or(Error::DecimalOverflowError)?;
                        Ok((c1, c2, pool_units_to_mint))
                    })
                    .filter_map(Result::ok)
                    .max_by(|(_, _, mint1), (_, _, mint2)| mint1.cmp(mint2))
                    .ok_or(Error::DecimalOverflowError)?,
                    // Illegal State: There is zero reserves in the liquidity pool but a non-zero
                    // amount of pool units. If this contribution goes through, then how much would
                    // this person own in the pool? Probably 100%. To signal that they own 100% we
                    // would have to mint Decimal::MAX or the maximum mint limit which is infeasible
                    // and dilutes the worth of pool units heavily.
                    (false, false, true) => {
                        return Err(Error::NonZeroPoolUnitSupplyButZeroReserves.into())
                    }
                };

                let amount1 =
                    Decimal::try_from(amount1).map_err(|_| Error::DecimalOverflowError)?;
                let amount2 =
                    Decimal::try_from(amount2).map_err(|_| Error::DecimalOverflowError)?;
                let pool_units_to_mint = Decimal::try_from(pool_units_to_mint)
                    .map_err(|_| Error::DecimalOverflowError)?;

                (amount1, amount2, pool_units_to_mint)
            };

            // Get the amounts after the rounding
            let contribution_bucket1 = bucket1.take_advanced(
                amount1,
                WithdrawStrategy::Rounded(RoundingMode::ToNegativeInfinity),
                api,
            )?;
            let contribution_bucket2 = bucket2.take_advanced(
                amount2,
                WithdrawStrategy::Rounded(RoundingMode::ToNegativeInfinity),
                api,
            )?;
            let amount1 = contribution_bucket1.amount(api)?;
            let amount2 = contribution_bucket2.amount(api)?;

            // If the amount that will be contributed of the two resources after the rounding and
            // all is zero despite the reserves of that resource not being zero, then error out. It
            // means that one of the resources was rounded down to zero due to its divisibility.
            if (amount1 == Decimal::ZERO && reserves1 != Decimal::ZERO)
                || (amount2 == Decimal::ZERO && reserves2 != Decimal::ZERO)
            {
                return Err(Error::LargerContributionRequiredToMeetRatio.into());
            }

            // Minting the pool unit tokens
            if pool_units_to_mint == Decimal::ZERO {
                return Err(Error::ZeroPoolUnitsMinted.into());
            }
            let pool_units = substate
                .pool_unit_resource_manager
                .mint_fungible(pool_units_to_mint, api)?;

            // Construct the event - this will be emitted once the resources are contributed to the
            // pool
            let event = ContributionEvent {
                contributed_resources: indexmap! {
                    contribution_bucket1.resource_address(api)? => amount1,
                    contribution_bucket2.resource_address(api)? => amount2,
                },
                pool_units_minted: pool_units_to_mint,
            };

            // Deposit the calculated amount of each of the buckets into appropriate vault.
            vault1.put(contribution_bucket1, api)?;
            vault2.put(contribution_bucket2, api)?;

            // Determine if there is any change to return back to the caller - if there is not then
            // drop the empty buckets.
            //
            // The amount contributed must either be amount1 of bucket1 and/or amount2 of bucket2.
            // This means that there must exist at least 1 empty bucket and at most two. So, this
            // tries to find which of the buckets have been contributed in full to drop them and
            // which have not been contributed in full to treat them as change.
            let change_bucket = if !bucket1.is_empty(api)? {
                bucket2.drop_empty(api)?;
                Some(bucket1)
            } else if !bucket2.is_empty(api)? {
                bucket1.drop_empty(api)?;
                Some(bucket2)
            } else {
                bucket1.drop_empty(api)?;
                bucket2.drop_empty(api)?;
                None
            };

            Runtime::emit_event(api, event)?;

            Ok((pool_units.into(), change_bucket))
        })
    }

    pub fn redeem<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<TwoResourcePoolRedeemOutput, RuntimeError> {
        Self::with_state(api, |substate, api| {
            // Ensure that the passed pool resources are indeed pool resources
            let bucket_resource_address = bucket.resource_address(api)?;
            if bucket_resource_address != substate.pool_unit_resource_manager.0 {
                return Err(Error::InvalidPoolUnitResource {
                    expected: substate.pool_unit_resource_manager.0,
                    actual: bucket_resource_address,
                }
                .into());
            }

            let pool_units_to_redeem = bucket.amount(api)?;
            let pool_units_total_supply = substate
                .pool_unit_resource_manager
                .total_supply(api)?
                .expect("Total supply is always enabled for pool unit resource.");
            let mut reserves = index_map_new();
            for (resource_address, vault) in substate.vaults.iter() {
                let amount = vault.amount(api)?;
                let divisibility = ResourceManager(*resource_address).resource_type(api)
                .map(|resource_type| {
                    if let ResourceType::Fungible { divisibility } = resource_type {
                        divisibility
                    } else {
                        panic!("Impossible case, we check for this in the constructor and have a test for this.")
                    }
                })?;

                reserves.insert(
                    *resource_address,
                    ReserveResourceInformation {
                        reserves: amount,
                        divisibility,
                    },
                );
            }

            let amounts_owed = Self::calculate_amount_owed(
                pool_units_to_redeem,
                pool_units_total_supply,
                reserves,
            )?;

            bucket.burn(api)?;
            Runtime::emit_event(
                api,
                RedemptionEvent {
                    redeemed_resources: amounts_owed.clone(),
                    pool_unit_tokens_redeemed: pool_units_to_redeem,
                },
            )?;

            // The following part does some unwraps and panic-able operations but should never panic.
            {
                let buckets = amounts_owed
                    .into_iter()
                    .map(|(resource_address, amount)| {
                        substate.vault(resource_address).unwrap().take(amount, api)
                    })
                    .collect::<Result<Vec<Bucket>, _>>()?;
                Ok((Bucket(buckets[0].0), Bucket(buckets[1].0)))
            }
        })
    }

    pub fn protected_deposit<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<TwoResourcePoolProtectedDepositOutput, RuntimeError> {
        Self::with_state(api, |substate, api| {
            let resource_address = bucket.resource_address(api)?;
            let vault = substate.vault(resource_address);
            if let Some(mut vault) = vault {
                let event = DepositEvent {
                    amount: bucket.amount(api)?,
                    resource_address,
                };
                vault.put(bucket, api)?;
                Runtime::emit_event(api, event)?;
                Ok(())
            } else {
                Err(Error::ResourceDoesNotBelongToPool { resource_address }.into())
            }
        })
    }

    pub fn protected_withdraw<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<TwoResourcePoolProtectedWithdrawOutput, RuntimeError> {
        Self::with_state(api, |substate, api| {
            let vault = substate.vault(resource_address);

            if let Some(mut vault) = vault {
                let bucket = vault.take_advanced(amount, withdraw_strategy, api)?;
                let withdrawn_amount = bucket.amount(api)?;

                Runtime::emit_event(
                    api,
                    WithdrawEvent {
                        amount: withdrawn_amount,
                        resource_address,
                    },
                )?;

                Ok(bucket)
            } else {
                Err(Error::ResourceDoesNotBelongToPool { resource_address }.into())
            }
        })
    }

    pub fn get_redemption_value<Y: SystemApi<RuntimeError>>(
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<TwoResourcePoolGetRedemptionValueOutput, RuntimeError> {
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

            let mut reserves = index_map_new();
            for (resource_address, vault) in substate.vaults.into_iter() {
                let amount = vault.amount(api)?;
                let divisibility = ResourceManager(resource_address).resource_type(api)
                    .map(|resource_type| {
                        if let ResourceType::Fungible { divisibility } = resource_type {
                            divisibility
                        } else {
                            panic!("Impossible case, we check for this in the constructor and have a test for this.")
                        }
                    })?;

                reserves.insert(
                    resource_address,
                    ReserveResourceInformation {
                        reserves: amount,
                        divisibility,
                    },
                );
            }

            Self::calculate_amount_owed(pool_units_to_redeem, pool_units_total_supply, reserves)
        })
    }

    pub fn get_vault_amounts<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<TwoResourcePoolGetVaultAmountsOutput, RuntimeError> {
        Self::with_state(api, |substate, api| {
            substate
                .vaults
                .into_iter()
                .map(|(resource_address, vault)| {
                    vault.amount(api).map(|amount| (resource_address, amount))
                })
                .collect::<Result<IndexMap<_, _>, _>>()
        })
    }

    //===================
    // Utility Functions
    //===================

    fn with_state<Y: SystemApi<RuntimeError>, O>(
        api: &mut Y,
        callback: impl FnOnce(Substate, &mut Y) -> Result<O, RuntimeError>,
    ) -> Result<O, RuntimeError> {
        // Open
        let substate_key = TwoResourcePoolField::State.into();
        let handle =
            api.actor_open_field(ACTOR_STATE_SELF, substate_key, LockFlags::read_only())?;
        let substate = api
            .field_read_typed::<VersionedTwoResourcePoolState>(handle)?
            .fully_update_and_into_latest_version();

        // Op
        let rtn = callback(substate, api);

        // Close
        if rtn.is_ok() {
            api.field_close(handle)?;
        }
        rtn
    }

    fn calculate_amount_owed(
        pool_units_to_redeem: Decimal,
        pool_units_total_supply: Decimal,
        reserves: IndexMap<ResourceAddress, ReserveResourceInformation>,
    ) -> Result<IndexMap<ResourceAddress, Decimal>, RuntimeError> {
        let pool_units_to_redeem = PreciseDecimal::from(pool_units_to_redeem);
        let pool_units_total_supply = PreciseDecimal::from(pool_units_total_supply);

        reserves
            .into_iter()
            .map(
                |(
                    resource_address,
                    ReserveResourceInformation {
                        divisibility,
                        reserves,
                    },
                )| {
                    let reserves = PreciseDecimal::from(reserves);
                    let amount_owed = pool_units_to_redeem
                        .checked_div(pool_units_total_supply)
                        .and_then(|d| d.checked_mul(reserves))
                        .ok_or(Error::DecimalOverflowError)?;

                    let amount_owed = Decimal::try_from(amount_owed)
                        .ok()
                        .and_then(|value| {
                            value.checked_round(divisibility, RoundingMode::ToNegativeInfinity)
                        })
                        .ok_or(Error::DecimalOverflowError)?;

                    Ok((resource_address, amount_owed))
                },
            )
            .collect()
    }
}

struct ReserveResourceInformation {
    reserves: Decimal,
    divisibility: u8,
}
