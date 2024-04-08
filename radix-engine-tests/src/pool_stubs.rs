use core::ops::*;
use std::ops::Deref;

use radix_common::*;
use radix_common::constants::*;
use radix_common::data::scrypto::*;
use radix_common::math::*;
use radix_common::types::*;
use radix_engine::blueprints::pool::v1::constants::*;
use radix_engine::errors::*;
use radix_engine_interface::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::prelude::*;
use radix_native_sdk::resource::*;
use radix_substate_store_impls::memory_db::*;
use sbor::prelude::*;
use scrypto_test::environment::*;
use scrypto_test::sdk::*;

pub const MINT_LIMIT: Decimal = dec!(5708990770823839524233143877.797980545530986496);

pub fn with_multi_resource_pool<const N: usize, F, O>(divisibility: [u8; N], callback: F) -> O
where
    F: FnOnce(
        &mut TestEnvironment<InMemorySubstateDatabase>,
        [(Cloneable<Bucket>, ResourceAddress); N],
        MultiResourcePool<N>,
    ) -> O,
{
    let env = &mut TestEnvironment::new();
    let array = divisibility.map(|divisibility| {
        let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
            .divisibility(divisibility)
            .mint_initial_supply(
                MINT_LIMIT
                    .checked_round(divisibility, RoundingMode::ToZero)
                    .unwrap(),
                env,
            )
            .map(Cloneable)
            .unwrap();
        let resource_address = bucket.resource_address(env).unwrap();
        (bucket, resource_address)
    });
    let resource_addresses = array.clone().map(|(_, resource_address)| resource_address);
    let multi_resource_pool = MultiResourcePool::instantiate(
        resource_addresses,
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )
    .unwrap();
    callback(env, array, multi_resource_pool)
}

pub struct Cloneable<T>(pub T);

impl<T> From<T> for Cloneable<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Cloneable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for Cloneable<Bucket> {
    fn clone(&self) -> Self {
        Self(Bucket(self.0 .0))
    }
}

pub struct OneResourcePool(NodeId);

impl OneResourcePool {
    pub fn instantiate<Y>(
        resource_address: ResourceAddress,
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<Self, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_function::<_, _, OneResourcePoolInstantiateOutput>(
            POOL_PACKAGE,
            ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
            ONE_RESOURCE_POOL_INSTANTIATE_IDENT,
            OneResourcePoolInstantiateInput {
                resource_address,
                owner_role,
                pool_manager_rule,
                address_reservation,
            },
            api,
        )
        .map(|rtn| Self(rtn.0.into_node_id()))
    }

    pub fn contribute<Y>(&mut self, bucket: Bucket, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method::<_, _, OneResourcePoolContributeOutput>(
            &self.0,
            ONE_RESOURCE_POOL_CONTRIBUTE_IDENT,
            &OneResourcePoolContributeInput { bucket },
            api,
        )
    }

    pub fn protected_deposit<Y>(&mut self, bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method::<_, _, OneResourcePoolProtectedDepositOutput>(
            &self.0,
            ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
            &OneResourcePoolProtectedDepositInput { bucket },
            api,
        )
    }

    pub fn protected_withdraw<Y>(
        &mut self,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method::<_, _, OneResourcePoolProtectedWithdrawOutput>(
            &self.0,
            ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
            &OneResourcePoolProtectedWithdrawInput {
                amount,
                withdraw_strategy,
            },
            api,
        )
    }

    pub fn get_redemption_value<Y>(
        &self,
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<OneResourcePoolGetRedemptionValueOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method(
            &self.0,
            ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
            &OneResourcePoolGetRedemptionValueInput {
                amount_of_pool_units,
            },
            api,
        )
    }

    pub fn redeem<Y>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<OneResourcePoolRedeemOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method(
            &self.0,
            ONE_RESOURCE_POOL_REDEEM_IDENT,
            &OneResourcePoolRedeemInput { bucket },
            api,
        )
    }
}

pub struct TwoResourcePool(NodeId);

impl TwoResourcePool {
    pub fn instantiate<Y>(
        resource_addresses: (ResourceAddress, ResourceAddress),
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<Self, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_function::<_, _, TwoResourcePoolInstantiateOutput>(
            POOL_PACKAGE,
            TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
            TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
            TwoResourcePoolInstantiateInput {
                resource_addresses,
                owner_role,
                pool_manager_rule,
                address_reservation,
            },
            api,
        )
        .map(|rtn| Self(rtn.0.into_node_id()))
    }

