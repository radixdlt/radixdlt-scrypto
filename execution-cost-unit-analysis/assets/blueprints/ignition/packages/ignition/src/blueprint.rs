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

//! This module implements the main project ignition blueprint and protocol.
//!
//! In simple terms, project ignition allows for users to provide one side of
//! liquidity and for itself to provide the other side of the liquidity. The
//! protocol is not quite made to be profit-generating, its main purpose is to
//! incentivize people to provide liquidity by providing users with a number of
//! benefits:
//!
//! * User's contribution is doubled in value; Ignition will contribute the
//! other side of the liquidity.
//! * Users get some percentage of rewards upfront.
//! * Users have impermanent loss protection and in most cases are guaranteed
//! to withdraw out the same amount of tokens that they put in plus fees earned
//! on their position.
//!
//! This makes Ignition a perfect incentive for users who already own an amount
//! of some of the supported tokens and who wish to provide liquidity with very
//! low downside, upfront rewards, increased fees, and impermanent loss
//! protection.
//!
//! The user locks their tokens for some period of time allowed by the protocol
//! and based on that they get some amount of upfront rewards. The longer the
//! lockup period is, the higher the rewards are. When the period is over, the
//! protocol will try to provide the user with the same amount of tokens that
//! they put in plus any trading fees earned in the process (on their asset).
//! If that can't be given, then the protocol will try to provide the user of
//! as much of the protocol's asset as possible to make them whole in terms of
//! value.
//!
//! In Ignition, the term "protocol's asset" refers to the asset that Ignition
//! has and that the protocol is willing to lend out to users when they wish to
//! provide liquidity. The term "user asset" refers to the asset or resource
//! that was provided by the user. So, the protocol and user assets are the two
//! sides of the liquidity that go into a liquidity pool, which name is used
//! depends on their source: the protocol for the ledger's resource and the user
//! for the user's resource.
//!
//! An important thing to note is that the protocol asset can't be changed at
//! runtime after the component has been instantiated, it will be forever stuck
//! with that protocol's asset. The user assets can be added and removed by
//! adding and removing pools to the allowed pools list. In the case of the
//! official protocol deployment, the protocol's asset will be XRD and the
//! user's asset will be BTC, ETH, USDC, and USDT. However, Ignition is actually
//! general enough that it can be used by projects who would like to improve
//! their liquidity and who're willing to lose some tokens in the process.
//!
//! The protocol's blueprint is made to be quite modular and to allow for easy
//! upgrading if needed. This means that the protocol's assets can be withdrawn
//! by the protocol owner and that many of the external components that the
//! protocol relies on can be swapped at runtime with little trouble. As an
//! example, the protocol communicates with Dexes through adapters meaning that
//! additional Dexes can be supported by writing and registering new adapters to
//! the existing component on ledger and that support for dexes can be removed
//! by removing their adapter. Additionally, the oracle can be swapped and
//! changed at any point of time to a new oracle. Changing the oracle or the
//! adapters relies on the interface being the same, if the interface is
//! different then, unfortunately, there is no way for the protocol to check at
//! runtime but calls using the oracle or adapter would fail. Thus, changes must
//! be preceded by an interface check.
//!
//! Similarly, the reward rates are quite modular too and are added at runtime
//! and not baked into the blueprint itself allowing additional reward rates to
//! be added and for some reward rates to be removed.

#![allow(clippy::type_complexity)]

use crate::errors::*;
use common::prelude::*;
use ports_interface::prelude::*;
use scrypto::prelude::*;
use std::cmp::*;

type PoolAdapter = PoolAdapterInterfaceScryptoStub;
type OracleAdapter = OracleAdapterInterfaceScryptoStub;

#[blueprint]
#[types(
    Decimal,
    ResourceAddress,
    ComponentAddress,
    NonFungibleGlobalId,
    BlueprintId,
    Vault,
    Vec<Vault>,
    FungibleVault,
    LockupPeriod,
    Volatility,
    StoredPoolBlueprintInformation,
    IndexMap<ResourceAddress, Vault>,
)]
mod ignition {
    enable_method_auth! {
        roles {
            protocol_owner => updatable_by: [protocol_owner];
            protocol_manager => updatable_by: [protocol_manager, protocol_owner];
        },
        methods {
            set_oracle_adapter => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            set_pool_adapter => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            add_allowed_pool => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            remove_allowed_pool => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            set_liquidity_receipt => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            insert_pool_information => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            remove_pool_information => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            set_maximum_allowed_price_staleness_in_seconds => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            remove_reward_rate => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            add_reward_rate => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            set_is_open_position_enabled => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            set_is_close_position_enabled => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            set_maximum_allowed_price_difference_percentage => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            insert_user_resource_volatility => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            upsert_matching_factor => restrict_to: [protocol_owner];
            deposit_protocol_resources => restrict_to: [protocol_owner];
            withdraw_protocol_resources => restrict_to: [protocol_owner];
            deposit_user_resources => restrict_to: [protocol_owner];
            withdraw_user_resources => restrict_to: [protocol_owner];
            deposit_pool_units => restrict_to: [protocol_owner];
            withdraw_pool_units => restrict_to: [protocol_owner];
            forcefully_liquidate => restrict_to: [protocol_owner];
            /* User methods */
            open_liquidity_position => PUBLIC;
            close_liquidity_position => PUBLIC;
            /* Getters */
            get_user_resource_reserves_amount => PUBLIC;
            get_protocol_resource_reserves_amount => PUBLIC;
        }
    }

    struct Ignition {
        /// A reference to the resource manager of the protocol's resource.
        /// This is the resource that the protocol will be lending out
        /// to users who wish to provide liquidity. In other words,
        /// this is the one side of the liquidity that will be provided
        /// by the protocol and the other side must be provided by the
        /// user. This can't be changed after the component has been
        /// instantiated. Thus, it would be chosen with some caution.
        ///
        /// Even though Ignition will only be lending out XRD this information
        /// is kept dynamic instead of static to allow for easier testing of
        /// Ignition and to allow for it to be deployed and tested on testnets
        /// with mintable resources.
        protocol_resource: ResourceManager,

        /// The adapter of the oracle to use for the protocol. The oracle is
        /// expected to have a specific interface that is required by this
        /// blueprint. This adapter can be updated and changed at runtime to
        /// a new one whose underlying oracle is completely different. Thus
        /// we can switch between oracle providers at runtime by developing a
        /// new adapter for said oracle provider.
        oracle_adapter: OracleAdapter,

        /// Information about the pool blueprints, indexed by the id of the
        /// blueprint. This contains information about the adapter to use, the
        /// list of pools that contributions are allowed to, and a reference
        /// to the resource manager of the liquidity receipt. Everything about
        /// this is updatable. Entries can be added and removed, adapters can
        /// be changed, pools can be added or removed from the list of allowed
        /// pools, and liquidity receipt addresses can be changed.
        ///
        /// The mapping of the [`BlueprintId`] to the pool information means
        /// that each Dex, or at least Dex blueprint, has a single entry in the
        /// protocol.
        ///
        /// Note: it is well understood that [`StoredPoolBlueprintInformation`]
        /// data is unbounded in size and that it can lead to state explosion.
        /// But, we will only have four allowed pools in Ignition and therefore
        /// we are not worried about the state explosion problems. Additionally,
        /// using a [`KeyValueStore`] there would mean that the pool information
        /// entires can not be removed or replaced from the map due to the fact
        /// that a kv-store can't be dropped. Therefore, a regular [`IndexMap`]
        /// is used and its guaranteed that there will only be four pools there.
        pool_information:
            KeyValueStore<BlueprintId, StoredPoolBlueprintInformation>,

        /// Maps a resource address to its volatility classification in the
        /// protocol. This is used to store whether a resource is considered to
        /// be volatile or non-volatile to then determine which vault of the
        /// protocol resources to use when matching the contributions.
        ///
        /// This is not quite meant to be an allow or deny list so the only
        /// operation that is allowed to this KVStore is an upsert, removals
        /// are not allowed.
        user_resource_volatility: KeyValueStore<ResourceAddress, Volatility>,

        /* Vaults */
        /// The reserves of the ignition protocol resource where they are split
        /// by the resources to use for volatile assets and the ones to use for
        /// non-volatile assets.
        protocol_resource_reserves: ProtocolResourceReserves,

