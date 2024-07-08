// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

mod blueprint_interface;
pub use blueprint_interface::*;

use std::cmp::*;

use common::prelude::*;
use ports_interface::prelude::*;
use scrypto::prelude::*;
use scrypto_interface::*;

macro_rules! define_error {
    (
        $(
            $name: ident => $item: expr;
        )*
    ) => {
        $(
            pub const $name: &'static str = concat!("[Ociswap v1 Adapter v1]", " ", $item);
        )*
    };
}

define_error! {
    FAILED_TO_GET_RESOURCE_ADDRESSES_ERROR
        => "Failed to get resource addresses - unexpected error.";
    FAILED_TO_GET_VAULT_ERROR
        => "Failed to get vault - unexpected error.";
    PRICE_IS_UNDEFINED
        => "Price is undefined.";
    FAILED_TO_CALCULATE_K_VALUE_OF_POOL_ERROR
        => "Failed to calculate the K value of the pool.";
    OVERFLOW_ERROR => "Calculation overflowed.";
    INVALID_NUMBER_OF_BUCKETS => "Invalid number of buckets.";
}

macro_rules! pool {
    ($address: expr) => {
        $crate::blueprint_interface::OciswapV1PoolInterfaceScryptoStub::from(
            $address,
        )
    };
}

#[blueprint_with_traits]
pub mod adapter {
    struct OciswapV1Adapter;

    impl OciswapV1Adapter {
        pub fn instantiate(
            _: AccessRule,
            _: AccessRule,
            metadata_init: MetadataInit,
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<OciswapV1Adapter> {
            let address_reservation =
                address_reservation.unwrap_or_else(|| {
                    Runtime::allocate_component_address(BlueprintId {
                        package_address: Runtime::package_address(),
                        blueprint_name: Runtime::blueprint_name(),
                    })
                    .0
                });

            Self {}
                .instantiate()
                .prepare_to_globalize(owner_role)
                .metadata(ModuleConfig {
                    init: metadata_init,
                    roles: Default::default(),
                })
                .with_address(address_reservation)
                .globalize()
        }

        pub fn liquidity_receipt_data(
            // Does not depend on state, this is kept in case this is required
            // in the future for whatever reason.
            &self,
            global_id: NonFungibleGlobalId,
        ) -> LiquidityReceipt<OciswapV1AdapterSpecificInformation> {
            // Read the non-fungible data.
            let LiquidityReceipt {
                name,
                lockup_period,
                pool_address,
                user_resource_address,
                user_contribution_amount,
                user_resource_volatility_classification,
                protocol_contribution_amount,
                maturity_date,
                adapter_specific_information,
            } = ResourceManager::from_address(global_id.resource_address())
                .get_non_fungible_data::<LiquidityReceipt<AnyValue>>(
                global_id.local_id(),
            );
            let adapter_specific_information = adapter_specific_information
                .as_typed::<OciswapV1AdapterSpecificInformation>()
                .unwrap();

            LiquidityReceipt {
                name,
                lockup_period,
                pool_address,
                user_resource_address,
                user_contribution_amount,
                user_resource_volatility_classification,
                protocol_contribution_amount,
                maturity_date,
                adapter_specific_information,
            }
        }
    }

    impl PoolAdapterInterfaceTrait for OciswapV1Adapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
            _: LockupPeriod,
        ) -> OpenLiquidityPositionOutput {
            let mut pool = pool!(pool_address);

            let (pool_units, change) = pool.add_liquidity(buckets.0, buckets.1);

            let user_share = pool_units
                .amount()
                .checked_div(
                    pool_units.resource_manager().total_supply().unwrap(),
                )
                .expect(OVERFLOW_ERROR);

            let pool_k = Global::<TwoResourcePool>::from(pool.liquidity_pool())
                .get_vault_amounts()
                .values()
                .map(|item| PreciseDecimal::from(*item))
                .reduce(|acc, item| {
                    acc.checked_mul(item).expect(OVERFLOW_ERROR)
                })
                .expect(FAILED_TO_CALCULATE_K_VALUE_OF_POOL_ERROR);

            OpenLiquidityPositionOutput {
                pool_units: IndexedBuckets::from_bucket(pool_units),
                change: change
                    .map(IndexedBuckets::from_bucket)
                    .unwrap_or_default(),
                others: Default::default(),
                adapter_specific_information:
                    OciswapV1AdapterSpecificInformation {
                        user_share_in_pool_when_position_opened: user_share,
                        pool_k_when_position_opened: pool_k,
                    }
                    .into(),
            }
        }

