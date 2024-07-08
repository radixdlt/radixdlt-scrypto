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
            pub const $name: &'static str = concat!("[DefiPlaza v2 Adapter v1]", " ", $item);
        )*
    };
}

define_error! {
    RESOURCE_DOESNT_BELONG_TO_POOL => "Resources don't belong to pool";
    OVERFLOW_ERROR => "Calculation overflowed.";
    UNEXPECTED_ERROR => "Unexpected Error.";
    INVALID_NUMBER_OF_BUCKETS => "Invalid number of buckets.";
    NO_PAIR_CONFIG => "The pair config of the provided pool is not known.";
    TARGET_RATIO_EXCEEDS_MAXIMUM => "The target ratio exceeds the allowed maximum";
}

macro_rules! pool {
    ($address: expr) => {
        $crate::blueprint_interface::DefiPlazaV2PoolInterfaceScryptoStub::from(
            $address,
        )
    };
}

#[blueprint_with_traits]
#[types(ComponentAddress, PairConfig)]
pub mod adapter {
    enable_method_auth! {
        roles {
            protocol_owner => updatable_by: [protocol_owner];
            protocol_manager => updatable_by: [protocol_manager, protocol_owner];
        },
        methods {
            add_pair_configs => restrict_to: [protocol_manager, protocol_owner];
            set_maximum_allowed_target_ratio => restrict_to: [
                protocol_manager,
                protocol_owner
            ];
            /* User methods */
            price => PUBLIC;
            resource_addresses => PUBLIC;
            liquidity_receipt_data => PUBLIC;
            open_liquidity_position => PUBLIC;
            close_liquidity_position => PUBLIC;
        }
    }

    struct DefiPlazaV2Adapter {
        /// The pair config of the various pools is constant but there is no
        /// getter function that can be used to get it on ledger. As such, the
        /// protocol owner or manager must submit this information to the
        /// adapter for its operation. This does not change, so, once set we
        /// do not expect to remove it again.
        pair_config: KeyValueStore<ComponentAddress, PairConfig>,

        /// There is a limit on the target ratio that we support in Defiplaza.
        /// This is because a larger target ratio means that the value in each
        /// pool would be further and further away from 50%/50%. Therefore, we
        /// limit it.
        max_allowed_target_ratio: Decimal,
    }