        /// A key value store of all of the vaults of ignition that contain the
        /// user resources. These vaults do not need to be funded with anything
        /// for the protocol to run, they're primarily used by the protocol to
        /// deposit some of the user assets obtained when closing liquidity
        /// positions. `protocol_resource_reserves` stores the protocol assets
        /// required for the protocol operations. Only the owner of the
        /// protocol is allowed to deposit and withdraw from these
        /// vaults.
        user_resources_vaults: KeyValueStore<ResourceAddress, FungibleVault>,

        /// The vaults storing the pool units and liquidity receipts obtained
        /// from providing the liquidity. It is indexed by the non-fungible
        /// global id of the liquidity receipt non-fungible token minted by
        /// the protocol when liquidity is provided. Only the owner of the
        /// protocol is allowed to deposit or withdraw into these vaults.
        ///
        /// The value of the map is another map which maps the address of the
        /// pool unit to a vault containing this pool unit. A map is used since
        /// not all exchanges work one pool units per contributions. Some of
        /// them require two or more.
        ///
        /// Note: it is understood that the use of [`IndexMap`] here can make
        /// the application vulnerable to state explosion. However, we chose to
        /// continue using it as the size of this will realistically always be
        /// 1 in the case of most exchanges and 2 in the case of DefiPlaza. It
        /// should not be any more than that. There is realistically no way for
        /// this map to have more than two items.
        pool_units: KeyValueStore<
            NonFungibleGlobalId,
            IndexMap<ResourceAddress, Vault>,
        >,

        /// A KeyValueStore that stores all of the tokens owed to users of the
        /// protocol whose liquidity claims have been forcefully liquidated.
        ///
        /// Note: It is understood that the value type used here can in theory
        /// lead to state explosion. However, realistically, there would only
        /// be 2 vaults in here since the pools we're interacting with are all
        /// of two resources. Perhaps there would be a third if some of the
        /// DEXs has an incentive program. However, it is very unlikely
        /// that there would be more than that.
        forced_liquidation_claims:
            KeyValueStore<NonFungibleGlobalId, Vec<Vault>>,

        /// A map that stores the _matching factor_ for each pool which is a
        /// [`Decimal`] between 0 and 1 that controls how much of the user's
        /// contribution is matched by Ignition. For a given pool, if the user
        /// provides X worth of a user resource and if the pool has a Y%
        /// matching factor then the amount of protocol resources provided is
        /// X • Y%.
        matching_factor: KeyValueStore<ComponentAddress, Decimal>,

        /* Configuration */
        /// The upfront reward rates supported by the protocol. This is a map
        /// of the lockup period to the reward rate ratio. In this
        /// case, the value is a decimal in the range [0, ∞] where 0
        /// means 0%, 0.5 means 50%, and 1 means 100%.
        reward_rates: KeyValueStore<LockupPeriod, Decimal>,

        /// Controls whether the protocol currently allows users to open
        /// liquidity positions or not.
        is_open_position_enabled: bool,

        /// Controls whether the protocol currently allows users to close
        /// liquidity positions or not.
        is_close_position_enabled: bool,

        /// The maximum allowed staleness of prices in seconds. If a price is
        /// found to be older than this then it is deemed to be invalid.
        maximum_allowed_price_staleness_in_seconds: i64,

        /// The maximum percentage of price difference the protocol is willing
        /// to accept before deeming the price difference to be too much. This
        /// is a decimal in the range [0, ∞] where 0 means 0%, 0.5 means 50%,
        /// and 1 means 100%.
        maximum_allowed_price_difference_percentage: Decimal,
    }

