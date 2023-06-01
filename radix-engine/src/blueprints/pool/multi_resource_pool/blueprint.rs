use crate::blueprints::pool::multi_resource_pool::*;
use crate::blueprints::pool::POOL_MANAGER_ROLE;
use crate::errors::*;
use crate::kernel::kernel_api::*;
use native_sdk::modules::access_rules::*;
use native_sdk::modules::metadata::*;
use native_sdk::modules::royalty::*;
use native_sdk::resource::*;
use native_sdk::runtime::Runtime;
use radix_engine_common::math::*;
use radix_engine_common::prelude::*;
use radix_engine_interface::api::node_modules::metadata::*;
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

        // Allocating the address of the pool - this is going to be needed for the metadata of the
        // pool unit resource.
        let address = {
            let node_id = api.kernel_allocate_node_id(EntityType::GlobalMultiResourcePool)?;
            GlobalAddress::new_or_panic(node_id.0)
        };

        // Creating the pool unit resource
        let pool_unit_resource_manager = {
            let component_caller_badge = NonFungibleGlobalId::global_caller_badge(address);

            let access_rules = btreemap!(
                Mint => (
                    rule!(require(component_caller_badge.clone())),
                    AccessRule::DenyAll,
                ),
                Burn => (rule!(require(component_caller_badge)), AccessRule::DenyAll),
                Recall => (AccessRule::DenyAll, AccessRule::DenyAll)
            );

            // TODO: Pool unit resource metadata - one things is needed to do this:
            // 1- A fix for the issue with references so that we can have the component address of
            //    the pool component in the metadata of the pool unit resource (currently results in
            //    an error because we're passing a reference to a node that doesn't exist).

            ResourceManager::new_fungible(18, Default::default(), access_rules, api)?
        };

        // Creating the pool nodes
        let access_rules = AccessRules::create(roles(pool_manager_rule), api)?.0;
        // TODO: The following fields must ALL be LOCKED. No entity with any authority should be
        // able to update them later on.
        let metadata = Metadata::create_with_data(
            btreemap!(
                "pool_vault_number".into() => MetadataValue::U8(2),
                "pool_resources".into() => MetadataValue::GlobalAddressArray(resource_addresses.iter().cloned().map(Into::into).collect()),
                "pool_unit".into() => MetadataValue::GlobalAddress(pool_unit_resource_manager.0.into()),
            ),
            api,
        )?;
        let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;
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

        api.globalize_with_address(
            btreemap!(
                ObjectModuleId::Main => object_id,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            address,
        )?;

        Ok(ComponentAddress::new_or_panic(address.as_node_id().0))
    }

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

        let pool_unit_total_supply = substate.pool_unit_resource_manager.total_supply(api)?;
        // Case: New Pool
        let (pool_units, change) = if pool_unit_total_supply.is_zero() {
            let pool_units_to_mint = amounts_of_resources_provided
                .values()
                .copied()
                .reduce(|acc, item| acc * item)
                .and_then(|value| value.sqrt())
                .unwrap();

            for bucket in buckets {
                let bucket_resource_address = bucket.resource_address(api)?;
                substate
                    .vaults
                    .get_mut(&bucket_resource_address)
                    .unwrap()
                    .put(bucket, api)?;
            }

            (
                substate
                    .pool_unit_resource_manager
                    .mint_fungible(pool_units_to_mint, api)?,
                Vec::<Bucket>::new(),
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
                let vault = substate.vaults.get(&bucket_resource_address).map_or(
                    Err(MultiResourcePoolError::ResourceDoesNotBelongToPool {
                        resource_address: bucket_resource_address,
                    }),
                    |vault| Ok(Vault(vault.0.clone())),
                )?;

                if let Some((_, store_bucket)) =
                    vaults_and_buckets.get_mut(&bucket_resource_address)
                {
                    store_bucket.put(bucket, api)?;
                } else {
                    vaults_and_buckets.insert(bucket_resource_address, (vault, bucket));
                };
            }

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
                .expect("Can't be empty");

            let mut change = vec![];
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

                vault.put(bucket.take(amount_to_contribute, api)?, api)?;
                change.push(bucket)
            }

            let pool_units_to_mint = pool_unit_total_supply * minimum_ratio;
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
        let pool_units_total_supply = substate.pool_unit_resource_manager.total_supply(api)?;
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
        let pool_units_total_supply = substate.pool_unit_resource_manager.total_supply(api)?;
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
        let handle = api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, lock_flags)?;
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

fn roles(pool_manager_rule: AccessRule) -> Roles {
    roles2! {
        POOL_MANAGER_ROLE => pool_manager_rule, mut [POOL_MANAGER_ROLE]
    }
}