    impl DefiPlazaV2Adapter {
        pub fn instantiate(
            protocol_manager_rule: AccessRule,
            protocol_owner_rule: AccessRule,
            metadata_init: MetadataInit,
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<DefiPlazaV2Adapter> {
            let address_reservation =
                address_reservation.unwrap_or_else(|| {
                    Runtime::allocate_component_address(BlueprintId {
                        package_address: Runtime::package_address(),
                        blueprint_name: Runtime::blueprint_name(),
                    })
                    .0
                });

            Self {
                pair_config: KeyValueStore::new_with_registered_type(),
                max_allowed_target_ratio: dec!(1.5),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .metadata(ModuleConfig {
                init: metadata_init,
                roles: Default::default(),
            })
            .roles(roles! {
                protocol_manager => protocol_manager_rule;
                protocol_owner => protocol_owner_rule;
            })
            .with_address(address_reservation)
            .globalize()
        }

        pub fn add_pair_configs(
            &mut self,
            pair_config: IndexMap<ComponentAddress, PairConfig>,
        ) {
            for (address, config) in pair_config.into_iter() {
                self.pair_config.insert(address, config);
            }
        }

        pub fn set_maximum_allowed_target_ratio(
            &mut self,
            target_ratio: Decimal,
        ) {
            self.max_allowed_target_ratio = target_ratio;
        }

        pub fn liquidity_receipt_data(
            // Does not depend on state, this is kept in case this is required
            // in the future for whatever reason.
            &self,
            global_id: NonFungibleGlobalId,
        ) -> LiquidityReceipt<DefiPlazaV2AdapterSpecificInformation> {
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
                .as_typed::<DefiPlazaV2AdapterSpecificInformation>()
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

    impl PoolAdapterInterfaceTrait for DefiPlazaV2Adapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
            _: LockupPeriod,
        ) -> OpenLiquidityPositionOutput {
            // When opening a liquidity position we follow the algorithm that
            // Jazzer described to us:
            //
            // 1) state = pair.get_state()
            // 2) see which token is in shortage by inspecting state.shortage
            // 3) store lp1_original_target = state.target_ratio * bucket1.amount()
            //    where bucket1 is the token in shortage.
            // 4) (lp1, remainder_bucket) = pair.add_liquidity(bucket1, bucket2)
            //    and store the resulting lp1 tokens
            // 5) store lp2_original_target = remainder_bucket.amount()
            // 6) call (lp2, remainder2) =
            //    pair.add_liquidity(remainder_bucket.expect(), None) and store
            //    the resulting lp2 tokens (remainder2 will be None)

            let mut pool = pool!(pool_address);
            let (base_resource_address, quote_resource_address) =
                pool.get_tokens();

            // Ensure that the passed buckets belong to the pool and sort them
            // into base and quote buckets.
            let (base_bucket, quote_bucket) = {
                let bucket_address1 = buckets.0.resource_address();
                let bucket_address2 = buckets.1.resource_address();

                if bucket_address1 == base_resource_address
                    && bucket_address2 == quote_resource_address
                {
                    (buckets.0, buckets.1)
                } else if bucket_address2 == base_resource_address
                    && bucket_address1 == quote_resource_address
                {
                    (buckets.1, buckets.0)
                } else {
                    panic!("{}", RESOURCE_DOESNT_BELONG_TO_POOL)
                }
            };

            // Step 1: Get the pair's state
            let pair_state = pool.get_state();

            // Ensure that the pool's target ratio does not exceed the maximum
            // allowed.
            assert!(
                pair_state.target_ratio <= self.max_allowed_target_ratio,
                "{}",
                TARGET_RATIO_EXCEEDS_MAXIMUM
            );

            // Step 2: Determine which of the resources is in shortage. The one
            // in shortage is the one that we will be contributing first to the
            // pool. If the pool is in equilibrium then we can pick any of the
            // two resources as the first (shortage) resource. In the code here
            // "first" and "second" refer to which one will be contributed first
            // and which will be contributed second.
            let shortage = pair_state.shortage;
            let shortage_state = ShortageState::from(shortage);

            let [(shortage_asset_resource_address, shortage_asset_bucket), (surplus_asset_resource_address, surplus_asset_bucket)] =
                match shortage_state {
                    ShortageState::Equilibrium => [
                        (base_resource_address, base_bucket),
                        (quote_resource_address, quote_bucket),
                    ],
                    ShortageState::Shortage(Asset::Base) => [
                        (base_resource_address, base_bucket),
                        (quote_resource_address, quote_bucket),
                    ],
                    ShortageState::Shortage(Asset::Quote) => [
                        (quote_resource_address, quote_bucket),
                        (base_resource_address, base_bucket),
                    ],
                };

            // Step 3: Calculate tate.target_ratio * bucket1.amount() where
            // bucket1 is the bucket currently in shortage or the resource that
            // will be contributed first.
            let shortage_asset_original_target = pair_state
                .target_ratio
                .checked_mul(shortage_asset_bucket.amount())
                .expect(OVERFLOW_ERROR);

            // Step 4: Contribute to the pool. The first bucket to provide the
            // pool is the bucket of the asset in shortage or the asset that we
            // now refer to as "first" and then followed by the "second" bucket.
            //
            // In the case of equilibrium we do not contribute the second bucket
            // and instead just the first bucket.
            let (shortage_asset_pool_units, surplus_asset_change) =
                match shortage_state {
                    ShortageState::Equilibrium => (
                        pool.add_liquidity(shortage_asset_bucket, None).0,
                        Some(surplus_asset_bucket),
                    ),
                    ShortageState::Shortage(_) => pool.add_liquidity(
                        shortage_asset_bucket,
                        Some(surplus_asset_bucket),
                    ),
                };

            // Step 5: Calculate and store the original target of the second
            // liquidity position. This is calculated as the amount of assets
            // that are in the remainder (change) bucket.
            let surplus_asset_bucket =
                surplus_asset_change.expect(UNEXPECTED_ERROR);
            let surplus_asset_original_target = surplus_asset_bucket.amount();

            // Step 6: Add liquidity with the second resource & no co-liquidity.
            let (surplus_asset_pool_units, change) =
                pool.add_liquidity(surplus_asset_bucket, None);

            // We've been told that the change should be zero. Therefore, we
            // assert for it to make sure that everything is as we expect it
            // to be.
            assert_eq!(
                change
                    .as_ref()
                    .map(|bucket| bucket.amount())
                    .unwrap_or(Decimal::ZERO),
                Decimal::ZERO
            );

            // A sanity check to make sure that everything is correct. The pool
            // units obtained from the first contribution should be different
            // from those obtained in the second contribution.
            assert_ne!(
                shortage_asset_pool_units.resource_address(),
                surplus_asset_pool_units.resource_address(),
            );

            // The procedure for adding liquidity to the pool is now complete.
            // We can now construct the output.
            OpenLiquidityPositionOutput {
                pool_units: IndexedBuckets::from_buckets([
                    shortage_asset_pool_units,
                    surplus_asset_pool_units,
                ]),
                change: change.map(IndexedBuckets::from_bucket).unwrap_or_default(),
                others: vec![],
                adapter_specific_information: DefiPlazaV2AdapterSpecificInformation {
                    original_targets: indexmap! {
                        shortage_asset_resource_address => shortage_asset_original_target,
                        surplus_asset_resource_address => surplus_asset_original_target
                    },
                }
                .into(),
            }
        }

        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            mut pool_units: Vec<Bucket>,
            adapter_specific_information: AnyValue,
        ) -> CloseLiquidityPositionOutput {
            // When closing a position we follow the algorithm Jazzer described
            // to us:
            //
            // 1) state = pair.get_state()
            // 2) see which token is in shortage by inspecting state.shortage
            // 3) calculate new base target and quote target. Suppose base token
            //    is in shortage: new_base_target = state.target_ratio * current
            //    base tokens represented by base_LP. The new_quote_target will
            //    just be equal to the current quote tokens represented by
            //    quote_LP.
            // 4) base_fees = new_base_target - original_base_target
            // 5) quote_fees = new_quote_target - original_quote_target
            // 6) settle

            let pool = pool!(pool_address);

            let (pool_units1, pool_units2) = {
                let pool_units_bucket1 =
                    pool_units.pop().expect(INVALID_NUMBER_OF_BUCKETS);
                let pool_units_bucket2 =
                    pool_units.pop().expect(INVALID_NUMBER_OF_BUCKETS);
                if !pool_units.is_empty() {
                    panic!("{}", INVALID_NUMBER_OF_BUCKETS)
                }
                (pool_units_bucket1, pool_units_bucket2)
            };

            // Decoding the adapter specific information as the type we expect
            // it to be.
            let DefiPlazaV2AdapterSpecificInformation { original_targets } =
                adapter_specific_information.as_typed().unwrap();

            // We have gotten two pools units, one of the base pool and
            // another for the quote pool. We need to determine which is
            // which and overall split things according to shortage and
            // surplus. So, instead of referring to them as base and quote
            // we would figure out what is in shortage and what is surplus.

            // First thing we do is store the pool units in a map where the
            // key the address of the pool and the value is the pool units
            // bucket. We find the address of the pool through metadata on
            // the pool units since there is currently no other way to find
            // this information.
            let mut pool_component_to_pool_unit_mapping =
                [pool_units1, pool_units2]
                    .into_iter()
                    .map(|bucket| {
                        let resource_manager = bucket.resource_manager();
                        let pool_address = ComponentAddress::try_from(
                            resource_manager
                                .get_metadata::<_, GlobalAddress>("pool")
                                .unwrap()
                                .unwrap(),
                        )
                        .unwrap();
                        (pool_address, bucket)
                    })
                    .collect::<IndexMap<_, _>>();

            // With the way we combined them above we now want to split them
            // into base pool units and quote pool units. This is simple to
            // do, just get the address of the base and quote pools and then
            // do a simple `remove` from the above map.
            let (base_pool_component, quote_pool_component) = pool.get_pools();
            let base_pool_units = pool_component_to_pool_unit_mapping
                .swap_remove(&base_pool_component)
                .unwrap();
            let quote_pool_units = pool_component_to_pool_unit_mapping
                .swap_remove(&quote_pool_component)
                .unwrap();

            // At this point we have the the base and quote token addresses,
            // pool addresses, and pool units. We can now split things as
            // shortage and quote and stop referring to things as base and
            // quote.
            let (base_resource_address, quote_resource_address) =
                pool.get_tokens();
            let pair_state = pool.get_state();
            let (claimed_resources, fees) = match ShortageState::from(
                pair_state.shortage,
            ) {
                // The pool is in equilibrium, none of the assets are in
                // shortage so there is no need to multiply anything by the
                // target ratio.
                ShortageState::Equilibrium => {
                    // Claiming the assets from the pools.
                    let [resources_claimed_from_base_resource_pool, resources_claimed_from_quote_resource_pool] =
                        [
                            (base_pool_component, base_pool_units),
                            (quote_pool_component, quote_pool_units),
                        ]
                        .map(
                            |(pool_component_address, pool_units_bucket)| {
                                let mut pool = Global::<TwoResourcePool>::from(
                                    pool_component_address,
                                );
                                let (bucket1, bucket2) = pool
                                    .redeem(pool_units_bucket.as_fungible());
                                IndexedBuckets::from_buckets([bucket1, bucket2])
                            },
                        );

                    // The target of the two resources is just the amount we
                    // got back when closing the liquidity position.
                    let new_target_of_base_resource =
                        resources_claimed_from_base_resource_pool
                            .get(&base_resource_address)
                            .map(|bucket| bucket.amount())
                            .expect(UNEXPECTED_ERROR);
                    let new_target_of_quote_resource =
                        resources_claimed_from_quote_resource_pool
                            .get(&quote_resource_address)
                            .map(|bucket| bucket.amount())
                            .expect(UNEXPECTED_ERROR);

                    // Now that we have the target for the base and quote
                    // resources we can calculate the fees.
                    let base_resource_fees = original_targets
                        .get(&base_resource_address)
                        .expect(UNEXPECTED_ERROR)
                        .checked_sub(new_target_of_base_resource)
                        .expect(OVERFLOW_ERROR)
                        .max(dec!(0));
                    let quote_resource_fees = original_targets
                        .get(&quote_resource_address)
                        .expect(UNEXPECTED_ERROR)
                        .checked_sub(new_target_of_quote_resource)
                        .expect(OVERFLOW_ERROR)
                        .max(dec!(0));

                    let fees = indexmap! {
                        base_resource_address => base_resource_fees,
                        quote_resource_address => quote_resource_fees,
                    };

                    let claimed_resources =
                        resources_claimed_from_base_resource_pool.combine(
                            resources_claimed_from_quote_resource_pool,
                        );

                    (claimed_resources, fees)
                }
                // One of the assets is in shortage and the other is in
                // surplus. Determine which is which and sort the info.
                ShortageState::Shortage(asset) => {
                    let (
                        (
                            shortage_asset_pool_component,
                            shortage_asset_pool_units,
                            shortage_asset_resource_address,
                        ),
                        (
                            surplus_asset_pool_component,
                            surplus_asset_pool_units,
                            surplus_asset_resource_address,
                        ),
                    ) = match asset {
                        Asset::Base => (
                            (
                                base_pool_component,
                                base_pool_units,
                                base_resource_address,
                            ),
                            (
                                quote_pool_component,
                                quote_pool_units,
                                quote_resource_address,
                            ),
                        ),
                        Asset::Quote => (
                            (
                                quote_pool_component,
                                quote_pool_units,
                                quote_resource_address,
                            ),
                            (
                                base_pool_component,
                                base_pool_units,
                                base_resource_address,
                            ),
                        ),
                    };

                    // We have now split them into shortage and surplus and
                    // we can now close the liquidity positions and compute
                    // the new targets for the base and shortage assets.
                    let [resources_claimed_from_shortage_asset_pool, resources_claimed_from_surplus_asset_pool] =
                        [
                            (
                                shortage_asset_pool_component,
                                shortage_asset_pool_units,
                            ),
                            (
                                surplus_asset_pool_component,
                                surplus_asset_pool_units,
                            ),
                        ]
                        .map(
                            |(pool_component_address, pool_units_bucket)| {
                                let mut pool = Global::<TwoResourcePool>::from(
                                    pool_component_address,
                                );
                                let (bucket1, bucket2) = pool
                                    .redeem(pool_units_bucket.as_fungible());
                                IndexedBuckets::from_buckets([bucket1, bucket2])
                            },
                        );

                    // The target of the shortage asset can be calculated by
                    // multiplying the amount we got back from closing the
                    // position in the shortage pool by the target ratio of
                    // the pool in the current state.
                    let new_target_of_shortage_asset =
                        resources_claimed_from_shortage_asset_pool
                            .get(&shortage_asset_resource_address)
                            .map(|bucket| bucket.amount())
                            .expect(UNEXPECTED_ERROR)
                            .checked_mul(pair_state.target_ratio)
                            .expect(OVERFLOW_ERROR);

                    // The target of the surplus asset is simple, its the
                    // amount we got back when we closed the position in
                    // the surplus pool.
                    let new_target_of_surplus_asset =
                        resources_claimed_from_surplus_asset_pool
                            .get(&surplus_asset_resource_address)
                            .map(|bucket| bucket.amount())
                            .expect(UNEXPECTED_ERROR);

                    // Now that we have the target for the shortage and
                    // surplus assets we can calculate the fees earned on
                    // those assets. Its calculated by subtracting the
                    // new targets from the original targets.
                    let shortage_asset_fees = new_target_of_shortage_asset
                        .checked_sub(
                            original_targets
                                .get(&shortage_asset_resource_address)
                                .copied()
                                .expect(UNEXPECTED_ERROR),
                        )
                        .expect(OVERFLOW_ERROR)
                        .max(dec!(0));
                    let surplus_asset_fees = new_target_of_surplus_asset
                        .checked_sub(
                            original_targets
                                .get(&surplus_asset_resource_address)
                                .copied()
                                .expect(UNEXPECTED_ERROR),
                        )
                        .expect(OVERFLOW_ERROR)
                        .max(dec!(0));

                    let fees = indexmap! {
                        shortage_asset_resource_address => shortage_asset_fees,
                        surplus_asset_resource_address => surplus_asset_fees,
                    };

                    let claimed_resources =
                        resources_claimed_from_shortage_asset_pool
                            .combine(resources_claimed_from_surplus_asset_pool);

                    (claimed_resources, fees)
                }
            };

            CloseLiquidityPositionOutput {
                resources: claimed_resources,
                others: vec![],
                fees,
            }
        }

        fn price(&mut self, pool_address: ComponentAddress) -> Price {
            // In DefiPlaza there is no concept of a current pool price. Instead
            // there is a bid and ask kind of like an order book but they're not
            // one. The price is different depending on whether a given trade
            // would improve or worsen IL. We say that the current pool price is
            // the arithmetic mean of the bid and ask prices of the pool.
            let pool = pool!(pool_address);
            let (base_pool, quote_pool) = pool.get_pools();
            let (base_resource_address, quote_resource_address) =
                pool.get_tokens();
            let bid_ask = price_math::calculate_pair_prices(
                pool.get_state(),
                *self.pair_config.get(&pool_address).expect(NO_PAIR_CONFIG),
                Global::<TwoResourcePool>::from(base_pool),
                Global::<TwoResourcePool>::from(quote_pool),
            );

            let average_price = bid_ask
                .bid
                .checked_add(bid_ask.ask)
                .and_then(|value| value.checked_div(dec!(2)))
                .expect(OVERFLOW_ERROR);

            Price {
                base: base_resource_address,
                quote: quote_resource_address,
                price: average_price,
            }
        }

        fn resource_addresses(
            &mut self,
            pool_address: ComponentAddress,
        ) -> (ResourceAddress, ResourceAddress) {
            let pool = pool!(pool_address);
            let two_resource_pool =
                Global::<TwoResourcePool>::from(pool.get_pools().0);

            let mut resource_addresses =
                two_resource_pool.get_vault_amounts().into_keys();

            let resource_address1 =
                resource_addresses.next().expect(UNEXPECTED_ERROR);
            let resource_address2 =
                resource_addresses.next().expect(UNEXPECTED_ERROR);

            (resource_address1, resource_address2)
        }
    }
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct DefiPlazaV2AdapterSpecificInformation {
    pub original_targets: IndexMap<ResourceAddress, Decimal>,
}

impl From<DefiPlazaV2AdapterSpecificInformation> for AnyValue {
    fn from(value: DefiPlazaV2AdapterSpecificInformation) -> Self {
        AnyValue::from_typed(&value).unwrap()
    }
}

// The following functions are copied from the DefiPlaza repository (link:
// https://github.com/OmegaSyndicate/RadixPlaza) and have been slightly modified
// so that they're pure functions that require no state. The commit hash that
// is used here is `574acb12fef95d8040c449dce4d01cfc4115bd35`. DefiPlaza's
// source code is licensed under the MIT license which allows us to do such
// copies and modification of code.
//
// The `calculate_pair_prices` function is the entrypoint into the module and is
// the function to calculate the current bid and ask prices of the pairs.
#[allow(clippy::arithmetic_side_effects)]
mod price_math {
    use super::*;