        /// Closes the liquidity position and calculates the amount of fees
        /// earned on the position while it was opened.
        ///
        /// On the fees calculation, this method is incapable of finding the
        /// exact amount of fees earned, just an approximation of how much the
        /// fees could be. The basis of this calculation is that by finding the
        /// amount of X and Y assets we expect to get at some desired price (the
        /// output amount due to price action) we can deduce the fees by
        /// subtracting the actual amount from the expected amount.
        ///
        /// For an xy = k AMM we have the following equations:
        ///
        /// (1)     xy = k
        /// (2)     y/x = p
        ///
        /// From (2) y can be represented in terms of p and x where it becomes
        /// y = px. Plugging that into (1) we get that:
        ///
        /// (3)     xpx = k
        ///         x^2 = k/p
        ///         x   = sqrt(k/p)
        ///
        /// Once X is found we can find y by plugging in the equation of x.
        ///
        /// (4)     y = px
        ///         y = p * sqrt(k / p)
        ///
        /// With equations (3) and (4), we now have a way to calculate the
        /// amount of x and y resources in the pool at some price _p_ and some
        /// pool coefficient value _k_. The amount that the user gets back when
        /// they close their liquidity position is the multiplication of the
        /// above by the user share _s_ which is 0 <= s <= 1.
        ///
        /// Therefore, the amount that the user is owed is equal to:
        ///
        /// (6)     x_owed = s * sqrt(k/p)
        /// (7)     y_owed = s * p * sqrt(k/p)
        ///
        /// To find the amount of x and y we expect to get back due to price
        /// action alone we use:
        ///
        /// 1. `s` the share of the user in the pool when the position was first
        /// opened.
        /// 2. `p` the final price.
        /// 3. `k` the pool coffieicnet when the position was first opened.
        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            mut pool_units: Vec<Bucket>,
            adapter_specific_information: AnyValue,
        ) -> CloseLiquidityPositionOutput {
            let mut pool = pool!(pool_address);

            let pool_units = {
                let pool_units_bucket =
                    pool_units.pop().expect(INVALID_NUMBER_OF_BUCKETS);
                if !pool_units.is_empty() {
                    panic!("{}", INVALID_NUMBER_OF_BUCKETS)
                }
                pool_units_bucket
            };

            let (bucket1, bucket2) = pool.remove_liquidity(pool_units);

            // Calculating the fees.
            let fees = {
                let indexed_buckets = [&bucket1, &bucket2]
                    .into_iter()
                    .map(|bucket| Bucket(bucket.0))
                    .map(|bucket| (bucket.resource_address(), bucket))
                    .collect::<IndexMap<_, _>>();

                let OciswapV1AdapterSpecificInformation {
                    pool_k_when_position_opened,
                    user_share_in_pool_when_position_opened,
                } = adapter_specific_information
                    .as_typed::<OciswapV1AdapterSpecificInformation>()
                    .unwrap();

                let price = self.price(pool_address);

                let sqrt_k_div_p = pool_k_when_position_opened
                    .checked_div(price.price)
                    .and_then(|value| value.checked_sqrt())
                    .expect(OVERFLOW_ERROR);

                let predicted_amount_x = sqrt_k_div_p
                    .checked_mul(user_share_in_pool_when_position_opened)
                    .and_then(|value| Decimal::try_from(value).ok())
                    .expect(OVERFLOW_ERROR);
                let predicted_amount_y = predicted_amount_x
                    .checked_mul(price.price)
                    .expect(OVERFLOW_ERROR);

                let fees_x = max(
                    indexed_buckets
                        .get(&price.base)
                        .map(|bucket| bucket.amount())
                        .unwrap_or(Decimal::ZERO)
                        .checked_sub(predicted_amount_x)
                        .unwrap_or(Decimal::ZERO),
                    Decimal::ZERO,
                );
                let fees_y = max(
                    indexed_buckets
                        .get(&price.quote)
                        .map(|bucket| bucket.amount())
                        .unwrap_or(Decimal::ZERO)
                        .checked_sub(predicted_amount_y)
                        .unwrap_or(Decimal::ZERO),
                    Decimal::ZERO,
                );

                indexmap! {
                    price.base => fees_x,
                    price.quote => fees_y
                }
            };

            CloseLiquidityPositionOutput {
                resources: IndexedBuckets::from_buckets([bucket1, bucket2]),
                others: Default::default(),
                fees,
            }
        }

        fn price(&mut self, pool_address: ComponentAddress) -> Price {
            let pool = pool!(pool_address);
            let pool = Global::<TwoResourcePool>::from(pool.liquidity_pool());
            let vault_amounts = pool.get_vault_amounts();

            let (resource_address1, resource_address2) =
                self.resource_addresses(pool_address);
            let amount1 = *vault_amounts
                .get(&resource_address1)
                .expect(FAILED_TO_GET_VAULT_ERROR);
            let amount2 = *vault_amounts
                .get(&resource_address2)
                .expect(FAILED_TO_GET_VAULT_ERROR);

            Price {
                base: resource_address1,
                quote: resource_address2,
                price: amount2.checked_div(amount1).expect(PRICE_IS_UNDEFINED),
            }
        }

        fn resource_addresses(
            &mut self,
            pool_address: ComponentAddress,
        ) -> (ResourceAddress, ResourceAddress) {
            let pool = pool!(pool_address);
            let pool = Global::<TwoResourcePool>::from(pool.liquidity_pool());
            let mut keys = pool.get_vault_amounts().into_keys();

            let resource_address1 =
                keys.next().expect(FAILED_TO_GET_RESOURCE_ADDRESSES_ERROR);
            let resource_address2 =
                keys.next().expect(FAILED_TO_GET_RESOURCE_ADDRESSES_ERROR);

            (resource_address1, resource_address2)
        }
    }
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct OciswapV1AdapterSpecificInformation {
    /// The share of the user in the pool when the position was opened.
    pub user_share_in_pool_when_position_opened: Decimal,

    /// The value of the K of the pool when the position was opened.
    pub pool_k_when_position_opened: PreciseDecimal,
}

impl From<OciswapV1AdapterSpecificInformation> for AnyValue {
    fn from(value: OciswapV1AdapterSpecificInformation) -> Self {
        AnyValue::from_typed(&value).unwrap()
    }
}