    impl Ignition {
        /// Instantiates a new Ignition protocol component based on the provided
        /// protocol parameters.
        pub fn instantiate(
            metadata_init: MetadataInit,
            /* Rules */
            owner_role: OwnerRole,
            protocol_owner_role: AccessRule,
            protocol_manager_role: AccessRule,
            /* Initial Configuration */
            protocol_resource: ResourceManager,
            oracle_adapter: ComponentAddress,
            maximum_allowed_price_staleness_in_seconds: i64,
            maximum_allowed_price_difference_percentage: Decimal,
            /* Initializers */
            initialization_parameters: InitializationParameters,
            /* Misc */
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<Ignition> {
            // If no address reservation is provided then reserve an address to
            // globalize the component to - this is to provide us with a non
            // branching way of globalizing the component.
            let address_reservation = address_reservation
                .unwrap_or_else(|| Runtime::allocate_component_address(Ignition::blueprint_id()).0);

            let ignition = {
                let InitializationParameters {
                    initial_pool_information,
                    initial_user_resource_volatility,
                    initial_reward_rates,
                    initial_volatile_protocol_resources,
                    initial_non_volatile_protocol_resources,
                    initial_is_open_position_enabled,
                    initial_is_close_position_enabled,
                    initial_matching_factors,
                } = initialization_parameters;

                let mut ignition = Self {
                    protocol_resource,
                    oracle_adapter: oracle_adapter.into(),
                    pool_information: KeyValueStore::new_with_registered_type(),
                    user_resources_vaults:
                        KeyValueStore::new_with_registered_type(),
                    pool_units: KeyValueStore::new_with_registered_type(),
                    reward_rates: KeyValueStore::new_with_registered_type(),
                    is_open_position_enabled: false,
                    is_close_position_enabled: false,
                    maximum_allowed_price_staleness_in_seconds,
                    maximum_allowed_price_difference_percentage,
                    user_resource_volatility:
                        KeyValueStore::new_with_registered_type(),
                    protocol_resource_reserves: ProtocolResourceReserves::new(
                        protocol_resource.address(),
                    ),
                    forced_liquidation_claims:
                        KeyValueStore::new_with_registered_type(),
                    matching_factor: KeyValueStore::new_with_registered_type(),
                };

                if let Some(resource_volatility) =
                    initial_user_resource_volatility
                {
                    for (resource_address, volatility) in
                        resource_volatility.into_iter()
                    {
                        ignition.insert_user_resource_volatility(
                            resource_address,
                            volatility,
                        )
                    }
                }

                if let Some(pool_information) = initial_pool_information {
                    for (blueprint_id, information) in
                        pool_information.into_iter()
                    {
                        ignition
                            .insert_pool_information(blueprint_id, information)
                    }
                }

                if let Some(reward_rates) = initial_reward_rates {
                    for (lockup_period, reward) in reward_rates.into_iter() {
                        ignition.add_reward_rate(lockup_period, reward)
                    }
                }

                if let Some(volatile_protocol_resources) =
                    initial_volatile_protocol_resources
                {
                    ignition.deposit_protocol_resources(
                        volatile_protocol_resources,
                        Volatility::Volatile,
                    )
                }

                if let Some(non_volatile_protocol_resources) =
                    initial_non_volatile_protocol_resources
                {
                    ignition.deposit_protocol_resources(
                        non_volatile_protocol_resources,
                        Volatility::NonVolatile,
                    )
                }

                if let Some(matching_factors) = initial_matching_factors {
                    for (address, matching_factor) in
                        matching_factors.into_iter()
                    {
                        ignition
                            .upsert_matching_factor(address, matching_factor)
                    }
                }

                ignition.is_open_position_enabled =
                    initial_is_open_position_enabled.unwrap_or(false);
                ignition.is_close_position_enabled =
                    initial_is_close_position_enabled.unwrap_or(false);

                ignition
            };

            ignition
                .instantiate()
                .prepare_to_globalize(owner_role)
                .roles(roles! {
                    protocol_owner => protocol_owner_role;
                    protocol_manager => protocol_manager_role;
                })
                .metadata(ModuleConfig {
                    init: metadata_init,
                    roles: Default::default(),
                })
                .with_address(address_reservation)
                .globalize()
        }

        /// Opens a liquidity position for the user.
        ///
        /// Given some bucket of tokens, this method matches this bucket with
        /// XRD of the same value and contributes that XRD to the pool specified
        /// as an argument. The liquidity is locked in that pool for the lockup
        /// period specified as an argument and the user is given back a non
        /// fungible token that represents their portion in the pool.
        ///
        /// If opening a liquidity pool returns more than the pool units and the
        /// change, then these additional tokens are returned back to the caller
        /// and not kept by the protocol.
        ///
        /// # Panics
        ///
        /// There are a number of situations when this method panics and leads
        /// the transaction to fail. Some of them are:
        ///
        /// * If the specified pool is not a registered pool in Ignition and
        /// thus, no liquidity is allowed to be provided to this pool.
        /// * If the lockup period specified by the caller has no corresponding
        /// upfront rewards percentage, and thus it is not a recognized lockup
        /// period by the pool.
        /// * If no adapter is registered for the liquidity pool.
        /// * If the price difference between the pool and the oracle is higher
        /// than what is allowed by the protocol.
        ///
        /// # Arguments
        ///
        /// * `bucket`: [`FungibleBucket`] - A fungible bucket of tokens to
        /// contribute to the pool. Ignition will match the value of this bucket
        /// in XRD and contribute it alongside it to the specified pool.
        /// * `pool_address`: [`ComponentAddress`] - The address of the pool to
        /// contribute to, this must be a valid pool that is registered in the
        /// protocol and that has an adapter.
        /// * `lockup_period`: [`LockupPeriod`] - The amount of time (in
        /// seconds) to lockup the liquidity. This must be a registered lockup
        /// period with a defined upfront rewards rate.
        ///
        /// # Returns
        ///
        /// * [`NonFungibleBucket`] - A non-fungible bucket of the liquidity
        /// position resource that gives the holder the right to close their
        /// liquidity position when the lockup period is up.
        /// * [`FungibleBucket`] - A bucket of the upfront reward provided to
        /// the user based on how long they've locked up their liquidity to
        /// Ignition.
        /// * [`Vec<Bucket>`] - A vector of other buckets that the pools can
        /// return upon contribution, this can be their rewards tokens or
        /// anything else.
        pub fn open_liquidity_position(
            &mut self,
            bucket: FungibleBucket,
            pool_address: ComponentAddress,
            lockup_period: LockupPeriod,
        ) -> (NonFungibleBucket, FungibleBucket, Vec<Bucket>) {
            // Ensure that we currently allow opening liquidity positions.
            assert!(
                self.is_open_position_enabled,
                "{}",
                OPENING_LIQUIDITY_POSITIONS_IS_CLOSED_ERROR
            );

            // Caching a few information so that it is not constantly read from
            // the engine.
            let user_resource_address = bucket.resource_address();
            let user_resource_amount = bucket.amount();

            // Getting the volatility of the user asset from the volatility map.
            let volatility = *self
                .user_resource_volatility
                .get(&user_resource_address)
                .expect(USER_RESOURCES_VOLATILITY_UNKNOWN_ERROR);

            // Ensure that the pool has an adapter and that it is a registered
            // pool. If it is, this means that we can move ahead with the pool.
            // Also, it means that the pool is guaranteed to have the protocol
            // resource on one of its sides.
            let (mut adapter, liquidity_receipt_resource, pool_resources, _) =
                self.checked_get_pool_adapter_information(pool_address)
                    .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR);

            // Ensure that the passed bucket belongs to the pool and that it is
            // not some random resource.
            {
                let (resource1, resource2) = pool_resources;

                assert!(
                    resource1 == user_resource_address
                        || resource2 == user_resource_address,
                    "{}",
                    USER_ASSET_DOES_NOT_BELONG_TO_POOL_ERROR
                );

                assert_ne!(
                    user_resource_address,
                    self.protocol_resource.address(),
                    "{}",
                    USER_MUST_NOT_PROVIDE_PROTOCOL_ASSET_ERROR
                )
            }

            // Compare the price difference between the oracle reported price
            // and the pool reported price - ensure that it is within the
            // allowed price difference range.
            let (oracle_reported_price, pool_reported_price) = {
                let oracle_reported_price = self.checked_get_price(
                    user_resource_address,
                    self.protocol_resource.address(),
                );
                let pool_reported_price = adapter.price(pool_address);
                let relative_difference = oracle_reported_price
                    .relative_difference(&pool_reported_price)
                    .expect(USER_ASSET_DOES_NOT_BELONG_TO_POOL_ERROR);

                assert!(
                    relative_difference
                        <= self.maximum_allowed_price_difference_percentage,
                    "{}",
                    RELATIVE_PRICE_DIFFERENCE_LARGER_THAN_ALLOWED_ERROR
                );

                (oracle_reported_price, pool_reported_price)
            };

            let matching_factor = *self
                .matching_factor
                .get(&pool_address)
                .expect(NO_MATCHING_FACTOR_FOUND_FOR_POOL);

            let matching_amount_of_protocol_resource = pool_reported_price
                .exchange(user_resource_address, user_resource_amount)
                .and_then(|(_, value)| value.checked_mul(matching_factor))
                .expect(UNEXPECTED_ERROR);

            // An assertion added for safety - the pool reported value of the
            // resources must be less than (1 + padding_percentage) * oracle
            // price.
            {
                let maximum_amount = Decimal::ONE
                    .checked_add(
                        self.maximum_allowed_price_difference_percentage,
                    )
                    .and_then(|padding| {
                        oracle_reported_price
                            .exchange(
                                user_resource_address,
                                user_resource_amount,
                            )
                            .expect(UNEXPECTED_ERROR)
                            .1
                            .checked_mul(padding)
                    })
                    .and_then(|value| value.checked_mul(matching_factor))
                    .and_then(|value| {
                        // 17 decimal places so that 9.99 (with 18 nines) rounds
                        // to 10. Essentially fixing for any small loss of
                        // precision.
                        value
                            .checked_round(17, RoundingMode::ToPositiveInfinity)
                    })
                    .unwrap_or(Decimal::MAX);
                assert!(
                    matching_amount_of_protocol_resource <= maximum_amount,
                    "Amount provided by Ignition exceeds the maximum allowed at the current price. Provided: {}, Maximum allowed: {}",
                    matching_amount_of_protocol_resource,
                    maximum_amount
                );
            }

            // Contribute the resources to the pool.
            let user_side_of_liquidity = bucket;
            let protocol_side_of_liquidity = self.withdraw_protocol_resources(
                matching_amount_of_protocol_resource,
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
                volatility,
            );
            let OpenLiquidityPositionOutput {
                pool_units,
                mut change,
                others,
                adapter_specific_information,
            } = adapter.open_liquidity_position(
                pool_address,
                (user_side_of_liquidity.0, protocol_side_of_liquidity.0),
                lockup_period,
            );

            // Calculate the amount of resources that was actually contributed
            // based on the amount of change that we got back.
            let amount_of_user_tokens_contributed = user_resource_amount
                .checked_sub(
                    change
                        .get(&user_resource_address)
                        .map(Bucket::amount)
                        .unwrap_or(Decimal::ZERO),
                )
                // Impossible to get here. This is saying that not only did we
                // get change back that exceeded the amount that was put in, but
                // that the change we got back was so large that it lead us to
                // underflow.
                .expect(OVERFLOW_ERROR);
            let amount_of_protocol_tokens_contributed =
                matching_amount_of_protocol_resource
                    .checked_sub(
                        change
                            .get(&self.protocol_resource.address())
                            .map(Bucket::amount)
                            .unwrap_or(Decimal::ZERO),
                    )
                    // Impossible to get here. This is saying that not only did
                    // we get change back that exceeded the amount that was put
                    // in, but that the change we got back was so large that it
                    // lead us to underflow.
                    .expect(OVERFLOW_ERROR);

            // Determine the amount of upfront tokens to provide to the user
            // based on the lockup period specified.
            let upfront_rewards_amount_in_protocol_resource = {
                let oracle_reported_value_of_user_resource_actually_contributed_in_protocol_resource =
                    oracle_reported_price
                        .exchange(
                            user_resource_address,
                            amount_of_user_tokens_contributed,
                        )
                        .expect(UNEXPECTED_ERROR)
                        .1;

                let associated_rewards_rate = self
                    .reward_rates
                    .get(&lockup_period)
                    .expect(LOCKUP_PERIOD_HAS_NO_ASSOCIATED_REWARDS_RATE_ERROR);

                oracle_reported_value_of_user_resource_actually_contributed_in_protocol_resource
                    .checked_mul(*associated_rewards_rate)
                    .expect(OVERFLOW_ERROR)
            };

            let upfront_reward = self.withdraw_protocol_resources(
                upfront_rewards_amount_in_protocol_resource,
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
                volatility,
            );

            // Deposit the pool units into the protocol itself and mint an NFT
            // used to represent these locked pool units.
            let liquidity_receipt = {
                let data = LiquidityReceipt::new(
                    lockup_period,
                    pool_address,
                    user_resource_address,
                    amount_of_user_tokens_contributed,
                    volatility,
                    amount_of_protocol_tokens_contributed,
                    adapter_specific_information,
                );
                let liquidity_receipt = liquidity_receipt_resource
                    .mint_ruid_non_fungible(data)
                    .as_non_fungible();

                let global_id = NonFungibleGlobalId::new(
                    liquidity_receipt_resource.address(),
                    liquidity_receipt.non_fungible_local_id(),
                );
                self.pool_units.insert(
                    global_id,
                    pool_units
                        .into_inner()
                        .into_iter()
                        .map(|(address, bucket)| {
                            (address, Vault::with_bucket(bucket))
                        })
                        .collect(),
                );

                liquidity_receipt
            };

            // Create the buckets to return back to the user.
            if let Some(bucket) =
                change.remove(&self.protocol_resource.address())
            {
                self.deposit_protocol_resources(
                    FungibleBucket(bucket),
                    volatility,
                )
            }
            let buckets_to_return =
                change.into_values().chain(others).collect();

            // Return all
            (liquidity_receipt, upfront_reward, buckets_to_return)
        }

