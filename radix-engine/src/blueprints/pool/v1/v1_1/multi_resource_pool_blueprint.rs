use crate::blueprints::pool::v1::constants::*;
use crate::blueprints::pool::v1::errors::multi_resource_pool::*;
use crate::blueprints::pool::v1::events::multi_resource_pool::*;
use crate::blueprints::pool::v1::substates::multi_resource_pool::*;
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

pub struct MultiResourcePoolBlueprint;
impl MultiResourcePoolBlueprint {
    pub fn instantiate<Y: SystemApi<RuntimeError>>(
        resource_addresses: IndexSet<ResourceAddress>,
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<MultiResourcePoolInstantiateOutput, RuntimeError> {
        // A pool can't be created where one of the resources is non-fungible - error out if any of
        // them are
        for resource_address in resource_addresses.iter() {
            let resource_manager = ResourceManager(*resource_address);
            if let ResourceType::NonFungible { .. } = resource_manager.resource_type(api)? {
                return Err(Error::NonFungibleResourcesAreNotAccepted {
                    resource_address: *resource_address,
                }
                .into());
            }
        }

        // A multi-resource pool can not be created with no resources - at minimum there should be
        // one resource.
        if resource_addresses.is_empty() {
            return Err(Error::CantCreatePoolWithLessThanOneResource.into());
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
                    blueprint_name: MULTI_RESOURCE_POOL_BLUEPRINT_IDENT.to_string(),
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
                "pool_vault_number" => resource_addresses.len() as u64, locked;
                "pool_resources" => resource_addresses.iter().cloned().map(GlobalAddress::from).collect::<Vec<_>>(), locked;
                "pool_unit" => GlobalAddress::from(pool_unit_resource_manager.0), locked;
            },
            api,
        )?;
        let royalty = ComponentRoyalty::create(ComponentRoyaltyConfig::default(), api)?;
        let object_id = {
            let substate = Substate {
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
                indexmap! {
                    MultiResourcePoolField::State.field_index() => FieldValue::new(&MultiResourcePoolStateFieldPayload::from_content_source(substate)),
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

    /**
    This function calculates the amount of resources that should be contributed to the pool and then
    contributes them to the pool returning back a pool unit resource in exchange for the contributed
    resources.

    The various states that the pool can be in can be outlined within the context laid out in the
    docs of the [`TwoResourcePool::contribute`] function, but in a more generalized way.

    * State 1: If no pool units exist in circulation then consider the pool to be new. All of the
    resources are accepted and an amount of pool units equivalent to the geometric mean of the non
    zero contributions is minted.
    * State 2: If pool units exist in circulation but all of the reserves are empty then this pool
    is in an invalid state, one that requires external intervention from a protected withdraw or
    deposit to get out of. This has to do with how we represent the fact that the user owns 100%
    of the pool.
    * State 3: If pool units exist in circulation and some but not all of the reserves are empty
    then contributions will be accepted according to the ratio of resources in the pool and the
    contribution of any of the resources with zero reserves will be returned as change.
    * State 4: Pool units exist in circulation and none of the vaults are empty, the pool is in
    normal operation.

    In the case when the pool is operating normally an algorithm is needed to determine the
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

    [`TwoResourcePool::contribute`]: super::TwoResourcePoolBlueprint::contribute
    */
    pub fn contribute<Y: SystemApi<RuntimeError>>(
        buckets: Vec<Bucket>,
        api: &mut Y,
    ) -> Result<MultiResourcePoolContributeOutput, RuntimeError> {
        Self::with_state(api, |mut substate, api| {
            let pool_unit_total_supply = substate
                .pool_unit_resource_manager
                .total_supply(api)?
                .expect("Total supply is always enabled for pool unit resource.");
            let pool_unit_total_supply = PreciseDecimal::from(pool_unit_total_supply);

            let contribution_information = {
                let mut information = substate
                    .vaults
                    .iter()
                    .map(|(resource_address, vault)| -> Result<_, RuntimeError> {
                        Ok((
                            *resource_address,
                            ContributionInformation {
                                resource_address: *resource_address,
                                vault: Vault(vault.0),
                                bucket: Bucket::create(*resource_address, api)?,
                                reserves: vault.amount(api)?.into(),
                                contribution: PreciseDecimal::ZERO,
                            },
                        ))
                    })
                    .collect::<Result<IndexMap<_, _>, _>>()?;

                for bucket in buckets {
                    let resource_address = bucket.resource_address(api)?;
                    if let Some(information) = information.get_mut(&resource_address) {
                        information.contribution = information
                            .contribution
                            .checked_add(bucket.amount(api)?)
                            .ok_or(Error::DecimalOverflowError)?;
                        information.bucket.put(bucket, api)?;
                    } else {
                        return Err(Error::ResourceDoesNotBelongToPool { resource_address }.into());
                    }
                }
                information
            };

            // New Pool
            let mut contributed_resources = index_map_new::<ResourceAddress, Decimal>();
            let (pool_units_to_mint, change_buckets) = if pool_unit_total_supply.is_zero() {
                // Calculating the pool units to mint through the geometric mean.
                let pool_units_to_mint = {
                    let contributions = contribution_information
                        .values()
                        .filter_map(|information| {
                            if information.contribution.is_zero() {
                                None
                            } else {
                                Some(information.contribution)
                            }
                        })
                        .collect::<Vec<_>>();
                    let root_order = contributions.len();

                    // Pool Units to Mint = Geometric Average = root(n, c1 * c2 * ... * cn) Where
                    // n is the number of non-zero contributions and cn is any non-zero contribution
                    contributions
                        .into_iter()
                        .try_fold(PreciseDecimal::ONE, |accumulator, value| {
                            value
                                .checked_nth_root(root_order as u32)
                                .and_then(|value| value.checked_mul(accumulator))
                        })
                        .and_then(|value| value.checked_round(18, RoundingMode::ToPositiveInfinity))
                        .ok_or(Error::DecimalOverflowError)?
                };

                for mut information in contribution_information.into_values() {
                    let amount = information.bucket.amount(api)?;
                    if !amount.is_zero() {
                        let entry = contributed_resources
                            .entry(information.resource_address)
                            .or_default();
                        *entry = entry
                            .checked_add(amount)
                            .ok_or(Error::DecimalOverflowError)?;
                    }

                    information.vault.put(information.bucket, api)?;
                }

                (pool_units_to_mint, Vec::default())
            }
            // Not a new Pool
            else {
                // Calculate the minimum ratio. It is possible for this ratio to be zero. This can
                // happen in a number of different cases including very small contributions (e.g.
                // 1 atto) with any amount of reserves in the pool. In cases like this, we will just
                // end up minting zero pool units and catching that later on.
                let minimum_ratio = contribution_information
                    .values()
                    .filter_map(|information| {
                        if !information.reserves.is_zero() {
                            // There is a possibility that this can overflow if the reserves is not
                            // zero but is very close to zero (1 atto for example). However, this is
                            // fine since this would then just return a `None` and would not be one
                            // of the ratios that we look at to determine the minimum ratio.
                            information.contribution.checked_div(information.reserves)
                        } else {
                            None
                        }
                    })
                    .min()
                    .ok_or(Error::NoMinimumRatio)?;

                // Deposit the buckets into the vaults and then return the change buckets
                let change_buckets = contribution_information
                    .into_values()
                    .map(|mut information| -> Result<_, RuntimeError> {
                        let amount_to_contribute = information
                            .reserves
                            .checked_mul(minimum_ratio)
                            .and_then(|value| Decimal::try_from(value).ok())
                            .ok_or(Error::DecimalOverflowError)?;
                        let bucket_to_contribute = information.bucket.take_advanced(
                            amount_to_contribute,
                            WithdrawStrategy::Rounded(RoundingMode::ToNegativeInfinity),
                            api,
                        )?;
                        let amount_to_contribute = bucket_to_contribute.amount(api)?;
                        if amount_to_contribute == Decimal::ZERO
                            && information.reserves != PreciseDecimal::ZERO
                        {
                            return Err(Error::LargerContributionRequiredToMeetRatio.into());
                        }

                        information.vault.put(bucket_to_contribute, api)?;

                        let entry = contributed_resources
                            .entry(information.resource_address)
                            .or_default();
                        *entry = entry
                            .checked_add(amount_to_contribute)
                            .ok_or(Error::DecimalOverflowError)?;

                        let is_empty = information.bucket.is_empty(api)?;
                        if is_empty {
                            Bucket(information.bucket.0).drop_empty(api)?;
                        }

                        Ok((is_empty, information.bucket))
                    })
                    .filter_map(|result| match result {
                        Ok((is_empty, bucket)) => {
                            if is_empty {
                                None
                            } else {
                                Some(Ok(bucket))
                            }
                        }
                        Err(error) => Some(Err(error)),
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                (
                    pool_unit_total_supply
                        .checked_mul(minimum_ratio)
                        .ok_or(Error::DecimalOverflowError)?,
                    change_buckets,
                )
            };
            let pool_units_to_mint =
                Decimal::try_from(pool_units_to_mint).map_err(|_| Error::DecimalOverflowError)?;
            if pool_units_to_mint.is_zero() {
                return Err(Error::ZeroPoolUnitsMinted.into());
            }

            let pool_units = substate
                .pool_unit_resource_manager
                .mint_fungible(pool_units_to_mint, api)?;

            Runtime::emit_event(
                api,
                ContributionEvent {
                    contributed_resources,
                    pool_units_minted: pool_units_to_mint,
                },
            )?;

            Ok((pool_units.into(), change_buckets))
        })
    }

    pub fn redeem<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<MultiResourcePoolRedeemOutput, RuntimeError> {
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

            // The following part does some unwraps and panic-able operations but should never panic
            amounts_owed
                .into_iter()
                .map(|(resource_address, amount)| {
                    substate
                        .vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take(amount, api)
                })
                .collect::<Result<Vec<Bucket>, _>>()
        })
    }

    pub fn protected_deposit<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<MultiResourcePoolProtectedDepositOutput, RuntimeError> {
        Self::with_state(api, |mut substate, api| {
            let resource_address = bucket.resource_address(api)?;
            let vault = substate.vaults.get_mut(&resource_address);
            if let Some(vault) = vault {
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
    ) -> Result<MultiResourcePoolProtectedWithdrawOutput, RuntimeError> {
        Self::with_state(api, |mut substate, api| {
            let vault = substate.vaults.get_mut(&resource_address);

            if let Some(vault) = vault {
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
    ) -> Result<MultiResourcePoolGetRedemptionValueOutput, RuntimeError> {
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
        let substate_key = MultiResourcePoolField::State.into();
        let handle =
            api.actor_open_field(ACTOR_STATE_SELF, substate_key, LockFlags::read_only())?;
        let substate = api
            .field_read_typed::<VersionedMultiResourcePoolState>(handle)?
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

#[derive(Debug)]
struct ContributionInformation {
    /// The address of the resource.
    pub resource_address: ResourceAddress,
    /// The vault containing the reserves.
    pub vault: Vault,
    /// The bucket of the tokens the user wishes to contribute. Might not be contributed in full.
    pub bucket: Bucket,
    /// The amount of reserves in the vault.
    pub reserves: PreciseDecimal,
    /// The amount of resources the user wishes to contribute.
    pub contribution: PreciseDecimal,
}
