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

use crate::prelude::*;

macro_rules! define_error_checking_functions {
    (
        $(
            $prefix: ident => [
                $(
                    $error: ident
                ),* $(,)?
            ]
        ),* $(,)?
    ) => {
        paste::paste! {
            $(
                $(
                    #[must_use]
                    pub fn [< is_ $prefix:snake _ $error:lower:camel:snake >]<T>(
                        error: &Result<T, ::scrypto_test::prelude::RuntimeError>
                    ) -> bool {
                        matches!(
                            error,
                            Err(::scrypto_test::prelude::RuntimeError::ApplicationError(
                                ::scrypto_test::prelude::ApplicationError::PanicMessage(error)
                            ))
                            if error.contains($error)
                        )
                    }

                    pub fn [< assert_is_ $prefix:snake _ $error:lower:camel:snake >]<T>(
                        error: &Result<T, ::scrypto_test::prelude::RuntimeError>
                    )
                    where
                        T: Debug
                    {
                        assert!(
                            [< is_ $prefix:snake _ $error:lower:camel:snake >](error),
                            "Running \"{}\" against {:?} failed",
                            stringify!([< assert_is_ $prefix:snake _ $error:lower:camel:snake >]),
                            error
                        )
                    }
                )*
            )*
        }
    };
}

define_error_checking_functions! {
    ignition => [
        NO_ADAPTER_FOUND_FOR_POOL_ERROR,
        NEITHER_POOL_RESOURCE_IS_PROTOCOL_RESOURCE_ERROR,
        NEITHER_POOL_RESOURCE_IS_USER_RESOURCE_ERROR,
        NO_ASSOCIATED_VAULT_ERROR,
        NO_ASSOCIATED_LIQUIDITY_RECEIPT_VAULT_ERROR,
        NOT_AN_IGNITION_ADDRESS_ERROR,
        OPENING_LIQUIDITY_POSITIONS_IS_CLOSED_ERROR,
        CLOSING_LIQUIDITY_POSITIONS_IS_CLOSED_ERROR,
        NO_REWARDS_RATE_ASSOCIATED_WITH_LOCKUP_PERIOD_ERROR,
        POOL_IS_NOT_IN_ALLOW_LIST_ERROR,
        ORACLE_REPORTED_PRICE_IS_STALE_ERROR,
        LOCKUP_PERIOD_HAS_NO_ASSOCIATED_REWARDS_RATE_ERROR,
        UNEXPECTED_ERROR,
        RELATIVE_PRICE_DIFFERENCE_LARGER_THAN_ALLOWED_ERROR,
        USER_ASSET_DOES_NOT_BELONG_TO_POOL_ERROR,
        MORE_THAN_ONE_LIQUIDITY_RECEIPT_NFTS_ERROR,
        NOT_A_VALID_LIQUIDITY_RECEIPT_ERROR,
        LIQUIDITY_POSITION_HAS_NOT_MATURED_ERROR,
        USER_MUST_NOT_PROVIDE_PROTOCOL_ASSET_ERROR,
        USER_RESOURCES_VOLATILITY_UNKNOWN_ERROR,
        BOTH_POOL_ASSETS_ARE_THE_PROTOCOL_RESOURCE,
        OVERFLOW_ERROR,
        INVALID_MAXIMUM_PRICE_STALENESS,
        INVALID_UPFRONT_REWARD_PERCENTAGE,
    ],
    ociswap_adapter => [
        FAILED_TO_GET_RESOURCE_ADDRESSES_ERROR,
        FAILED_TO_GET_VAULT_ERROR,
        PRICE_IS_UNDEFINED
    ]
}
