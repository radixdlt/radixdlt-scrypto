use crate::blueprints::pool::multi_resource_pool::*;
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

pub const MULTI_RESOURCE_POOL_BLUEPRINT_IDENT: &'static str = "MultiResourcePool";

pub struct MultiResourcePoolBlueprint;
impl MultiResourcePoolBlueprint {
    pub fn instantiate<Y>(
        resource_addresses: BTreeSet<ResourceAddress>,
        pool_manager_rule: AccessRule,
        api: &mut Y,
    ) -> Result<MultiResourcePoolInstantiateOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi,
    {
        // A pool can't be created where one of the resources is non-fungible - error out if any of
        // them are
        for resource_address in resource_addresses.iter() {
            let resource_manager = ResourceManager(*resource_address);
            if let ResourceType::NonFungible { .. } = resource_manager.resource_type(api)? {
                return Err(MultiResourcePoolError::NonFungibleResourcesAreNotAccepted {
                    resource_address: *resource_address,
                }
                .into());
            }
        }

        // A multi-resource pool can not be created with no resources - at minimum there should be
        // one resource.
        if resource_addresses.len() < 1 {
            return Err(MultiResourcePoolError::CantCreatePoolWithLessThanOneResource.into());
        }

        // Allocating the address of the pool - this is going to be needed for the metadata of the
        // pool unit resource.
        let (address_reservation, address) = api.allocate_global_address(BlueprintId {
            package_address: POOL_PACKAGE,
            blueprint_name: MULTI_RESOURCE_POOL_BLUEPRINT_IDENT.to_string(),
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
                "pool_vault_number" => resource_addresses.len() as u64, locked;
                "pool_resources" => resource_addresses.iter().cloned().map(GlobalAddress::from).collect::<Vec<_>>(), locked;
                "pool_unit" => GlobalAddress::from(pool_unit_resource_manager.0), locked;
            },
            api,
        )?;
        let royalty = ComponentRoyalty::create(ComponentRoyaltyConfig::default(), api)?;
        let object_id = {
            let substate = MultiResourcePoolSubstate {
                vaults: resource_addresses
                    .into_iter()
                    .map(|resource_address| {
                        Vault::create(resource_address, api).map(|vault| (resource_address, vault))
                    })
                    .collect::<Result<_, _>>()?,
                pool_unit_resource_manager,
            };
            api.new_simple_object(
                MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
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

    /**
    This function calculates the amount of resources that should be contributed to the pool and then
    contributes them to the pool returning back a pool unit resource in exchange for the contributed
    resources.

    Note that this function checks to ensure that:
    - Some amount of resources were provided for each of the resources in the pool, otherwise the
    operation errors out.
    - No buckets are provided which do not belong to the liquidity pool. If this happens then the
    contribution logic will fail and abort the transaction.

    Note: it is acceptable for two buckets to contain the same resource, as long as the above checks
    pass then this is acceptable and the pool can account for it accordingly.

    In the case where the pool is new and there are currently no pool units all of the resources are
    accepted and the pool mints as many pool units as the geometric average of the contributed
    resources.

    There are three operation modes that a pool can be in:

    - **Pool units total supply is zero:** regardless of whether there are some reserves or not,
    the pool is considered to be back to its initial state. The first contributor is able to
    determine the amount that they wish to contribute and they get minted an amount of pool units
    that is equal to the geometric average of their contribution.
    - **Pool units total supply is not zero, but some reserves are empty:** In this case, the pool is
    said to be in an illegal state. Some people out there are holding pool units that equate to some
    percentage of zero, which is an illegal state for the pool to be in.
    - **Pool units total supply is not zero, none of the reserves are empty:** The pool is operating
    normally and is governed by the antilogarithm discussed below.

    In the case when the pool is operating normally an antilogarithm is needed to determine the
    following:
    - Given some resources, how much of these resources can the pool accept.
    - Given some resources, how much pool units should the pool mint.

    Let r<sub>1</sub>, r<sub>2</sub>, ..., r<sub>n</sub> be the reserves of the resources in the
    pool and c<sub>1</sub>, c<sub>2</sub>, ..., c<sub>n</sub> the amounts being contributed to each
    of the resources in the pool.

    We calculate the ratios of contribution to reserves which is denoted as k<sub>n</sub> where
    k<sub>n</sub> = c<sub>n</sub> / r<sub>n</sub> such that we find k<sub>1</sub>, k<sub>2</sub>,
    ..., k<sub>n</sub>. We then find the minimum value of k denoted as k<sub>min</sub> which gives
    us the ratio which can be satisfied by all of the resources provided for contribution.

    To determine the amount of resources that should be contributed k<sub>min</sub> is multiplied by
    the reserves of each of the resources. This amount is put in the pool's vaults and whatever
    amount remains is returned as change.

    To determine the amount of pool units to mint k<sub>min</sub> is multiplied by the pool units
    total supply.

    The following is a minimal example

    | n               |1     |2     |3     | Note                             |
    |---------------- |:----:|:----:|:----:| -------------------------------- |
    | r               | 1000 | 2000 | 3000 |                                  |
    | c               | 2000 | 3000 | 4000 |                                  |
    | k               | 2    | 1.5  | 1.33 |                                  |
    | k<sub>min</sub> |      |      | 1.33 |                                  |
    | ca              | 1333 | 2666 | 4000 | Amount of contribution to accept |
    */
    pub fn contribute<Y>(
        buckets: Vec<Bucket>,
        api: &mut Y,
    ) -> Result<MultiResourcePoolContributeOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (mut substate, lock_handle) = Self::lock_and_read(api, LockFlags::read_only())?;

        // Checks
        let amounts_of_resources_provided = {
            // Checking that all of the buckets passed belong to this pool
            let mut resource_bucket_amount_mapping = substate
                .vaults
                .keys()
                .map(|resource_address| (*resource_address, Decimal::ZERO))
                .collect::<BTreeMap<ResourceAddress, Decimal>>();
            for bucket in buckets.iter() {
                let bucket_resource_address = bucket.resource_address(api)?;
                let bucket_amount = bucket.amount(api)?;
                if let Some(value) =
                    resource_bucket_amount_mapping.get_mut(&bucket_resource_address)
                {
                    *value += bucket_amount;
                    Ok(())
                } else {
                    Err(MultiResourcePoolError::ResourceDoesNotBelongToPool {
                        resource_address: bucket_resource_address,
                    })
                }?;
            }

            // Checking that there are no buckets missing.
            let resources_with_missing_buckets = resource_bucket_amount_mapping
                .iter()
                .filter_map(|(resource_address, amount_provided)| {
                    if amount_provided.is_zero() {
                        Some(*resource_address)
                    } else {
                        None
                    }
                })
                .collect::<BTreeSet<ResourceAddress>>();

            if resources_with_missing_buckets.len() != 0 {
                Err(MultiResourcePoolError::MissingOrEmptyBuckets {
                    resource_addresses: resources_with_missing_buckets,
                })
            } else {
                Ok(())
            }?;

            resource_bucket_amount_mapping
        };

        let pool_unit_total_supply = substate
            .pool_unit_resource_manager
            .total_supply(api)?
            .expect("Total supply is always enabled for pool unit resource.");
        // Case: New Pool
        let (pool_units, change) = if pool_unit_total_supply.is_zero() {
            // Regarding the unwrap here, there are two cases here where this unwrap could panic:
            // 1- If the value.sqrt is done on a negative decimal - this is impossible, how can the
            //    amount of buckets in a vault be negative?
            // 2- If reduce is called over an empty iterator - this is also impossible, we ensure
            //    that the pool has at least one resource.
            let pool_units_to_mint = amounts_of_resources_provided
                .values()
                .copied()
                .reduce(|acc, item| acc * item)
                .and_then(|value| value.sqrt())
                .unwrap();

            // The following unwrap is safe to do. We've already checked that all of the buckets
            // provided belong to the pool and have a corresponding vault.
            for bucket in buckets {
                let bucket_resource_address = bucket.resource_address(api)?;
                substate
                    .vaults
                    .get_mut(&bucket_resource_address)
                    .unwrap()
                    .put(bucket, api)?;
            }

            Runtime::emit_event(
                api,
                ContributionEvent {
                    contributed_resources: amounts_of_resources_provided,
                    pool_units_minted: pool_units_to_mint,
                },
            )?;

            (
                substate
                    .pool_unit_resource_manager
                    .mint_fungible(pool_units_to_mint, api)?,
                vec![],
            )
        } else {
            // Check if any of the vaults are empty. If any of them are, then the pool is in an
            // illegal state and it can not be contributed to.
            for vault in substate.vaults.values() {
                let amount = vault.amount(api)?;
                if amount.is_zero() {
                    return Err(MultiResourcePoolError::NonZeroPoolUnitSupplyButZeroReserves.into());
                }
            }

            let mut vaults_and_buckets = BTreeMap::<ResourceAddress, (Vault, Bucket)>::new();
            for bucket in buckets.into_iter() {
                let bucket_resource_address = bucket.resource_address(api)?;

                if let Some((_, store_bucket)) =
                    vaults_and_buckets.get_mut(&bucket_resource_address)
                {
                    store_bucket.put(bucket, api)?;
                } else {
                    let vault = substate.vaults.get(&bucket_resource_address).map_or(
                        Err(MultiResourcePoolError::ResourceDoesNotBelongToPool {
                            resource_address: bucket_resource_address,
                        }),
                        |vault| Ok(Vault(vault.0.clone())),
                    )?;

                    vaults_and_buckets.insert(bucket_resource_address, (vault, bucket));
                };
            }

            // Safe to unwrap here as well. Min returns `None` if called on an empty iterator. The
            // pool has a minimum of one resource at all times thus min is never none.
            let minimum_ratio = *vaults_and_buckets
                .values()
                .map(|(vault, bucket)| {
                    vault.amount(api).and_then(|vault_amount| {
                        bucket
                            .amount(api)
                            .map(|bucket_amount| bucket_amount / vault_amount)
                    })
                })
                .collect::<Result<Vec<Decimal>, _>>()?
                .iter()
                .min()
                .unwrap();

            let mut change = vec![];
            let mut contributed_resources = BTreeMap::new();
            for (resource_address, (mut vault, bucket)) in vaults_and_buckets.into_iter() {
                let divisibility = ResourceManager(resource_address).resource_type(api)
                    .map(|resource_type| {
                        if let ResourceType::Fungible { divisibility } = resource_type {
                            divisibility
                        } else {
                            panic!("Impossible case, we check for this in the constructor and have a test for this.")
                        }
                    })?;

                let amount_to_contribute = {
                    let amount_to_contribute = vault.amount(api)? * minimum_ratio;
                    if divisibility == 18 {
                        amount_to_contribute
                    } else {
                        amount_to_contribute
                            .round(divisibility as u32, RoundingMode::TowardsNegativeInfinity)
                    }
                };

                contributed_resources.insert(resource_address, amount_to_contribute);

                vault.put(bucket.take(amount_to_contribute, api)?, api)?;
                change.push(bucket)
            }

            let pool_units_to_mint = pool_unit_total_supply * minimum_ratio;

            Runtime::emit_event(
                api,
                ContributionEvent {
                    contributed_resources,
                    pool_units_minted: pool_units_to_mint,
                },
            )?;

            (
                substate
                    .pool_unit_resource_manager
                    .mint_fungible(pool_units_to_mint, api)?,
                change,
            )
        };

        api.field_lock_release(lock_handle)?;
        Ok((pool_units, change))
    }

    pub fn redeem<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<MultiResourcePoolRedeemOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (mut substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;

        // Ensure that the passed pool resources are indeed pool resources
        let bucket_resource_address = bucket.resource_address(api)?;
        if bucket_resource_address != substate.pool_unit_resource_manager.0 {
            return Err(MultiResourcePoolError::InvalidPoolUnitResource {
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
        let buckets = amounts_owed
            .into_iter()
            .map(|(resource_address, amount)| {
                substate
                    .vaults
                    .get_mut(&resource_address)
                    .unwrap()
                    .take(amount, api)
            })
            .collect::<Result<Vec<Bucket>, _>>()?;

        bucket.burn(api)?;
        api.field_lock_release(handle)?;

        Runtime::emit_event(api, event)?;

        Ok(buckets)
    }

    pub fn protected_deposit<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<MultiResourcePoolProtectedDepositOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (mut substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;
        let resource_address = bucket.resource_address(api)?;
        let vault = substate.vaults.get_mut(&resource_address);
        if let Some(vault) = vault {
            let event = DepositEvent {
                amount: bucket.amount(api)?,
                resource_address,
            };
            vault.put(bucket, api)?;
            api.field_lock_release(handle)?;
            Runtime::emit_event(api, event)?;
            Ok(())
        } else {
            Err(MultiResourcePoolError::ResourceDoesNotBelongToPool { resource_address }.into())
        }
    }

    pub fn protected_withdraw<Y>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<MultiResourcePoolProtectedWithdrawOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (mut substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;
        let vault = substate.vaults.get_mut(&resource_address);

        if let Some(vault) = vault {
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
            Err(MultiResourcePoolError::ResourceDoesNotBelongToPool { resource_address }.into())
        }
    }

    pub fn get_redemption_value<Y>(
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<MultiResourcePoolGetRedemptionValueOutput, RuntimeError>
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
    ) -> Result<MultiResourcePoolGetVaultAmountsOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (multi_resource_pool_substate, handle) =
            Self::lock_and_read(api, LockFlags::read_only())?;
        let amounts = multi_resource_pool_substate
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
    ) -> Result<(MultiResourcePoolSubstate, LockHandle), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = MultiResourcePoolField::MultiResourcePool.into();
        let handle = api.actor_open_field(OBJECT_HANDLE_SELF, substate_key, lock_flags)?;
        let multi_resource_pool = api.field_lock_read_typed(handle)?;

        Ok((multi_resource_pool, handle))
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
