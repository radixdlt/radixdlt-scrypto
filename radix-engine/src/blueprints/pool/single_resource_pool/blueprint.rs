use crate::blueprints::pool::single_resource_pool::*;
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

pub const SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT: &'static str = "SingleResourcePool";

pub struct SingleResourcePoolBlueprint;
impl SingleResourcePoolBlueprint {
    pub fn instantiate<Y>(
        resource_address: ResourceAddress,
        pool_manager_rule: AccessRule,
        api: &mut Y,
    ) -> Result<SingleResourcePoolInstantiateOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi,
    {
        // Validate that the resource is a fungible resource - a pool can't be created with non
        // fungible resources.
        let resource_manager = ResourceManager(resource_address);
        if let ResourceType::NonFungible { .. } = resource_manager.resource_type(api)? {
            Err(SingleResourcePoolError::NonFungibleResourcesAreNotAccepted { resource_address })?
        }

        // Allocating the component address of the pool - this will be used later for the component
        // caller badge.
        let node_id = api.kernel_allocate_node_id(EntityType::GlobalSingleResourcePool)?;
        let address = GlobalAddress::new_or_panic(node_id.0);

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

        let access_rules = AccessRules::create(roles(pool_manager_rule), api)?.0;
        // TODO: The following fields must ALL be LOCKED. No entity with any authority should be
        // able to update them later on.
        let metadata = Metadata::create_with_data(
            btreemap!(
                "pool_vault_number".into() => MetadataValue::U8(1),
                "pool_resources".into() => MetadataValue::GlobalAddress(resource_address.into()),
                "pool_unit".into() => MetadataValue::GlobalAddress(pool_unit_resource_manager.0.into()),
            ),
            api,
        )?;
        let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;
        let object_id = {
            let vault = Vault::create(resource_address, api)?;
            let substate = SingleResourcePoolSubstate {
                vault,
                pool_unit_resource_manager,
            };
            api.new_simple_object(
                SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT,
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
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<SingleResourcePoolContributeOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // No check that the bucket is of the same resource as the vault. This check will be handled
        // by the vault itself on deposit.

        let (substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;
        let mut pool_unit_resource_manager = substate.pool_unit_resource_manager;
        let mut vault = substate.vault;

        if bucket.is_empty(api)? {
            return Err(SingleResourcePoolError::ContributionOfEmptyBucketError.into());
        }

        /*
        There are four states that the pool could be in at this point of time depending on the total
        supply of the pool units and the the total amount of reserves that the pool unit has. We can
        examine each of those states.

        Let PU denote the total supply of pool units where 0 means that none exists and 1 means that
        some amount exists. Let R denote the total amount of reserves that the pool has where 0 here
        means that no reserves exist in the pool and 1 means that some reserves exist in the pool.

        PU  R
        0   0 => This is a new pool - no pool units and no pool reserves.
        0   1 => This is a pool which has been used but has dried out and all of the pool units have
                 been burned. The first contribution to this pool gets whatever dust is left behind.
        1   0 => This is an illegal state! Some amount of people own some % of zero which is invalid
        1   1 => The pool is in normal operations.

        Thus depending on the supply of these resources the pool behaves differently.
         */

        let reserves = vault.amount(api)?;
        let pool_unit_total_supply = pool_unit_resource_manager.total_supply(api)?;
        let amount_of_contributed_resources = bucket.amount(api)?;

        let pool_units_to_mint = match (
            pool_unit_total_supply > Decimal::ZERO,
            reserves > Decimal::ZERO,
        ) {
            (false, false) => Ok(amount_of_contributed_resources),
            (false, true) => Ok(amount_of_contributed_resources + reserves),
            (true, false) => Err(SingleResourcePoolError::NonZeroPoolUnitSupplyButZeroReserves),
            (true, true) => Ok(amount_of_contributed_resources * pool_unit_total_supply / reserves),
        }?;

        vault.put(bucket, api)?;
        let pool_units = pool_unit_resource_manager.mint_fungible(pool_units_to_mint, api)?;

        api.field_lock_release(handle)?;

        Runtime::emit_event(
            api,
            ContributionEvent {
                amount_of_resources_contributed: amount_of_contributed_resources,
                pool_unit_tokens_minted: pool_units_to_mint,
            },
        )?;

        Ok(pool_units)
    }

    pub fn redeem<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<SingleResourcePoolRedeemOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (pool_unit_resource_manager, mut vault, handle) = {
            let (substate, lock_handle) = Self::lock_and_read(api, LockFlags::read_only())?;

            (
                substate.pool_unit_resource_manager,
                substate.vault,
                lock_handle,
            )
        };

        // Ensure that the passed pool resources are indeed pool resources
        let bucket_resource_address = bucket.resource_address(api)?;
        if bucket_resource_address != pool_unit_resource_manager.0 {
            return Err(SingleResourcePoolError::InvalidPoolUnitResource {
                expected: pool_unit_resource_manager.0,
                actual: bucket_resource_address,
            }
            .into());
        }

        // Calculating the amount owed based on the passed pool units.
        let pool_units_to_redeem = bucket.amount(api)?;
        let pool_units_total_supply = pool_unit_resource_manager.total_supply(api)?;
        let pool_resource_reserves = vault.amount(api)?;
        let pool_resource_divisibility = vault
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
            pool_units_total_supply,
            pool_resource_reserves,
            pool_resource_divisibility,
        );

        // Burn the pool units and take the owed resources from the bucket.
        bucket.burn(api)?;
        let owed_resources = vault.take(amount_owed, api)?;

        api.field_lock_release(handle)?;

        Runtime::emit_event(
            api,
            RedemptionEvent {
                pool_unit_tokens_redeemed: pool_units_to_redeem,
                redeemed_amount: amount_owed,
            },
        )?;

        Ok(owed_resources)
    }

    pub fn protected_deposit<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<SingleResourcePoolProtectedDepositOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (mut substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;

        let event = DepositEvent {
            amount: bucket.amount(api)?,
        };

        substate.vault.put(bucket, api)?;
        api.field_lock_release(handle)?;

        Runtime::emit_event(api, event)?;

        Ok(())
    }

    pub fn protected_withdraw<Y>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<SingleResourcePoolProtectedWithdrawOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (mut substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;

        let bucket = substate.vault.take(amount, api)?;
        api.field_lock_release(handle)?;

        Runtime::emit_event(api, WithdrawEvent { amount })?;

        Ok(bucket)
    }

    pub fn get_redemption_value<Y>(
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<SingleResourcePoolGetRedemptionValueOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (pool_unit_resource_manager, vault, handle) = {
            let (substate, lock_handle) = Self::lock_and_read(api, LockFlags::read_only())?;

            (
                substate.pool_unit_resource_manager,
                substate.vault,
                lock_handle,
            )
        };

        let pool_units_to_redeem = amount_of_pool_units;
        let pool_units_total_supply = pool_unit_resource_manager.total_supply(api)?;
        let pool_resource_reserves = vault.amount(api)?;
        let pool_resource_divisibility = vault
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
            pool_units_total_supply,
            pool_resource_reserves,
            pool_resource_divisibility,
        );

        api.field_lock_release(handle)?;

        Ok(amount_owed)
    }