        /// Closes a liquidity position after its maturity period has elapsed.
        ///
        /// Given the non-fungible representing the liquidity receipt, this
        /// method closes the liquidity position after the maturity period
        /// elapses. The liquidity receipt is burned and the user is given
        /// back some amount of assets. If the user has been forcefully
        /// liquidated by the owner of the protocol then the amount returned
        /// will be the amount they were owed at liquidation time.
        ///
        /// # Arguments
        ///
        /// `liquidity_receipt`: [`NonFungibleBucket`] - A bucket of the non
        /// fungible liquidity receipt.
        ///
        /// # Returns
        ///
        /// [`Vec<Bucket>`] - A vector of buckets of the amount to give back to
        /// the user.
        pub fn close_liquidity_position(
            &mut self,
            liquidity_receipt: NonFungibleBucket,
        ) -> Vec<Bucket> {
            // Ensure that there is only a single NFT in the bucket, we do not
            // service more than a single one at a time.
            assert_eq!(
                liquidity_receipt.amount(),
                Decimal::ONE,
                "{}",
                MORE_THAN_ONE_LIQUIDITY_RECEIPT_NFTS_ERROR
            );

            // At this point it is safe to get the non-fungible global id of the
            // liquidity receipt NFT.
            let liquidity_receipt_global_id = liquidity_receipt
                .non_fungible::<LiquidityReceipt<AnyValue>>()
                .global_id()
                .clone();

            // If the passed non-fungible is found in the KVStore of liquidity
            // claims then it has been forcefully closed and it can be claimed
            // from there.
            let entry = self
                .forced_liquidation_claims
                .get_mut(&liquidity_receipt_global_id);
            if let Some(mut vaults) = entry {
                // The liquidity receipt is no longer needed and can be burned.
                liquidity_receipt.burn();

                // Take all of the funds in the vaults and return them back to
                // the user.
                vaults.iter_mut().map(Vault::take_all).collect()
            }
            // There is no entry in the forced liquidations for this receipt. So
            // we can close it.
            else {
                drop(entry);

                // A liquidity position can only be closed when the closing
                // period is opened. Otherwise, it can't be. However, this
                // does not apply for claiming already liquidated positions.
                assert!(
                    self.is_close_position_enabled,
                    "{}",
                    CLOSING_LIQUIDITY_POSITIONS_IS_CLOSED_ERROR
                );

                let buckets = self.liquidate(liquidity_receipt_global_id);

                // The liquidity receipt is no longer needed and can be burned.
                liquidity_receipt.burn();

                buckets
            }
        }

        /// Forcefully liquidates a liquidity position keeping the resources
        /// in a separate claims KVStore such that users can claim them at any
        /// point of time.
        ///
        /// # Arguments
        ///
        /// `liquidity_receipt_global_id`: [`NonFungibleGlobalId`] - The non
        /// fungible global id of liquidity receipt to liquidate.
        pub fn forcefully_liquidate(
            &mut self,
            liquidity_receipt_global_id: NonFungibleGlobalId,
        ) {
            let buckets = self.liquidate(liquidity_receipt_global_id.clone());
            self.forced_liquidation_claims.insert(
                liquidity_receipt_global_id,
                buckets.into_iter().map(Vault::with_bucket).collect(),
            );
        }

        /// Liquidates a liquidity position after its maturity period has
        /// elapsed.
        ///
        /// Given the non-fungible representing the liquidity receipt, this
        /// method closes the liquidity position after the maturity period
        /// elapses. The liquidity receipt is burned and the user is given
        /// back some amount of assets.
        ///
        /// The assets given back to the user depends on what the protocol gets
        /// back from closing the liquidity position. The following is the
        /// algorithm employed to determine what and how much should be returned
        ///
        /// * Is the amount of the user asset the protocol got back greater than
        /// or equal to the amount that they initially put in?
        ///     * Yes: Return the same amount to them plus any fees from the
        ///     _user_ asset.
        ///     * No: Return to them all of the user asset the protocol got back
        ///     plus the amount required to buy back their missing amount or the
        ///     protocol assets returned when closing the liquidity position,
        ///     whichever one is smaller.
        ///
        /// Whatever the amount obtained from the algorithm defined at the top
        /// is the amount returned to the user. Some of the calculations take
        /// place in the adapters: specifically the estimation of fees.
        ///
        /// # Arguments
        ///
        /// `liquidity_receipt_global_id`: [`NonFungibleGlobalId`] - The non
        /// fungible global id of liquidity receipt to liquidate.
        ///
        /// # Returns
        ///
        /// [`Vec<Bucket>`] - A vector of buckets of the amount to give back to
        /// the user.
        fn liquidate(
            &mut self,
            liquidity_receipt_global_id: NonFungibleGlobalId,
        ) -> Vec<Bucket> {
            let (
                mut adapter,
                liquidity_receipt_data,
                liquidity_receipt_global_id,
            ) = {
                // Reading the data of the non-fungible resource passed and then
                // validating that the resource address is what we expect. We do
                // this as we need to check it against the data of the blueprint
                // of the pool. So, that must be read first.
                let non_fungible =
                    NonFungible::<LiquidityReceipt<AnyValue>>::from(
                        liquidity_receipt_global_id,
                    );
                let liquidity_receipt_data = non_fungible.data();
                let (pool_adapter, liquidity_receipt_resource, _, _) = self
                    .checked_get_pool_adapter_information(
                        liquidity_receipt_data.pool_address,
                    )
                    .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR);

                assert_eq!(
                    non_fungible.resource_address(),
                    liquidity_receipt_resource.address(),
                    "{}",
                    NOT_A_VALID_LIQUIDITY_RECEIPT_ERROR
                );

                // At this point, the non-fungible can be trusted to belong to
                // the liquidity receipt resource of the blueprint.
                (
                    pool_adapter,
                    liquidity_receipt_data,
                    non_fungible.global_id().clone(),
                )
            };

            // Assert that we're after the maturity date.
            assert!(
                Clock::current_time_is_at_or_after(
                    liquidity_receipt_data.maturity_date,
                    TimePrecision::Minute
                ),
                "{}",
                LIQUIDITY_POSITION_HAS_NOT_MATURED_ERROR
            );

            // Compare the price difference between the oracle reported price
            // and the pool reported price - ensure that it is within the
            // allowed price difference range.
            let oracle_reported_price = {
                let oracle_reported_price = self.checked_get_price(
                    liquidity_receipt_data.user_resource_address,
                    self.protocol_resource.address(),
                );
                let pool_reported_price =
                    adapter.price(liquidity_receipt_data.pool_address);
                let relative_difference = oracle_reported_price
                    .relative_difference(&pool_reported_price)
                    .expect(USER_ASSET_DOES_NOT_BELONG_TO_POOL_ERROR);

                assert!(
                    relative_difference
                        <= self.maximum_allowed_price_difference_percentage,
                    "{}",
                    RELATIVE_PRICE_DIFFERENCE_LARGER_THAN_ALLOWED_ERROR
                );

                oracle_reported_price
            };

            /* The liquidity position can be closed! */

            // Withdraw all of the pool units associated with the position and
            // close it through the adapter.
            let CloseLiquidityPositionOutput {
                resources,
                others,
                mut fees,
            } = {
                let pool_units = self
                    .pool_units
                    .get_mut(&liquidity_receipt_global_id)
                    .expect(UNEXPECTED_ERROR)
                    .values_mut()
                    .map(|vault| vault.take_all())
                    .collect::<Vec<_>>();
                adapter.close_liquidity_position(
                    liquidity_receipt_data.pool_address,
                    pool_units,
                    liquidity_receipt_data.adapter_specific_information,
                )
            };