    #[derive(
        ScryptoSbor,
        ManifestSbor,
        Copy,
        Clone,
        Debug,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
    )]
    pub struct PairPrices {
        pub bid: Decimal,
        pub ask: Decimal,
    }

    pub fn calculate_pair_prices(
        pair_state: PairState,
        pair_config: PairConfig,
        base_pool: Global<TwoResourcePool>,
        quote_pool: Global<TwoResourcePool>,
    ) -> PairPrices {
        let input_is_quote = false;

        // Check which pool we're workings with and extract relevant values
        let (pool, old_pref, _) =
            select_pool(pair_state, input_is_quote, base_pool, quote_pool);
        let (actual, surplus, shortfall) =
            assess_pool(pool, pair_state.target_ratio);

        // Compute time since previous trade and resulting decay factor for the
        // filter
        let t =
            Clock::current_time_rounded_to_minutes().seconds_since_unix_epoch;
        let delta_t = (t - pair_state.last_outgoing).max(0);
        let factor =
            Decimal::checked_powi(&pair_config.decay_factor, delta_t / 60)
                .unwrap();

        // Calculate the filtered reference price
        let p_ref_ss = match shortfall > Decimal::ZERO {
            true => calc_p0_from_curve(
                shortfall,
                surplus,
                pair_state.target_ratio,
                pair_config.k_in,
            ),
            false => old_pref,
        };
        let p_ref = factor * old_pref + (Decimal::ONE - factor) * p_ref_ss;

        let adjusted_target_ratio = match actual > Decimal::ZERO {
            true => calc_target_ratio(p_ref, actual, surplus, pair_config.k_in),
            false => Decimal::ZERO,
        };

        let last_outgoing_spot = match pool == base_pool {
            true => pair_state.last_out_spot,
            false => Decimal::ONE / pair_state.last_out_spot,
        };

        let incoming_spot =
            calc_spot(p_ref, adjusted_target_ratio, pair_config.k_in);
        let outgoing_spot = factor * last_outgoing_spot
            + (Decimal::ONE - factor) * incoming_spot;

        let bid = incoming_spot;
        let ask = outgoing_spot;

        // TODO: What to do at equilibrium?
        match pair_state.shortage {
            Shortage::Equilibrium | Shortage::BaseShortage => {
                PairPrices { bid, ask }
            }
            Shortage::QuoteShortage => PairPrices {
                bid: 1 / ask,
                ask: 1 / bid,
            },
        }
    }