    pub fn get_vault_amount<Y>(
        api: &mut Y,
    ) -> Result<SingleResourcePoolGetVaultAmountOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (substate, handle) = Self::lock_and_read(api, LockFlags::read_only())?;
        let amount = substate.vault.amount(api)?;
        api.field_lock_release(handle)?;
        Ok(amount)
    }

    //===================
    // Utility Functions
    //===================

    fn calculate_amount_owed(
        pool_units_to_redeem: Decimal,
        pool_units_total_supply: Decimal,
        pool_resource_reserves: Decimal,
        pool_resource_divisibility: u8,
    ) -> Decimal {
        let amount_owed = pool_units_to_redeem * pool_resource_reserves / pool_units_total_supply;

        if pool_resource_divisibility == 18 {
            amount_owed
        } else {
            amount_owed.round(
                pool_resource_divisibility as u32,
                RoundingMode::TowardsNegativeInfinity,
            )
        }
    }

    fn lock_and_read<Y>(
        api: &mut Y,
        lock_flags: LockFlags,
    ) -> Result<(SingleResourcePoolSubstate, LockHandle), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = SingleResourcePoolField::SingleResourcePool.into();
        let handle = api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, lock_flags)?;
        let substate = api.field_lock_read_typed::<SingleResourcePoolSubstate>(handle)?;

        Ok((substate, handle))
    }
}

fn roles(pool_manager_rule: AccessRule) -> Roles {
    roles2! {
        POOL_MANAGER_ROLE => pool_manager_rule, mut [POOL_MANAGER_ROLE]
    }
}
