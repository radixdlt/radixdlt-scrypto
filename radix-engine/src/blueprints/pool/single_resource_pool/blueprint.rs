use crate::errors::*;
use radix_engine_common::math::*;
use radix_engine_common::types::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;

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
