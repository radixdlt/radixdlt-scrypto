use crate::errors::*;
use radix_engine_common::math::*;
use radix_engine_common::types::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

pub const SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT: &'static str = "SingleResourcePool";

pub struct SingleResourcePoolBlueprint;
impl SingleResourcePoolBlueprint {
    pub fn instantiate<Y>(
        _resource_address: ResourceAddress,
        _api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn instantiate_with_owner_rule<Y>(
        _resource_address: ResourceAddress,
        _owner_rule: AccessRule,
        _api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn contribute<Y>(_bucket: Bucket, _api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn redeem<Y>(_bucket: Bucket, _api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn protected_deposit<Y>(_bucket: Bucket, _api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn protected_withdraw<Y>(_amount: Decimal, _api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn get_redemption_value<Y>(
        _amount_of_pool_units: Decimal,
        _api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn get_vault_amount<Y>(_api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }
}

fn authority_rules(pool_manager_rule: AccessRule) -> AuthorityRules {
    let mut authority_rules = AuthorityRules::new();
    /*
    FIXME: When we have a way to map methods to authorities I would like to:

    pool_manager_authority => [
        SINGLE_RESOURCE_POOL_CONTRIBUTE_IDENT,
        SINGLE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
        SINGLE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
    ]
    public => all else
     */
    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_CONTRIBUTE_IDENT,
        pool_manager_rule.clone(),
        pool_manager_rule.clone(),
    );
    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
        pool_manager_rule.clone(),
        pool_manager_rule.clone(),
    );
    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
        pool_manager_rule.clone(),
        pool_manager_rule.clone(),
    );

    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_REDEEM_IDENT,
        rule!(allow_all),
        rule!(allow_all),
    );
    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
        rule!(allow_all),
        rule!(allow_all),
    );
    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_GET_VAULT_AMOUNT_IDENT,
        rule!(allow_all),
        rule!(allow_all),
    );

    authority_rules
}
