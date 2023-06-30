use crate::blueprints::pool::two_resource_pool::*;
use crate::errors::*;
use crate::kernel::kernel_api::*;
use native_sdk::modules::access_rules::*;
use native_sdk::modules::metadata::*;
use native_sdk::modules::royalty::*;
use native_sdk::resource::*;
use native_sdk::runtime::Runtime;
use radix_engine_common::math::*;
use radix_engine_common::prelude::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use radix_engine_interface::*;

pub const TWO_RESOURCE_POOL_BLUEPRINT_IDENT: &'static str = "TwoResourcePool";

pub struct TwoResourcePoolBlueprint;
impl TwoResourcePoolBlueprint {
    pub fn instantiate<Y>(
        (resource_address1, resource_address2): (ResourceAddress, ResourceAddress),
        pool_manager_rule: AccessRule,
        api: &mut Y,
    ) -> Result<TwoResourcePoolInstantiateOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi,
    {
        // A pool can't be created between the same resources - error out if it's
        if resource_address1 == resource_address2 {
            return Err(TwoResourcePoolError::ContributionOfEmptyBucketError.into());
        }

        // A pool can't be created where one of the resources is non-fungible - error out if any of
        // them are
        for resource_address in [resource_address1, resource_address2] {
            let resource_manager = ResourceManager(resource_address);
            if let ResourceType::NonFungible { .. } = resource_manager.resource_type(api)? {
                return Err(TwoResourcePoolError::NonFungibleResourcesAreNotAccepted {
                    resource_address,
                }
                .into());
            }
        }

        // Allocating the address of the pool - this is going to be needed for the metadata of the
        // pool unit resource.
        let (address_reservation, address) = api.allocate_global_address(BlueprintId {
            package_address: POOL_PACKAGE,
            blueprint_name: TWO_RESOURCE_POOL_BLUEPRINT_IDENT.to_string(),
        })?;

        // Create owner role of both the pool component and pool unit resource
        let owner_role = OwnerRole::Updatable(pool_manager_rule);

        // Creating the pool unit resource
        let pool_unit_resource_manager = {
            let component_caller_badge = NonFungibleGlobalId::global_caller_badge(address);

            let access_rules = btreemap!(
                Mint => (
                    rule!(require(component_caller_badge.clone())),
                    AccessRule::DenyAll,
                ),
                Burn => (rule!(require(component_caller_badge.clone())), AccessRule::DenyAll),
                Recall => (AccessRule::DenyAll, AccessRule::DenyAll)
            );

            ResourceManager::new_fungible(
                owner_role.clone(),
                true,
                18,
                access_rules,
                metadata_init! {
                    "pool" => address, locked;
                },
                None,
                api,
            )?
        };

        // Creating the pool nodes
        let access_rules = AccessRules::create(owner_role, btreemap!(), api)?.0;

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
            let substate = TwoResourcePoolSubstate {
                vaults: [
                    (resource_address1, Vault::create(resource_address1, api)?),
                    (resource_address2, Vault::create(resource_address2, api)?),
                ],
                pool_unit_resource_manager,
            };
            api.new_simple_object(
                TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                vec![scrypto_encode(&substate).unwrap()],
            )?
        };

        api.globalize(
            btreemap!(
                ObjectModuleId::Main => object_id,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            Some(address_reservation),
        )?;

        Ok(ComponentAddress::new_or_panic(address.as_node_id().0))
    }