    const MIN_K_IN: Decimal = dec!(0.001);

    fn select_pool(
        state: PairState,
        input_is_quote: bool,
        base_pool: Global<TwoResourcePool>,
        quote_pool: Global<TwoResourcePool>,
    ) -> (Global<TwoResourcePool>, Decimal, bool) {
        let p_ref = state.p0;
        let p_ref_inv = Decimal::ONE / p_ref;
        match (state.shortage, input_is_quote) {
            (Shortage::BaseShortage, true) => (base_pool, p_ref, false),
            (Shortage::BaseShortage, false) => (base_pool, p_ref, true),
            (Shortage::Equilibrium, true) => (base_pool, p_ref, false),
            (Shortage::Equilibrium, false) => (quote_pool, p_ref_inv, false),
            (Shortage::QuoteShortage, true) => (quote_pool, p_ref_inv, true),
            (Shortage::QuoteShortage, false) => (quote_pool, p_ref_inv, false),
        }
    }

    fn assess_pool(
        pool: Global<TwoResourcePool>,
        target_ratio: Decimal,
    ) -> (Decimal, Decimal, Decimal) {
        let reserves = pool.get_vault_amounts();
        let actual =
            *reserves.get_index(0).map(|(_addr, amount)| amount).unwrap();
        let surplus =
            *reserves.get_index(1).map(|(_addr, amount)| amount).unwrap();
        let shortfall = target_ratio * actual - actual;
        (actual, surplus, shortfall)
    }