    pub fn contribute<Y>(
        &mut self,
        buckets: (Bucket, Bucket),
        api: &mut Y,
    ) -> Result<TwoResourcePoolContributeOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method(
            &self.0,
            TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
            &TwoResourcePoolContributeInput { buckets },
            api,
        )
    }

    pub fn protected_deposit<Y>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<TwoResourcePoolProtectedDepositOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method(
            &self.0,
            TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
            &TwoResourcePoolProtectedDepositInput { bucket },
            api,
        )
    }

    pub fn protected_withdraw<Y>(
        &mut self,
        resource_address: ResourceAddress,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method::<_, _, TwoResourcePoolProtectedWithdrawOutput>(
            &self.0,
            TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
            &TwoResourcePoolProtectedWithdrawInput {
                resource_address,
                amount,
                withdraw_strategy,
            },
            api,
        )
    }

    pub fn get_redemption_value<Y>(
        &self,
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<TwoResourcePoolGetRedemptionValueOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method(
            &self.0,
            TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
            &TwoResourcePoolGetRedemptionValueInput {
                amount_of_pool_units,
            },
            api,
        )
    }

    pub fn redeem<Y>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<TwoResourcePoolRedeemOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method(
            &self.0,
            TWO_RESOURCE_POOL_REDEEM_IDENT,
            &TwoResourcePoolRedeemInput { bucket },
            api,
        )
    }
}

pub struct MultiResourcePool<const N: usize>(NodeId);

impl<const N: usize> MultiResourcePool<N> {
    pub fn instantiate<Y>(
        resource_addresses: [ResourceAddress; N],
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<Self, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_function::<_, _, MultiResourcePoolInstantiateOutput>(
            POOL_PACKAGE,
            MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
            MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
            MultiResourcePoolInstantiateInput {
                resource_addresses: resource_addresses.into_iter().collect(),
                owner_role,
                pool_manager_rule,
                address_reservation,
            },
            api,
        )
        .map(|rtn| Self(rtn.0.into_node_id()))
    }

    pub fn contribute<Y>(
        &mut self,
        buckets: [Bucket; N],
        api: &mut Y,
    ) -> Result<MultiResourcePoolContributeOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method(
            &self.0,
            MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
            &MultiResourcePoolContributeInput {
                buckets: buckets.into(),
            },
            api,
        )
    }

    pub fn protected_deposit<Y>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<MultiResourcePoolProtectedDepositOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method(
            &self.0,
            MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
            &MultiResourcePoolProtectedDepositInput { bucket },
            api,
        )
    }

    pub fn protected_withdraw<Y>(
        &mut self,
        resource_address: ResourceAddress,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method::<_, _, MultiResourcePoolProtectedWithdrawOutput>(
            &self.0,
            MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
            &MultiResourcePoolProtectedWithdrawInput {
                resource_address,
                amount,
                withdraw_strategy,
            },
            api,
        )
    }

    pub fn get_redemption_value<Y>(
        &self,
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<MultiResourcePoolGetRedemptionValueOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method(
            &self.0,
            MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
            &MultiResourcePoolGetRedemptionValueInput {
                amount_of_pool_units,
            },
            api,
        )
    }

    pub fn redeem<Y>(&mut self, bucket: Bucket, api: &mut Y) -> Result<[Bucket; N], RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        typed_call_method::<_, _, MultiResourcePoolRedeemOutput>(
            &self.0,
            MULTI_RESOURCE_POOL_REDEEM_IDENT,
            &MultiResourcePoolRedeemInput { bucket },
            api,
        )
        .map(|item| item.try_into().unwrap())
    }
}

fn typed_call_function<Y, I, O>(
    package_address: PackageAddress,
    blueprint_name: &str,
    function_name: &str,
    input: I,
    api: &mut Y,
) -> Result<O, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
    I: ScryptoEncode,
    O: ScryptoDecode,
{
    api.call_function(
        package_address,
        blueprint_name,
        function_name,
        scrypto_encode(&input).unwrap(),
    )
    .map(|rtn| scrypto_decode::<O>(&rtn).unwrap())
}

fn typed_call_method<Y, I, O>(
    address: &NodeId,
    method_name: &str,
    input: I,
    api: &mut Y,
) -> Result<O, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
    I: ScryptoEncode,
    O: ScryptoDecode,
{
    api.call_method(address, method_name, scrypto_encode(&input).unwrap())
        .map(|rtn| scrypto_decode::<O>(&rtn).unwrap())
}