            let (mut user_resource_bucket, mut protocol_resource_bucket) = {
                let user_resource = resources
                    .get(&liquidity_receipt_data.user_resource_address)
                    .map(|item| Bucket(item.0))
                    .expect(UNEXPECTED_ERROR);
                let protocol_resource = resources
                    .get(&self.protocol_resource.address())
                    .map(|item| Bucket(item.0))
                    .expect(UNEXPECTED_ERROR);
                drop(resources);
                (user_resource, protocol_resource)
            };

            let user_resource_bucket_amount = user_resource_bucket.amount();
            let protocol_resource_bucket_amount =
                protocol_resource_bucket.amount();

            fees.values_mut().for_each(|value| {
                // Disallowing any fees from being zero by having a lower bound
                // at 0. This is enforced by the protocol itself such that any
                // adapter that returns any negative fees due to estimations
                // does not cause the protocol to calculate incorrectly.
                *value = max(*value, Decimal::ZERO)
            });
            let (user_resource_fees, _) = {
                let user_resource = fees
                    .get(&liquidity_receipt_data.user_resource_address)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let protocol_resource = fees
                    .get(&self.protocol_resource.address())
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                drop(fees);
                (user_resource, protocol_resource)
            };

            // Determine the amount of resources that the user should be given
            // back.
            //
            // Branch 1: There is enough of the user asset to give the user back
            // the same amount that they put in.
            let (
                amount_of_protocol_resource_to_give_user,
                amount_of_user_resource_to_give_user,
            ) = if user_resource_bucket_amount
                >= liquidity_receipt_data.user_contribution_amount
            {
                let amount_of_protocol_resource_to_give_user = dec!(0);
                let amount_of_user_resource_to_give_user = min(
                    user_resource_bucket_amount,
                    liquidity_receipt_data
                        .user_contribution_amount
                        .checked_add(user_resource_fees)
                        .expect(OVERFLOW_ERROR),
                );

                (
                    amount_of_protocol_resource_to_give_user,
                    amount_of_user_resource_to_give_user,
                )
            }
            // Branch 2: There is not enough of the user token to given them
            // back the same amount that they put in.
            else {
                let amount_of_protocol_resource_to_give_user = {
                    let user_amount_missing = liquidity_receipt_data
                        .user_contribution_amount
                        .checked_sub(user_resource_bucket_amount)
                        .expect(OVERFLOW_ERROR);
                    let (_, protocol_resources_required_for_buy_back) =
                        oracle_reported_price
                            .exchange(
                                liquidity_receipt_data.user_resource_address,
                                user_amount_missing,
                            )
                            .expect(UNEXPECTED_ERROR);
                    min(
                        protocol_resources_required_for_buy_back,
                        protocol_resource_bucket_amount,
                    )
                };
                let amount_of_user_resource_to_give_user =
                    user_resource_bucket_amount;

                (
                    amount_of_protocol_resource_to_give_user,
                    amount_of_user_resource_to_give_user,
                )
            };

            let mut bucket_returns = others;
            bucket_returns.push(user_resource_bucket.take_advanced(
                amount_of_user_resource_to_give_user,
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            ));
            bucket_returns.push(protocol_resource_bucket.take_advanced(
                amount_of_protocol_resource_to_give_user,
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            ));

            // Deposit the remaining resources back into the protocol.
            self.deposit_user_resources(user_resource_bucket.as_fungible());
            self.deposit_protocol_resources(
                protocol_resource_bucket.as_fungible(),
                liquidity_receipt_data.user_resource_volatility_classification,
            );

            // Return the buckets back
            bucket_returns
        }

        /// Updates the matching factor of a pool.
        ///
        /// This method updates the matching factor for a given pool after doing
        /// a bounds check on it ensuring that it is in the range [0, 1]. This
        /// performs an upsert operation meaning that if an entry already exists
        /// then that entry will be overwritten.
        ///
        /// # Example Scenario
        ///
        /// We may want to dynamically control the matching factor of pools such
        /// that we can update them at runtime instead of doing a new deployment
        /// of Ignition.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// * `component_address`: [`ComponentAddress`] - The address of the
        /// pool to set the matching factor for.
        /// * `matching_factor`: [`Decimal`] - The matching factor of the pool.
        pub fn upsert_matching_factor(
            &mut self,
            component_address: ComponentAddress,
            matching_factor: Decimal,
        ) {
            if matching_factor < Decimal::ZERO || matching_factor > Decimal::ONE
            {
                panic!("{}", INVALID_MATCHING_FACTOR)
            }
            self.matching_factor
                .insert(component_address, matching_factor)
        }

        /// Updates the oracle adapter used by the protocol to a different
        /// adapter.
        ///
        /// This method does _not_ check that the interface of the new oracle
        /// matches that we expect. Thus, such a check must be performed
        /// off-ledger.
        ///
        /// To be more specific, this method takes in the component address of
        /// the oracle's _adapter_ and not the oracle itself. The adapter must
        /// have the interface defined in [`OracleAdapter`].
        ///
        /// # Example Scenario
        ///
        /// We may wish to change the oracle provider for any number of reasons.
        /// As an example, imagine if the initial oracle provider goes under and
        /// stops operations. This allows for the oracle to be replaced with one
        /// that has the same interface without the need to jump to a new
        /// component.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// * `oracle`: [`ComponentAddress`] - The address of the new oracle
        /// component to use.
        ///
        /// # Note
        ///
        /// This performs no interface checks and can theoretically accept the
        /// address of a component that does not implement the oracle interface.
        pub fn set_oracle_adapter(&mut self, oracle_adapter: ComponentAddress) {
            self.oracle_adapter = oracle_adapter.into();
        }

