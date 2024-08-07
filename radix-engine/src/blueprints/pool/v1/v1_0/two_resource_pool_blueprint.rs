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
                    TwoResourcePoolField::State.field_index() => FieldValue::immutable(&TwoResourcePoolStateFieldPayload::from_content_source(substate)),
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

    pub fn contribute<Y: SystemApi<RuntimeError>>(
        (bucket1, bucket2): (Bucket, Bucket),
        api: &mut Y,
    ) -> Result<TwoResourcePoolContributeOutput, RuntimeError> {
        let (mut substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;

        let (resource_address1, resource_address2, mut vault1, mut vault2, bucket1, bucket2) = {
            // Getting the vaults of the two resource pool - before getting them we sort them according
            // to a deterministic and predictable order. This helps make the code less generalized and
            // simple.
            let ((vault1, vault1_resource_address), (vault2, vault2_resource_address)) = {
                let vault1 = Vault((&substate.vaults[0].1 .0).clone());
                let vault2 = Vault((&substate.vaults[1].1 .0).clone());

                let resource_address1 = substate.vaults[0].0;
                let resource_address2 = substate.vaults[1].0;

                if resource_address1 > resource_address2 {
                    ((vault1, resource_address1), (vault2, resource_address2))
                } else {
                    ((vault2, resource_address2), (vault1, resource_address1))
                }
            };

            // Getting the buckets of the two resource pool - before getting them we sort them according
            // to a deterministic and predictable order. This helps make the code less generalized and
            // simple.
            let ((bucket1, bucket1_resource_address), (bucket2, bucket2_resource_address)) = {
                let resource_address1 = bucket1.resource_address(api)?;
                let resource_address2 = bucket2.resource_address(api)?;

                if resource_address1 > resource_address2 {
                    ((bucket1, resource_address1), (bucket2, resource_address2))
                } else {
                    ((bucket2, resource_address2), (bucket1, resource_address1))
                }
            };

            // Ensure that the two buckets given as arguments match the two vaults that the pool has.
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

            (
                bucket1_resource_address,
                bucket2_resource_address,
                vault1,
                vault2,
                bucket1,
                bucket2,
            )
        };

        // Determine the amount of pool units to mint based on the the current state of the pool.
        let (pool_units_to_mint, amount1, amount2) = {
            let pool_unit_total_supply = substate
                .pool_unit_resource_manager
                .total_supply(api)?
                .expect("Total supply is always enabled for pool unit resource.");
            let reserves1 = vault1.amount(api)?;
            let reserves2 = vault2.amount(api)?;
            let contribution1 = bucket1.amount(api)?;
            let contribution2 = bucket2.amount(api)?;
            let divisibility1 = ResourceManager(resource_address1).resource_type(api).map(|resource_type| {
                if let ResourceType::Fungible { divisibility } = resource_type {
                    divisibility
                } else {
                    panic!("Impossible case, we check for this in the constructor and have a test for this.")
                }
            })?;
            let divisibility2 = ResourceManager(resource_address2).resource_type(api).map(|resource_type| {
                if let ResourceType::Fungible { divisibility } = resource_type {
                    divisibility
                } else {
                    panic!("Impossible case, we check for this in the constructor and have a test for this.")
                }
            })?;

            if contribution1 == Decimal::ZERO || contribution2 == Decimal::ZERO {
                return Err(Error::ContributionOfEmptyBucketError.into());
            }

            match (
                pool_unit_total_supply > Decimal::ZERO,
                reserves1 > Decimal::ZERO,
                reserves2 > Decimal::ZERO,
            ) {
                (false, false, false) => Ok((
                    /*
                    This is doing the following:
                    dec(
                        round(
                            sqrt(pdec(c1)) * sqrt(pdec(c2)),
                            19
                        )
                    )
                     */
                    PreciseDecimal::from(contribution1)
                        .checked_sqrt()
                        .and_then(|c1_sqrt| {
                            PreciseDecimal::from(contribution2)
                                .checked_sqrt()
                                .and_then(|c2_sqrt| c1_sqrt.checked_mul(c2_sqrt))
                        })
                        .and_then(|d| d.checked_round(19, RoundingMode::ToPositiveInfinity))
                        .and_then(|d| Decimal::try_from(d).ok())
                        .ok_or(Error::DecimalOverflowError)?,
                    contribution1,
                    contribution2,
                )),
                (false, _, _) => Ok((
                    /*
                    This is doing the following:
                    dec(
                        round(
                            sqrt(pdec(c1) + pdec(r1)) * sqrt(pdec(c2) + pdec(r2)),
                            19
                        )
                    )
                     */
                    PreciseDecimal::from(contribution1)
                        .checked_add(PreciseDecimal::from(reserves1))
                        .and_then(|d| d.checked_sqrt())
                        .and_then(|sqrt_cr1| {
                            PreciseDecimal::from(contribution2)
                                .checked_add(PreciseDecimal::from(reserves2))
                                .and_then(|d| d.checked_sqrt())
                                .and_then(|sqrt_cr2| sqrt_cr1.checked_mul(sqrt_cr2))
                        })
                        .and_then(|d| d.checked_round(19, RoundingMode::ToPositiveInfinity))
                        .and_then(|d| Decimal::try_from(d).ok())
                        .ok_or(Error::DecimalOverflowError)?,
                    contribution1,
                    contribution2,
                )),
                (true, true, true) => {
                    // We need to determine how much of the resources given for contribution can
                    // actually be contributed to keep the ratio of resources in the pool the same.
                    //
                    // The logic to do this follows a simple algorithm:
                    // For contribution1 we calculated the required_contribution2. We do the same
                    // for contribution2 we calculated the required_contribution1. We collect them
                    // into an array of tuples of:
                    // [
                    //     (contribution1, required_contribution2),
                    //     (required_contribution1, contribution2)
                    // ]
                    // We filter out entries in this array where the amounts contributed is less
                    // than the amounts required.
                    //
                    // If both of the entries remain in the array, we calculate the pool units that
                    // can be minted for both of them and then take the one which yield the largest
                    // amount of pool units.
                    [
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
                    .map(|(c1, c2)| -> Result<(Decimal, Decimal), RuntimeError> {
                        Ok((
                            c1.checked_round(divisibility1, RoundingMode::ToNegativeInfinity)
                                .ok_or(Error::DecimalOverflowError)?,
                            c2.checked_round(divisibility2, RoundingMode::ToNegativeInfinity)
                                .ok_or(Error::DecimalOverflowError)?,
                        ))
                    })
                    .filter_map(Result::ok)
                    .map(
                        |(c1, c2)| -> Result<(Decimal, Decimal, Decimal), RuntimeError> {
                            let pool_units_to_mint = c1
                                .checked_div(reserves1)
                                .and_then(|d| d.checked_mul(pool_unit_total_supply))
                                .ok_or(Error::DecimalOverflowError)?;
                            Ok((pool_units_to_mint, c1, c2))
                        },
                    )
                    .filter_map(Result::ok)
                    .max_by(|(mint1, _, _), (mint2, _, _)| mint1.cmp(mint2))
                    .ok_or(Error::DecimalOverflowError)
                }
                (true, _, _) => Err(Error::NonZeroPoolUnitSupplyButZeroReserves),
            }
        }?;

        // Construct the event - this will be emitted once the resources are contributed to the pool
        let event = ContributionEvent {
            contributed_resources: indexmap! {
                bucket1.resource_address(api)? => amount1,
                bucket2.resource_address(api)? => amount2,
            },
            pool_units_minted: pool_units_to_mint,
        };

        // Minting the pool unit tokens
        let pool_units = substate
            .pool_unit_resource_manager
            .mint_fungible(pool_units_to_mint, api)?;

        // Deposit the calculated amount of each of the buckets into appropriate vault.
        bucket1
            .take(amount1, api)
            .and_then(|bucket| vault1.put(bucket, api))?;
        bucket2
            .take(amount2, api)
            .and_then(|bucket| vault2.put(bucket, api))?;

        // Determine if there is any change to return back to the caller - if there is not then drop
        // the empty buckets.
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

        api.field_close(handle)?;

        Runtime::emit_event(api, event)?;

        Ok((pool_units.into(), change_bucket))
    }

    pub fn redeem<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<TwoResourcePoolRedeemOutput, RuntimeError> {
        let (substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;

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

        let amounts_owed =
            Self::calculate_amount_owed(pool_units_to_redeem, pool_units_total_supply, reserves)?;

        let event = RedemptionEvent {
            redeemed_resources: amounts_owed.clone(),
            pool_unit_tokens_redeemed: pool_units_to_redeem,
        };

        // The following part does some unwraps and panic-able operations but should never panic.
        let buckets = {
            let buckets = amounts_owed
                .into_iter()
                .map(|(resource_address, amount)| {
                    substate.vault(resource_address).unwrap().take(amount, api)
                })
                .collect::<Result<Vec<Bucket>, _>>()?;
            (Bucket(buckets[0].0), Bucket(buckets[1].0))
        };

        bucket.burn(api)?;
        api.field_close(handle)?;

        Runtime::emit_event(api, event)?;

        Ok(buckets)
    }

    pub fn protected_deposit<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<TwoResourcePoolProtectedDepositOutput, RuntimeError> {
        let (substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;
        let resource_address = bucket.resource_address(api)?;
        let vault = substate.vault(resource_address);
        if let Some(mut vault) = vault {
            let event = DepositEvent {
                amount: bucket.amount(api)?,
                resource_address,
            };
            vault.put(bucket, api)?;
            api.field_close(handle)?;
            Runtime::emit_event(api, event)?;
            Ok(())
        } else {
            Err(Error::ResourceDoesNotBelongToPool { resource_address }.into())
        }
    }

    pub fn protected_withdraw<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<TwoResourcePoolProtectedWithdrawOutput, RuntimeError> {
        let (substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;
        let vault = substate.vault(resource_address);

        if let Some(mut vault) = vault {
            let bucket = vault.take_advanced(amount, withdraw_strategy, api)?;
            api.field_close(handle)?;
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
    }

    pub fn get_redemption_value<Y: SystemApi<RuntimeError>>(
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<TwoResourcePoolGetRedemptionValueOutput, RuntimeError> {
        let (substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;

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

        let amounts_owed =
            Self::calculate_amount_owed(pool_units_to_redeem, pool_units_total_supply, reserves)?;

        api.field_close(handle)?;

        Ok(amounts_owed)
    }

    pub fn get_vault_amounts<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<TwoResourcePoolGetVaultAmountsOutput, RuntimeError> {
        let (two_resource_pool_substate, handle) =
            Self::lock_and_read(api, LockFlags::read_only())?;
        let amounts = two_resource_pool_substate
            .vaults
            .into_iter()
            .map(|(resource_address, vault)| {
                vault.amount(api).map(|amount| (resource_address, amount))
            })
            .collect::<Result<IndexMap<_, _>, _>>()?;

        api.field_close(handle)?;
        Ok(amounts)
    }

    //===================
    // Utility Functions
    //===================

    fn lock_and_read<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
        lock_flags: LockFlags,
    ) -> Result<(Substate, SubstateHandle), RuntimeError> {
        let substate_key = TwoResourcePoolField::State.into();
        let handle = api.actor_open_field(ACTOR_STATE_SELF, substate_key, lock_flags)?;
        let two_resource_pool_substate = api
            .field_read_typed::<VersionedTwoResourcePoolState>(handle)?
            .fully_update_and_into_latest_version();

        Ok((two_resource_pool_substate, handle))
    }

    fn calculate_amount_owed(
        pool_units_to_redeem: Decimal,
        pool_units_total_supply: Decimal,
        reserves: IndexMap<ResourceAddress, ReserveResourceInformation>,
    ) -> Result<IndexMap<ResourceAddress, Decimal>, RuntimeError> {
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
                    let amount_owed = pool_units_to_redeem
                        .checked_div(pool_units_total_supply)
                        .and_then(|d| d.checked_mul(reserves))
                        .ok_or(Error::DecimalOverflowError)?;

                    let amount_owed = if divisibility == 18 {
                        amount_owed
                    } else {
                        amount_owed
                            .checked_round(divisibility, RoundingMode::ToNegativeInfinity)
                            .ok_or(Error::DecimalOverflowError)?
                    };

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