    pub fn contribute<Y>(
        (bucket1, bucket2): (Bucket, Bucket),
        api: &mut Y,
    ) -> Result<TwoResourcePoolContributeOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (mut substate, handle) = Self::lock_and_read(api, LockFlags::MUTABLE)?;

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
                return Err(TwoResourcePoolError::ResourceDoesNotBelongToPool {
                    resource_address: bucket1_resource_address,
                }
                .into());
            }
            if bucket2_resource_address != vault2_resource_address {
                return Err(TwoResourcePoolError::ResourceDoesNotBelongToPool {
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
                return Err(TwoResourcePoolError::ContributionOfEmptyBucketError.into());
            }

            match (
                pool_unit_total_supply > Decimal::ZERO,
                reserves1 > Decimal::ZERO,
                reserves2 > Decimal::ZERO,
            ) {
                (false, false, false) => Ok((
                    (contribution1 * contribution2).sqrt().unwrap(),
                    contribution1,
                    contribution2,
                )),
                (false, _, _) => Ok((
                    ((contribution1 + reserves1) * (contribution2 + reserves2))
                        .sqrt()
                        .unwrap(),
                    contribution1,
                    contribution2,
                )),
                (true, true, true) => {
                    // Calculating everything in terms of m, n, dm, and dn where they're defined as
                    // follows:
                    // m:  the reserves of the first resource.
                    // n:  the reserves of the second resource.
                    // dm: the change of m or the amount in the bucket of m being contributed.
                    // dn: the change of n or the amount in the bucket of n being contributed.

                    let m = reserves1;
                    let n = reserves2;
                    let dm = contribution1;
                    let dn = contribution2;

                    let (mut amount1, mut amount2) = if (m / n) == (dm / dn) {
                        (dm, dn)
                    } else if (m / n) < (dm / dn) {
                        (dn * m / n, dn)
                    } else {
                        (dm, dm * n / m)
                    };

                    if divisibility1 != 18 {
                        amount1 = amount1
                            .round(divisibility1 as u32, RoundingMode::TowardsNegativeInfinity)
                    }
                    if divisibility2 != 18 {
                        amount2 = amount2
                            .round(divisibility2 as u32, RoundingMode::TowardsNegativeInfinity)
                    }

                    let pool_units_to_mint = amount1 / reserves1 * pool_unit_total_supply;

                    Ok((pool_units_to_mint, amount1, amount2))
                }
                (true, _, _) => Err(TwoResourcePoolError::NonZeroPoolUnitSupplyButZeroReserves),
            }
        }?;

        // Construct the event - this will be emitted once the resources are contributed to the pool
        let event = ContributionEvent {
            contributed_resources: btreemap! {
                bucket1.resource_address(api)? => bucket1.amount(api)?,
                bucket2.resource_address(api)? => bucket2.amount(api)?,
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

        api.field_lock_release(handle)?;

        Runtime::emit_event(api, event)?;

        Ok((pool_units, change_bucket))
    }

    pub fn redeem<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<TwoResourcePoolRedeemOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;

        // Ensure that the passed pool resources are indeed pool resources
        let bucket_resource_address = bucket.resource_address(api)?;
        if bucket_resource_address != substate.pool_unit_resource_manager.0 {
            return Err(TwoResourcePoolError::InvalidPoolUnitResource {
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
        let mut reserves = BTreeMap::new();
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
            Self::calculate_amount_owed(pool_units_to_redeem, pool_units_total_supply, reserves);

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
        api.field_lock_release(handle)?;

        Runtime::emit_event(api, event)?;

        Ok(buckets)
    }

    pub fn protected_deposit<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<TwoResourcePoolProtectedDepositOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;
        let resource_address = bucket.resource_address(api)?;
        let vault = substate.vault(resource_address);
        if let Some(mut vault) = vault {
            let event = DepositEvent {
                amount: bucket.amount(api)?,
                resource_address,
            };
            vault.put(bucket, api)?;
            api.field_lock_release(handle)?;
            Runtime::emit_event(api, event)?;
            Ok(())
        } else {
            Err(TwoResourcePoolError::ResourceDoesNotBelongToPool { resource_address }.into())
        }
    }

    pub fn protected_withdraw<Y>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<TwoResourcePoolProtectedWithdrawOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;
        let vault = substate.vault(resource_address);

        if let Some(mut vault) = vault {
            let bucket = vault.take(amount, api)?;

            api.field_lock_release(handle)?;

            Runtime::emit_event(
                api,
                WithdrawEvent {
                    amount,
                    resource_address,
                },
            )?;

            Ok(bucket)
        } else {
            Err(TwoResourcePoolError::ResourceDoesNotBelongToPool { resource_address }.into())
        }
    }

    pub fn get_redemption_value<Y>(
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<TwoResourcePoolGetRedemptionValueOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;

        let pool_units_to_redeem = amount_of_pool_units;
        let pool_units_total_supply = substate
            .pool_unit_resource_manager
            .total_supply(api)?
            .expect("Total supply is always enabled for pool unit resource.");
        let mut reserves = BTreeMap::new();
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
            Self::calculate_amount_owed(pool_units_to_redeem, pool_units_total_supply, reserves);

        api.field_lock_release(handle)?;

        Ok(amounts_owed)
    }

    pub fn get_vault_amounts<Y>(
        api: &mut Y,
    ) -> Result<TwoResourcePoolGetVaultAmountsOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (two_resource_pool_substate, handle) =
            Self::lock_and_read(api, LockFlags::read_only())?;
        let amounts = two_resource_pool_substate
            .vaults
            .into_iter()
            .map(|(resource_address, vault)| {
                vault.amount(api).map(|amount| (resource_address, amount))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;

        api.field_lock_release(handle)?;
        Ok(amounts)
    }

    //===================
    // Utility Functions
    //===================

    fn lock_and_read<Y>(
        api: &mut Y,
        lock_flags: LockFlags,
    ) -> Result<(TwoResourcePoolSubstate, LockHandle), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = TwoResourcePoolField::TwoResourcePool.into();
        let handle = api.actor_open_field(OBJECT_HANDLE_SELF, substate_key, lock_flags)?;
        let two_resource_pool_substate =
            api.field_lock_read_typed::<TwoResourcePoolSubstate>(handle)?;

        Ok((two_resource_pool_substate, handle))
    }

    fn calculate_amount_owed(
        pool_units_to_redeem: Decimal,
        pool_units_total_supply: Decimal,
        reserves: BTreeMap<ResourceAddress, ReserveResourceInformation>,
    ) -> BTreeMap<ResourceAddress, Decimal> {
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
                    let amount_owed = (pool_units_to_redeem / pool_units_total_supply) * reserves;

                    let amount_owed = if divisibility == 18 {
                        amount_owed
                    } else {
                        amount_owed
                            .round(divisibility as u32, RoundingMode::TowardsNegativeInfinity)
                    };

                    (resource_address, amount_owed)
                },
            )
            .collect()
    }
}

struct ReserveResourceInformation {
    reserves: Decimal,
    divisibility: u8,
}