        /// Sets the pool adapter that should be used by a pools belonging to a
        /// particular blueprint.
        ///
        /// Given the blueprint id of a pool whose information is already known
        /// to the protocol, this method changes it to use a new adapter instead
        /// of its existing one. All future opening and closing of liquidity
        /// positions happens through the new adapter.
        ///
        /// This method does not check that the provided adapter conforms to the
        /// [`PoolAdapter`] interface. It is the job of the caller to perform
        /// this check off-ledger.
        ///
        /// # Panics
        ///
        /// This function panics in the following cases:
        ///
        /// * If the provided address's blueprint has no corresponding
        /// blueprint.
        ///
        /// # Example Scenario
        ///
        /// We may wish to add support for additional decentralized exchanges
        /// after the protocol goes live. To do this, we would just need to
        /// develop and deploy an adapter and then register the adapter to the
        /// protocol through this method.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// `blueprint_id`: [`BlueprintId`] - The package address and blueprint
        /// name of the pool blueprint.
        /// `pool_adapter`: [`ComponentAddress`] - The address of the adapter
        /// component.
        ///
        /// # Note
        ///
        /// This performs no interface checks and can theoretically accept the
        /// address of a component that does not implement the oracle interface.
        pub fn set_pool_adapter(
            &mut self,
            blueprint_id: BlueprintId,
            pool_adapter: ComponentAddress,
        ) {
            self.pool_information
                .get_mut(&blueprint_id)
                .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR)
                .adapter = pool_adapter.into();
        }

        /// Adds an allowed pool to the protocol.
        ///
        /// This protocol does not provide an incentive to any liquidity pool.
        /// Only a small set of pools that are chosen by the pool manager. This
        /// method adds a pool to the set of pools that the protocol provides an
        /// incentive for and that users can provide liquidity to.
        ///
        /// This method checks that an adapter exists for the passed component.
        /// If no adapter exists then this method panics and the transaction
        /// fails.
        ///
        /// # Panics
        ///
        /// This function panics in two main cases:
        ///
        /// * If the provided address's blueprint has no corresponding
        /// blueprint.
        /// * If neither side of the pool is the protocol resource.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Example Scenario
        ///
        /// We may wish to incentivize liquidity for a new bridged resource and
        /// a new set of pools. An even more compelling scenario, we may wish to
        /// provide incentives for a newly released DEX.
        ///
        /// # Arguments
        ///
        /// * `component`: [`ComponentAddress`] - The address of the pool
        /// component to add to the set of allowed pools.
        pub fn add_allowed_pool(&mut self, pool_address: ComponentAddress) {
            let protocol_resource_address = self.protocol_resource.address();
            let user_resource_volatility = KeyValueStore {
                id: self.user_resource_volatility.id,
                key: PhantomData,
                value: PhantomData,
            };

            self.with_pool_blueprint_information_mut(
                pool_address,
                |pool_information| {
                    let resources = PoolAdapter::from(pool_information.adapter)
                        .resource_addresses(pool_address);

                    Self::check_pool_resources(
                        resources,
                        protocol_resource_address,
                        &user_resource_volatility,
                    );

                    pool_information
                        .allowed_pools
                        .insert(pool_address, resources);
                },
            )
            .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR)
        }

        /// Removes one of the existing allowed liquidity pools.
        ///
        /// Given the component address of the liquidity pool, this method
        /// removes that liquidity pool from the list of allowed liquidity
        /// pools.
        ///
        /// # Panics
        ///
        /// This function panics in the following cases:
        ///
        /// * If the provided address's blueprint has no corresponding
        /// blueprint.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Example Scenario
        ///
        /// We may wish to to remove or stop a certain liquidity pool from the
        /// incentive program essentially disallowing new liquidity positions
        /// but permitting closure of liquidity positions.
        ///
        /// # Arguments
        ///
        /// * `component`: [`ComponentAddress`] - The address of the pool
        /// component to remove from the set of allowed pools.
        pub fn remove_allowed_pool(&mut self, pool_address: ComponentAddress) {
            self.with_pool_blueprint_information_mut(
                pool_address,
                |pool_information| {
                    pool_information.allowed_pools.swap_remove(&pool_address);
                },
            )
            .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR)
        }

        /// Sets the liquidity receipt resource associated with a particular
        /// pool blueprint.
        ///
        /// # Panics
        ///
        /// This function panics in the following cases:
        ///
        /// * If the provided address's blueprint has no corresponding
        /// blueprint.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// `blueprint_id`: [`BlueprintId`] - The blueprint id of the pool
        /// blueprint.
        /// `liquidity_receipt``: [`ResourceManager`] - The resource address of
        /// the new liquidity receipt resource to use.
        pub fn set_liquidity_receipt(
            &mut self,
            blueprint_id: BlueprintId,
            liquidity_receipt: ResourceManager,
        ) {
            self.pool_information
                .get_mut(&blueprint_id)
                .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR)
                .liquidity_receipt = liquidity_receipt.address();
        }

        /// Inserts the pool information, adding it to the protocol, performing
        /// an upsert.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// * `blueprint_id`: [`BlueprintId`] - The id of the pool blueprint
        /// to add the information for.
        /// * `PoolBlueprintInformation`: [`PoolBlueprintInformation`] The
        /// protocol information related to the blueprint.
        pub fn insert_pool_information(
            &mut self,
            blueprint_id: BlueprintId,
            pool_information: PoolBlueprintInformation,
        ) {
            let protocol_resource_address = self.protocol_resource.address();
            let pool_information = StoredPoolBlueprintInformation {
                adapter: PoolAdapter::from(pool_information.adapter),
                liquidity_receipt: pool_information.liquidity_receipt,
                allowed_pools: pool_information
                    .allowed_pools
                    .into_iter()
                    .map(|pool_component_address| {
                        let mut adapter =
                            PoolAdapter::from(pool_information.adapter);

                        let resources =
                            adapter.resource_addresses(pool_component_address);

                        Self::check_pool_resources(
                            resources,
                            protocol_resource_address,
                            &self.user_resource_volatility,
                        );

                        (pool_component_address, resources)
                    })
                    .collect(),
            };

            self.pool_information.insert(blueprint_id, pool_information)
        }

        /// Removes the pool's blueprint information from the protocol.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// * `blueprint_id`: [`BlueprintId`] - The id of the pool blueprint
        /// to remove the information for.
        pub fn remove_pool_information(&mut self, blueprint_id: BlueprintId) {
            self.pool_information.remove(&blueprint_id);
        }

        /// Deposits protocol resources into the appropriate vaults.
        ///
        /// Depending on whether the protocol resources deposited are to be used
        /// for volatile or non-volatile contributions this method deposits them
        /// into the appropriate vaults.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// * `bucket`: [`FungibleBucket`] - A bucket of the protocol resources
        /// to deposit into the protocol, making them available to the protocol
        /// to be used in matching the contribution of users.
        /// * `volatility`: [`Volatility`] - Whether the resources are to be
        /// used for matching volatile or non-volatile user assets.
        pub fn deposit_protocol_resources(
            &mut self,
            bucket: FungibleBucket,
            volatility: Volatility,
        ) {
            self.protocol_resource_reserves.deposit(bucket, volatility)
        }

        /// Withdraws protocol resources from the protocol.
        ///
        /// Withdraws the specified amount from the appropriate vault which is
        /// either that of the volatile or non-volatile contributions.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// * `amount`: [`Decimal`] - The amount of resources to withdraw.
        /// * `withdraw_strategy`: [`WithdrawStrategy`] - The strategy to use
        /// when withdrawing. This is only relevant when the protocol resource's
        /// divisibility is not 18. If it is 18, then this does not really make
        /// any difference.
        /// * `volatility`: [`Volatility`] - Controls whether the withdraw
        /// should happen against the volatile or non-volatile vaults.
        ///
        /// # Returns
        ///
        /// * [`FungibleBucket`] - A bucket of the fungible protocol resources
        /// withdrawn from the protocol.
        pub fn withdraw_protocol_resources(
            &mut self,
            amount: Decimal,
            withdraw_strategy: WithdrawStrategy,
            volatility: Volatility,
        ) -> FungibleBucket {
            self.protocol_resource_reserves.withdraw(
                amount,
                withdraw_strategy,
                volatility,
            )
        }

        /// Deposits resources into the protocol.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Example Scenario
        ///
        /// This method can be used to fund the incentive program with XRD and
        /// deposit other resources as well.
        ///
        /// # Arguments
        ///
        /// * `bucket`: [`FungibleBucket`] - A bucket of resources to deposit
        /// into the protocol.
        pub fn deposit_user_resources(&mut self, bucket: FungibleBucket) {
            let entry = self
                .user_resources_vaults
                .get_mut(&bucket.resource_address());
            if let Some(mut vault) = entry {
                vault.put(bucket);
            } else {
                drop(entry);
                self.user_resources_vaults.insert(
                    bucket.resource_address(),
                    FungibleVault::with_bucket(bucket),
                )
            }
        }

        /// Withdraws resources from the protocol.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Example Scenario
        ///
        /// This method can be used to end the incentive program by withdrawing
        /// the XRD in the protocol. Additionally, it can be used for upgrading
        /// the protocol by withdrawing the resources in the protocol.
        ///
        /// # Arguments
        ///
        /// * `resource_address`: [`ResourceAddress`] - The address of the
        /// resource to withdraw.
        /// * `amount`: [`Decimal`] - The amount to withdraw.
        ///
        /// # Returns
        ///
        /// * [`FungibleBucket`] - A bucket of the withdrawn tokens.
        pub fn withdraw_user_resources(
            &mut self,
            resource_address: ResourceAddress,
            amount: Decimal,
        ) -> FungibleBucket {
            self.user_resources_vaults
                .get_mut(&resource_address)
                .expect(NO_ASSOCIATED_VAULT_ERROR)
                .take(amount)
        }

        /// Deposits pool units into the protocol.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Arguments
        ///
        /// * `global_id`: [`NonFungibleGlobalId`] - The global id of the
        /// non-fungible liquidity position NFT whose associated pool units
        /// are to be deposited.
        /// * `pool_units`: [`Bucket`] - The pool units to deposit into the
        /// protocol.
        pub fn deposit_pool_units(
            &mut self,
            global_id: NonFungibleGlobalId,
            pool_units: Bucket,
        ) {
            let pool_units_resource_address = pool_units.resource_address();

            let entry = self.pool_units.get_mut(&global_id);
            if let Some(mut vaults) = entry {
                if let Some(vault) =
                    vaults.get_mut(&pool_units_resource_address)
                {
                    vault.put(pool_units)
                } else {
                    vaults.insert(
                        pool_units_resource_address,
                        Vault::with_bucket(pool_units),
                    );
                }
            } else {
                drop(entry);
                self.pool_units.insert(
                    global_id,
                    indexmap! {
                        pool_units_resource_address => Vault::with_bucket(pool_units)
                    },
                )
            }
        }

        /// Withdraws pool units from the protocol. This is primarily for any
        /// upgradeability needs that the protocol has.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Example Scenario
        ///
        /// This method can be used to withdraw the pool units from the protocol
        /// for the purposes of upgradeability to move them to another component
        ///
        /// # Arguments
        ///
        /// * `id`: [`NonFungibleGlobalId`] - The global id of the non-fungible
        /// liquidity position NFTs to withdraw the pool units associated with.
        ///
        /// # Returns
        ///
        /// * [`Vec<Bucket>`] - A vector of buckets of the pool units for the
        /// specified liquidity receipt.
        pub fn withdraw_pool_units(
            &mut self,
            global_id: NonFungibleGlobalId,
        ) -> Vec<Bucket> {
            self.pool_units
                .get_mut(&global_id)
                .expect(NO_ASSOCIATED_LIQUIDITY_RECEIPT_VAULT_ERROR)
                .values_mut()
                .map(|vault| vault.take_all())
                .collect()
        }

        /// Updates the value of the maximum allowed price staleness used by
        /// the protocol.
        ///
        /// This means that any price checks that happen when opening or closing
        /// liquidity positions will be subjected to the new maximum allowed
        /// staleness.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` or `protocol_manager` role.
        ///
        /// # Example Scenario
        ///
        /// We may wish to change the allowed staleness of prices to a very
        /// short period if we get an oracle that operates at realtime speeds
        /// or if we change oracle vendors.
        ///
        /// # Arguments
        ///
        /// * `value`: [`i64`] - The maximum allowed staleness period in
        /// seconds.
        pub fn set_maximum_allowed_price_staleness_in_seconds(
            &mut self,
            value: i64,
        ) {
            assert!(value >= 0, "{}", INVALID_MAXIMUM_PRICE_STALENESS);
            self.maximum_allowed_price_staleness_in_seconds = value
        }

        /// Adds a rewards rate to the protocol.
        ///
        /// Given a certain lockup period in seconds and a percentage rewards
        /// rate, this method adds this rate to the protocol allowing users to
        /// choose this option when contributing liquidity.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Example Scenario
        ///
        /// We might wish to add a new higher rate with a longer lockup period
        /// to incentivize people to lock up their liquidity for even shorter.
        /// Or, we might want to introduce a new 3 months category, or anything
        /// in between.
        ///
        /// # Arguments
        ///
        /// * `lockup_period`: [`LockupPeriod`] - The lockup period.
        /// * `rate`: [`Decimal`] - The rewards rate as a percent. This is a
        /// percentage value where 0 represents 0%, 0.5 represents 50% and 1
        /// represents 100%.
        pub fn add_reward_rate(
            &mut self,
            lockup_period: LockupPeriod,
            percentage: Decimal,
        ) {
            assert!(
                percentage >= Decimal::ZERO,
                "{}",
                INVALID_UPFRONT_REWARD_PERCENTAGE
            );
            self.reward_rates.insert(lockup_period, percentage)
        }

        /// Removes a rewards rate from the protocol.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Example Scenario
        ///
        /// A certain rate might get used too much and we might want to switch
        /// off this rate (even if temporarily). This allows us to remove this
        /// rate and add it back later when we want to.
        ///
        /// # Arguments
        ///
        /// * `lockup_period`: [`LockupPeriod`] - The lockup period in seconds
        /// associated with the rewards rate that we would like to remove.
        pub fn remove_reward_rate(&mut self, lockup_period: LockupPeriod) {
            self.reward_rates.remove(&lockup_period);
        }

        /// Inserts the volatility of the user resource to the protocol.
        ///
        /// # Arguments
        ///
        /// * `resource_address`: [`ResourceAddress`] - The address of the
        /// resource to add a volatility classification for.
        /// * `volatility`: [`Volatility`] - The volatility classification of
        /// the resource.
        pub fn insert_user_resource_volatility(
            &mut self,
            resource_address: ResourceAddress,
            volatility: Volatility,
        ) {
            self.user_resource_volatility
                .insert(resource_address, volatility)
        }

        /// Enables or disables the ability to open new liquidity positions
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Example Scenario
        ///
        /// We might want to pause the incentive program for some period due to
        /// any number of reasons.
        ///
        /// # Arguments
        ///
        /// * `value`: [`bool`] - Controls whether opening of liquidity
        /// positions is enabled or disabled.
        pub fn set_is_open_position_enabled(&mut self, value: bool) {
            self.is_open_position_enabled = value
        }

        /// Enables or disables the ability to close new liquidity positions
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Example Scenario
        ///
        /// We might want to pause the incentive program for some period due to
        /// any number of reasons.
        ///
        /// # Arguments
        ///
        /// * `value`: [`bool`] - Controls whether closing of liquidity
        /// positions is enabled or disabled.
        pub fn set_is_close_position_enabled(&mut self, value: bool) {
            self.is_close_position_enabled = value
        }

        /// Updates the value of the maximum allowed price difference between
        /// the pool and the oracle.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` or `protocol_manager` role.
        ///
        /// # Example Scenario
        ///
        /// As more and more arbitrage bots get created, we may want to make the
        /// price difference allowed narrower and narrower.
        ///
        /// # Arguments
        ///
        /// `value`: [`Decimal`] - The maximum allowed percentage difference.
        /// This is a percentage value where 0 represents 0%, 0.5 represents
        /// 50% and 1 represents 100%.
        pub fn set_maximum_allowed_price_difference_percentage(
            &mut self,
            value: Decimal,
        ) {
            self.maximum_allowed_price_difference_percentage = value
        }

        /* Getters */
        pub fn get_user_resource_reserves_amount(
            &self,
            resource_address: ResourceAddress,
        ) -> Decimal {
            self.user_resources_vaults
                .get(&resource_address)
                .map(|vault| vault.amount())
                .unwrap_or_default()
        }

        pub fn get_protocol_resource_reserves_amount(
            &self,
            volatility: Volatility,
        ) -> Decimal {
            self.protocol_resource_reserves.vault(volatility).amount()
        }

        /// An internal method that is used to execute callbacks against the
        /// blueprint of some pool.
        fn with_pool_blueprint_information_mut<F, O>(
            &mut self,
            pool_address: ComponentAddress,
            callback: F,
        ) -> Option<O>
        where
            F: FnOnce(
                &mut KeyValueEntryRefMut<'_, StoredPoolBlueprintInformation>,
            ) -> O,
        {
            let blueprint_id = ScryptoVmV1Api::object_get_blueprint_id(
                pool_address.as_node_id(),
            );
            let entry = self.pool_information.get_mut(&blueprint_id);
            entry.map(|mut entry| callback(&mut entry))
        }

        /// Gets the adapter and the liquidity receipt given a pool address.
        ///
        /// This method first gets the pool information associated with the pool
        /// blueprint and then checks to ensure that the pool is in the allow
        /// list of pools. If it is, it returns the adapter and the resource
        /// manager reference of the liquidity receipt.
        ///
        /// If a [`None`] is returned it means that no pool information was
        /// found for the pool and that it has no corresponding adapter that
        /// we can use.
        ///
        /// # Panics
        ///
        /// * If the pool is not in the list of allowed pools.
        ///
        /// # Arguments
        ///
        /// `pool_address`: [`ComponentAddress`] - The address of the component
        /// to get the adapter and liquidity receipt for.
        ///
        /// # Returns
        ///
        /// * [`PoolAdapter`] - The adapter to use for the pool.
        /// * [`ResourceManager`] - The resource manager reference of the
        /// liquidity receipt token.
        /// * [`(ResourceAddress, ResourceAddress)`] - A tuple of the resource
        /// addresses of the pool.
        ///
        /// # Note
        ///
        /// The [`KeyValueEntryRef<'_, PoolBlueprintInformation>`] is returned
        /// to allow the references of the addresses to remain.
        fn checked_get_pool_adapter_information(
            &self,
            pool_address: ComponentAddress,
        ) -> Option<(
            PoolAdapter,
            ResourceManager,
            (ResourceAddress, ResourceAddress),
            KeyValueEntryRef<'_, StoredPoolBlueprintInformation>,
        )> {
            let blueprint_id = ScryptoVmV1Api::object_get_blueprint_id(
                pool_address.as_node_id(),
            );
            let entry = self.pool_information.get(&blueprint_id);

            entry.map(|entry| {
                let resources = entry
                    .allowed_pools
                    .get(&pool_address)
                    .expect(POOL_IS_NOT_IN_ALLOW_LIST_ERROR);

                (entry.adapter, entry.liquidity_receipt(), *resources, entry)
            })
        }

        /// Gets the price of the `base` resource in terms of the `quote`
        /// resource from the currently configured oracle, checks for
        /// staleness, and returns the price.
        ///
        /// # Arguments
        ///
        /// * `base`: [`ResourceAddress`] - The base resource address.
        /// * `quote`: [`ResourceAddress`] - The quote resource address.
        ///
        /// # Returns
        ///
        /// [`Price`] - The price of the base resource in terms of the quote
        /// resource.
        fn checked_get_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
        ) -> Price {
            // Get the price
            let (price, last_update) =
                self.oracle_adapter.get_price(base, quote);
            let final_price_validity = last_update
                .add_seconds(self.maximum_allowed_price_staleness_in_seconds)
                .unwrap_or(Instant::new(i64::MAX));

            // Check for staleness
            assert!(
                Clock::current_time_is_at_or_before(
                    final_price_validity,
                    TimePrecision::Minute
                ),
                "{}",
                ORACLE_REPORTED_PRICE_IS_STALE_ERROR
            );

            // Return price
            Price { price, base, quote }
        }

        fn check_pool_resources(
            resources: (ResourceAddress, ResourceAddress),
            protocol_resource_address: ResourceAddress,
            user_resource_volatility: &KeyValueStore<
                ResourceAddress,
                Volatility,
            >,
        ) {
            // Ensure that one of the resources is the protocol resource.
            assert!(
                resources.0 == protocol_resource_address
                    || resources.1 == protocol_resource_address,
                "{}",
                NEITHER_POOL_RESOURCE_IS_PROTOCOL_RESOURCE_ERROR
            );

            // Ensure that the user asset has a registered volatility.
            let user_resource = if resources.0 == protocol_resource_address {
                resources.1
            } else if resources.1 == protocol_resource_address {
                resources.0
            } else {
                unreachable!("{}", NEITHER_POOL_RESOURCE_IS_USER_RESOURCE_ERROR)
            };

            // Ensure that the user's resource is not the protocol resource.
            // A pool whose two assets is the same is an issue.
            assert_ne!(
                user_resource, protocol_resource_address,
                "{}",
                BOTH_POOL_ASSETS_ARE_THE_PROTOCOL_RESOURCE
            );

            user_resource_volatility
                .get(&user_resource)
                .expect(USER_RESOURCES_VOLATILITY_UNKNOWN_ERROR);
        }
    }
}

