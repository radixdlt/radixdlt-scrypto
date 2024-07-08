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

macro_rules! define_error {
    (
        $(
            $name: ident => $item: expr;
        )*
    ) => {
        $(
            pub const $name: &'static str = concat!("[Ignition]", " ", $item);
        )*
    };
}

define_error! {
    NO_ADAPTER_FOUND_FOR_POOL_ERROR
        => "No adapter found for liquidity pool.";
    NEITHER_POOL_RESOURCE_IS_PROTOCOL_RESOURCE_ERROR
        => "Neither pool resource is the protocol resource.";
    NEITHER_POOL_RESOURCE_IS_USER_RESOURCE_ERROR
        => "Neither pool resource is the user resource.";
    NO_ASSOCIATED_VAULT_ERROR
        => "The resource has no associated vault in the protocol.";
    NO_ASSOCIATED_LIQUIDITY_RECEIPT_VAULT_ERROR
        => "The liquidity receipt has no associated vault in the protocol.";
    NOT_AN_IGNITION_ADDRESS_ERROR
        => "The passed allocated address is not an ignition address.";
    OPENING_LIQUIDITY_POSITIONS_IS_CLOSED_ERROR
        => "Opening liquidity positions is disabled.";
    CLOSING_LIQUIDITY_POSITIONS_IS_CLOSED_ERROR
        => "Closing liquidity positions is disabled.";
    NO_REWARDS_RATE_ASSOCIATED_WITH_LOCKUP_PERIOD_ERROR
        => "No rewards rate associated with lockup period.";
    POOL_IS_NOT_IN_ALLOW_LIST_ERROR
        => "Pool is not in allow list.";
    ORACLE_REPORTED_PRICE_IS_STALE_ERROR
        => "Oracle reported price is stale.";
    LOCKUP_PERIOD_HAS_NO_ASSOCIATED_REWARDS_RATE_ERROR
        => "Lockup period has no associated rewards rate.";
    UNEXPECTED_ERROR
        => "Unexpected error.";
    RELATIVE_PRICE_DIFFERENCE_LARGER_THAN_ALLOWED_ERROR
        => "Relative price difference between oracle and pool exceeds allowed.";
    USER_ASSET_DOES_NOT_BELONG_TO_POOL_ERROR
        => "The asset of the user does not belong to the pool.";
    MORE_THAN_ONE_LIQUIDITY_RECEIPT_NFTS_ERROR
        => "More than one liquidity receipt non-fungibles were provided.";
    NOT_A_VALID_LIQUIDITY_RECEIPT_ERROR
        => "Not a valid liquidity receipt resource.";
    LIQUIDITY_POSITION_HAS_NOT_MATURED_ERROR
        => "Can't close a liquidity position before it has matured.";
    USER_MUST_NOT_PROVIDE_PROTOCOL_ASSET_ERROR
        => "The user has provided the protocol asset, which is not allowed";
    USER_RESOURCES_VOLATILITY_UNKNOWN_ERROR
        => "A user resource with no registered volatility status was interacted with.";
    BOTH_POOL_ASSETS_ARE_THE_PROTOCOL_RESOURCE
        => "The user resource can not be the protocol resource.";
    OVERFLOW_ERROR => "Overflow error";
    INVALID_MAXIMUM_PRICE_STALENESS
        => "Price staleness must be a positive or zero integer";
    INVALID_UPFRONT_REWARD_PERCENTAGE
        => "Upfront rewards must be positive or zero decimals";
    NO_MATCHING_FACTOR_FOUND_FOR_POOL
        => "Pool doesn't have a matching factor";
    INVALID_MATCHING_FACTOR => "Invalid matching factor";
}
