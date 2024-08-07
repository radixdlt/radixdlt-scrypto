use radix_engine::blueprints::pool::v1::constants::*;
use scrypto_test::prelude::*;
use std::ops::Deref;

pub const MINT_LIMIT: Decimal = dec!(5708990770823839524233143877.797980545530986496);

pub fn with_multi_resource_pool<const N: usize, F, O>(divisibility: [u8; N], callback: F) -> O
where
    F: FnOnce(
        &mut TestEnvironment<InMemorySubstateDatabase>,
        [(Cloneable<FungibleBucket>, ResourceAddress); N],
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

impl Clone for Cloneable<FungibleBucket> {
    fn clone(&self) -> Self {
        Self(FungibleBucket(Bucket(self.0 .0 .0)))
    }
}

pub struct OneResourcePool(NodeId);

impl OneResourcePool {
    pub fn instantiate<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        typed_call_function::<OneResourcePoolInstantiateOutput>(
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

    pub fn contribute<Y: SystemApi<RuntimeError>>(
        &mut self,
        bucket: FungibleBucket,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let bucket = bucket.into();
        typed_call_method::<OneResourcePoolContributeOutput>(
            &self.0,
            ONE_RESOURCE_POOL_CONTRIBUTE_IDENT,
            &OneResourcePoolContributeInput { bucket },
            api,
        )
    }

    pub fn protected_deposit<Y: SystemApi<RuntimeError>>(
        &mut self,
        bucket: FungibleBucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let bucket = bucket.into();
        typed_call_method::<OneResourcePoolProtectedDepositOutput>(
            &self.0,
            ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
            &OneResourcePoolProtectedDepositInput { bucket },
            api,
        )
    }

    pub fn protected_withdraw<Y: SystemApi<RuntimeError>>(
        &mut self,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        typed_call_method::<OneResourcePoolProtectedWithdrawOutput>(
            &self.0,
            ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
            &OneResourcePoolProtectedWithdrawInput {
                amount,
                withdraw_strategy,
            },
            api,
        )
    }

    pub fn get_redemption_value<Y: SystemApi<RuntimeError>>(
        &self,
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<OneResourcePoolGetRedemptionValueOutput, RuntimeError> {
        typed_call_method(
            &self.0,
            ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
            &OneResourcePoolGetRedemptionValueInput {
                amount_of_pool_units,
            },
            api,
        )
    }

    pub fn redeem<Y: SystemApi<RuntimeError>>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<OneResourcePoolRedeemOutput, RuntimeError> {
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
    pub fn instantiate<Y: SystemApi<RuntimeError>>(
        resource_addresses: (ResourceAddress, ResourceAddress),
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        typed_call_function::<TwoResourcePoolInstantiateOutput>(
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

    pub fn contribute<Y: SystemApi<RuntimeError>>(
        &mut self,
        buckets: (FungibleBucket, FungibleBucket),
        api: &mut Y,
    ) -> Result<TwoResourcePoolContributeOutput, RuntimeError> {
        typed_call_method(
            &self.0,
            TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
            &TwoResourcePoolContributeInput {
                buckets: (buckets.0.into(), buckets.1.into()),
            },
            api,
        )
    }

    pub fn protected_deposit<Y: SystemApi<RuntimeError>>(
        &mut self,
        bucket: FungibleBucket,
        api: &mut Y,
    ) -> Result<TwoResourcePoolProtectedDepositOutput, RuntimeError> {
        typed_call_method(
            &self.0,
            TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
            &TwoResourcePoolProtectedDepositInput {
                bucket: bucket.into(),
            },
            api,
        )
    }

    pub fn protected_withdraw<Y: SystemApi<RuntimeError>>(
        &mut self,
        resource_address: ResourceAddress,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        typed_call_method::<TwoResourcePoolProtectedWithdrawOutput>(
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

    pub fn get_redemption_value<Y: SystemApi<RuntimeError>>(
        &self,
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<TwoResourcePoolGetRedemptionValueOutput, RuntimeError> {
        typed_call_method(
            &self.0,
            TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
            &TwoResourcePoolGetRedemptionValueInput {
                amount_of_pool_units,
            },
            api,
        )
    }

    pub fn redeem<Y: SystemApi<RuntimeError>>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<TwoResourcePoolRedeemOutput, RuntimeError> {
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
    pub fn instantiate<Y: SystemApi<RuntimeError>>(
        resource_addresses: [ResourceAddress; N],
        owner_role: OwnerRole,
        pool_manager_rule: AccessRule,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        typed_call_function::<MultiResourcePoolInstantiateOutput>(
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

    pub fn contribute<Y: SystemApi<RuntimeError>>(
        &mut self,
        buckets: [FungibleBucket; N],
        api: &mut Y,
    ) -> Result<MultiResourcePoolContributeOutput, RuntimeError> {
        let buckets = buckets.into_iter().map(|b| b.into()).collect();
        typed_call_method(
            &self.0,
            MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
            &MultiResourcePoolContributeInput { buckets },
            api,
        )
    }

    pub fn protected_deposit<Y: SystemApi<RuntimeError>>(
        &mut self,
        bucket: FungibleBucket,
        api: &mut Y,
    ) -> Result<MultiResourcePoolProtectedDepositOutput, RuntimeError> {
        let bucket = bucket.into();
        typed_call_method(
            &self.0,
            MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
            &MultiResourcePoolProtectedDepositInput { bucket },
            api,
        )
    }

    pub fn protected_withdraw<Y: SystemApi<RuntimeError>>(
        &mut self,
        resource_address: ResourceAddress,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        typed_call_method::<MultiResourcePoolProtectedWithdrawOutput>(
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

    pub fn get_redemption_value<Y: SystemApi<RuntimeError>>(
        &self,
        amount_of_pool_units: Decimal,
        api: &mut Y,
    ) -> Result<MultiResourcePoolGetRedemptionValueOutput, RuntimeError> {
        typed_call_method(
            &self.0,
            MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
            &MultiResourcePoolGetRedemptionValueInput {
                amount_of_pool_units,
            },
            api,
        )
    }

    pub fn redeem<Y: SystemApi<RuntimeError>>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<[Bucket; N], RuntimeError> {
        typed_call_method::<MultiResourcePoolRedeemOutput>(
            &self.0,
            MULTI_RESOURCE_POOL_REDEEM_IDENT,
            &MultiResourcePoolRedeemInput { bucket },
            api,
        )
        .map(|item| item.try_into().unwrap())
    }
}

fn typed_call_function<O: ScryptoDecode>(
    package_address: PackageAddress,
    blueprint_name: &str,
    function_name: &str,
    input: impl ScryptoEncode,
    api: &mut impl SystemApi<RuntimeError>,
) -> Result<O, RuntimeError> {
    api.call_function(
        package_address,
        blueprint_name,
        function_name,
        scrypto_encode(&input).unwrap(),
    )
    .map(|rtn| scrypto_decode(&rtn).unwrap())
}

fn typed_call_method<O: ScryptoDecode>(
    address: &NodeId,
    method_name: &str,
    input: impl ScryptoEncode,
    api: &mut impl SystemApi<RuntimeError>,
) -> Result<O, RuntimeError> {
    api.call_method(address, method_name, scrypto_encode(&input).unwrap())
        .map(|rtn| scrypto_decode(&rtn).unwrap())
}