/// Represents the information of pools belonging to a particular blueprint that
/// the Ignition component stores in its state. This type is not public as it
/// does not need to be.
#[derive(Clone, Debug, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
struct StoredPoolBlueprintInformation {
    /// The adapter to utilize when making calls to pools belonging to this
    /// blueprint.
    pub adapter: PoolAdapter,

    /// A map of the pools that the protocol allows contributions to. A pool
    /// that is not found in this map for their corresponding blueprint will
    /// not be allowed to be contributed to. The value in this map is the
    pub allowed_pools:
        IndexMap<ComponentAddress, (ResourceAddress, ResourceAddress)>,

    /// A reference to the resource manager of the resource used as a receipt
    /// for providing liquidity to pools of this blueprint
    pub liquidity_receipt: ResourceAddress,
}

/// Represents the information of pools belonging to a particular blueprint.
#[derive(Clone, Debug, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct PoolBlueprintInformation {
    /// The adapter to utilize when making calls to pools belonging to this
    /// blueprint.
    pub adapter: ComponentAddress,

    /// A vector of the pools that the protocol allows contributions to. A pool
    /// that is not found in this list for their corresponding blueprint will
    /// not be allowed to be contributed to.
    pub allowed_pools: IndexSet<ComponentAddress>,

    /// A reference to the resource manager of the resource used as a receipt
    /// for providing liquidity to pools of this blueprint
    pub liquidity_receipt: ResourceAddress,
}

