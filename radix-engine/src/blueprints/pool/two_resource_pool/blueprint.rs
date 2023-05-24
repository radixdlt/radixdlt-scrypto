use crate::blueprints::pool::two_resource_pool::*;
use crate::errors::*;
use crate::kernel::kernel_api::*;
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
        (_resource_address_1, _resource_address_2): (ResourceAddress, ResourceAddress),
        _pool_manager_rule: AccessRule,
        _api: &mut Y,
    ) -> Result<TwoResourcePoolInstantiateOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi,
    {
        todo!()
    }

    pub fn contribute<Y>(
        _buckets: (Bucket, Bucket),
        _api: &mut Y,
    ) -> Result<TwoResourcePoolContributeOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn redeem<Y>(
        _bucket: Bucket,
        _api: &mut Y,
    ) -> Result<TwoResourcePoolRedeemOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn protected_deposit<Y>(
        _bucket: Bucket,
        _api: &mut Y,
    ) -> Result<TwoResourcePoolProtectedDepositOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn protected_withdraw<Y>(
        _resource_address: ResourceAddress,
        _amount: Decimal,
        _api: &mut Y,
    ) -> Result<TwoResourcePoolProtectedWithdrawOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn get_redemption_value<Y>(
        _amount_of_pool_units: Decimal,
        _api: &mut Y,
    ) -> Result<TwoResourcePoolGetRedemptionValueOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn get_vault_amounts<Y>(
        _api: &mut Y,
    ) -> Result<TwoResourcePoolGetVaultAmountsOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    //===================
    // Utility Functions
    //===================

    fn _lock_and_read<Y>(
        _api: &mut Y,
        _lock_flags: LockFlags,
    ) -> Result<(TwoResourcePoolSubstate, LockHandle), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }
}

fn _authority_rules(pool_manager_rule: AccessRule) -> AuthorityRules {
    let mut authority_rules = AuthorityRules::new();
    /*
    FIXME: When we have a way to map methods to authorities I would like to:

    pool_manager_authority => [
        TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
        TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
        TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
    ]
    public => all else
     */
    authority_rules.set_main_authority_rule(
        TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
        pool_manager_rule.clone(),
        pool_manager_rule.clone(),
    );
    authority_rules.set_main_authority_rule(
        TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
        pool_manager_rule.clone(),
        pool_manager_rule.clone(),
    );
    authority_rules.set_main_authority_rule(
        TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
        pool_manager_rule.clone(),
        pool_manager_rule.clone(),
    );

    authority_rules.set_main_authority_rule(
        TWO_RESOURCE_POOL_REDEEM_IDENT,
        rule!(allow_all),
        rule!(allow_all),
    );
    authority_rules.set_main_authority_rule(
        TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
        rule!(allow_all),
        rule!(allow_all),
    );
    authority_rules.set_main_authority_rule(
        TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT,
        rule!(allow_all),
        rule!(allow_all),
    );

    authority_rules
}
