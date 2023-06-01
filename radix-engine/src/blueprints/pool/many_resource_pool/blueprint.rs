#![allow(unused)] // TODO: Remove

use crate::blueprints::pool::many_resource_pool::*;
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

pub const MANY_RESOURCE_POOL_BLUEPRINT_IDENT: &'static str = "ManyResourcePool";

pub struct ManyResourcePoolBlueprint;
impl ManyResourcePoolBlueprint {
    pub fn instantiate<Y>(
        resource_addresses: BTreeSet<ResourceAddress>,
        pool_manager_rule: AccessRule,
        api: &mut Y,
    ) -> Result<ManyResourcePoolInstantiateOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi,
    {
        todo!()
    }

    pub fn contribute<Y>(
        buckets: Vec<Bucket>,
        api: &mut Y,
    ) -> Result<ManyResourcePoolContributeOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn redeem<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<ManyResourcePoolRedeemOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn protected_deposit<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<ManyResourcePoolProtectedDepositOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn protected_withdraw<Y>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<ManyResourcePoolProtectedWithdrawOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn get_redemption_value<Y>(
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<ManyResourcePoolGetRedemptionValueOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn get_vault_amounts<Y>(
        api: &mut Y,
    ) -> Result<ManyResourcePoolGetVaultAmountsOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (many_resource_pool_substate, handle) =
            Self::lock_and_read(api, LockFlags::read_only())?;
        let amounts = many_resource_pool_substate
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
    ) -> Result<(ManyResourcePoolSubstate, LockHandle), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    fn calculate_amount_owed(
        pool_units_to_redeem: Decimal,
        pool_units_total_supply: Decimal,
        reserves: BTreeMap<ResourceAddress, ReserveResourceInformation>,
    ) -> BTreeMap<ResourceAddress, Decimal> {
        todo!()
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