impl StoredPoolBlueprintInformation {
    pub fn liquidity_receipt(&self) -> ResourceManager {
        ResourceManager::from(self.liquidity_receipt)
    }
}

/// The reserves of the ignition protocol asset split by the assets to use in
/// volatile and non-volatile contributions.
#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct ProtocolResourceReserves {
    /// A fungible vault of the protocol asset used for matching contributions
    /// to pools of volatile assets.
    pub volatile: FungibleVault,
    /// A fungible vault of the protocol asset used for matching contributions
    /// to pools of non volatile assets.
    pub non_volatile: FungibleVault,
}

impl ProtocolResourceReserves {
    pub fn new(protocol_resource_address: ResourceAddress) -> Self {
        Self {
            volatile: FungibleVault::new(protocol_resource_address),
            non_volatile: FungibleVault::new(protocol_resource_address),
        }
    }

    pub fn withdraw(
        &mut self,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        volatility: Volatility,
    ) -> FungibleBucket {
        self.vault_mut(volatility)
            .take_advanced(amount, withdraw_strategy)
    }

    pub fn deposit(&mut self, bucket: FungibleBucket, volatility: Volatility) {
        self.vault_mut(volatility).put(bucket)
    }

    fn vault_mut(&mut self, volatility: Volatility) -> &mut FungibleVault {
        match volatility {
            Volatility::Volatile => &mut self.volatile,
            Volatility::NonVolatile => &mut self.non_volatile,
        }
    }

    fn vault(&self, volatility: Volatility) -> &FungibleVault {
        match volatility {
            Volatility::Volatile => &self.volatile,
            Volatility::NonVolatile => &self.non_volatile,
        }
    }
}

/// Optional parameters to set on Ignition when its first instantiated. All of
/// the items here are not required to be provided when ignition is first
/// created, but providing them this way saves on fees.
#[derive(Debug, PartialEq, Eq, ScryptoSbor, Default)]
pub struct InitializationParameters {
    /// The initial set of pool information to add to to Ignition.
    pub initial_pool_information:
        Option<IndexMap<BlueprintId, PoolBlueprintInformation>>,

    /// The initial volatility settings to add to Ignition.
    pub initial_user_resource_volatility:
        Option<IndexMap<ResourceAddress, Volatility>>,

    /// The initial set of reward rates to add to Ignition.
    pub initial_reward_rates: Option<IndexMap<LockupPeriod, Decimal>>,

    /// The initial volatile protocol resources to deposit into that vault.
    pub initial_volatile_protocol_resources: Option<FungibleBucket>,

    /// The initial non volatile protocol resources to deposit into that vault.
    pub initial_non_volatile_protocol_resources: Option<FungibleBucket>,

    /// The initial control of whether the user is allowed to open a liquidity
    /// position or not. Defaults to [`false`] if not specified.
    pub initial_is_open_position_enabled: Option<bool>,

    /// The initial control of whether the user is allowed to close a liquidity
    /// position or not. Defaults to [`false`] if not specified.
    pub initial_is_close_position_enabled: Option<bool>,

    /// The initial map of matching factors to use in Ignition.
    pub initial_matching_factors: Option<IndexMap<ComponentAddress, Decimal>>,
}

#[derive(Debug, PartialEq, Eq, ManifestSbor, Default)]
pub struct InitializationParametersManifest {
    /// The initial set of pool information to add to to Ignition.
    pub initial_pool_information:
        Option<IndexMap<BlueprintId, PoolBlueprintInformation>>,

    /// The initial volatility settings to add to Ignition.
    pub initial_user_resource_volatility:
        Option<IndexMap<ResourceAddress, Volatility>>,

    /// The initial set of reward rates to add to Ignition.
    pub initial_reward_rates: Option<IndexMap<LockupPeriod, Decimal>>,

    /// The initial volatile protocol resources to deposit into that vault.
    pub initial_volatile_protocol_resources: Option<ManifestBucket>,

    /// The initial non volatile protocol resources to deposit into that vault.
    pub initial_non_volatile_protocol_resources: Option<ManifestBucket>,

    /// The initial control of whether the user is allowed to open a liquidity
    /// position or not. Defaults to [`false`] if not specified.
    pub initial_is_open_position_enabled: Option<bool>,

    /// The initial control of whether the user is allowed to close a liquidity
    /// position or not. Defaults to [`false`] if not specified.
    pub initial_is_close_position_enabled: Option<bool>,

    /// The initial map of matching factors to use in Ignition.
    pub initial_matching_factors: Option<IndexMap<ComponentAddress, Decimal>>,
}