    fn calc_p0_from_curve(
        shortfall: Decimal,
        surplus: Decimal,
        target_ratio: Decimal,
        k: Decimal,
    ) -> Decimal {
        assert!(shortfall > Decimal::ZERO, "Invalid shortfall");
        assert!(surplus > Decimal::ZERO, "Invalid surplus");
        assert!(target_ratio >= Decimal::ONE, "Invalid target ratio");
        assert!(k >= MIN_K_IN, "Invalid k");

        // Calculate the price at equilibrium (p0) using the given formula
        surplus / shortfall / (Decimal::ONE + k * (target_ratio - Decimal::ONE))
    }

    fn calc_spot(p0: Decimal, target_ratio: Decimal, k: Decimal) -> Decimal {
        assert!(p0 > Decimal::ZERO, "Invalid p0");
        assert!(target_ratio >= Decimal::ONE, "Invalid target ratio");
        assert!(k >= MIN_K_IN, "Invalid k");

        let ratio2 = target_ratio * target_ratio;
        (Decimal::ONE + k * (ratio2 - Decimal::ONE)) * p0
    }

    fn calc_target_ratio(
        p0: Decimal,
        actual: Decimal,
        surplus: Decimal,
        k: Decimal,
    ) -> Decimal {
        assert!(p0 > Decimal::ZERO, "Invalid p0");
        assert!(actual > Decimal::ZERO, "Invalid actual reserves");
        assert!(surplus >= Decimal::ZERO, "Invalid surplus amount");
        assert!(k >= MIN_K_IN, "Invalid k");

        let radicand = Decimal::ONE + dec!(4) * k * surplus / p0 / actual;
        let num = dec!(2) * k - Decimal::ONE + radicand.checked_sqrt().unwrap();
        num / k / dec!(2)
    }
}
