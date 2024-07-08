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
use scrypto::prelude::*;

/// The data of the liquidity positions given to the users of Ignition.
#[derive(ScryptoSbor, Clone, Debug, PartialEq, Eq, NonFungibleData)]
pub struct LiquidityReceipt<T>
where
    T: ScryptoSbor,
{
    /* Metadata/NonFungibleData standard */
    pub name: String,

    /* Display Data - Just for wallet display, no logic depends on this. */
    /// A string of the lockup period of the liquidity provided through the
    /// protocol (e.g., "6 Months").
    pub lockup_period: String,

    /* Application data */
    /// The pool that the resources were contributed to.
    pub pool_address: ComponentAddress,

    /// The address of the resource that the user contributed through the
    /// protocol.
    pub user_resource_address: ResourceAddress,

    /// The amount of the resource that the user contributed through the
    /// protocol.
    pub user_contribution_amount: Decimal,

    /// The volatility classification of the user resource at the time when the
    /// liquidity position was opened. This will be used to later deposit any
    /// protocol assets back into the same vault.
    pub user_resource_volatility_classification: Volatility,

    /// The amount of XRD that was contributed by the Ignition protocol to
    /// match the users contribution.
    pub protocol_contribution_amount: Decimal,

    /// The date after which this liquidity position can be closed.
    pub maturity_date: Instant,

    /// This is adapter specific data passed by the adapter when a position is
    /// opened. This is information that the adapter expects to be passed back
    /// when a liquidity position is closed. This is used in calculating the
    /// fees.
    pub adapter_specific_information: T,
}

impl<T> LiquidityReceipt<T>
where
    T: ScryptoSbor,
{
    pub fn new(
        lockup_period: LockupPeriod,
        pool_address: ComponentAddress,
        user_resource_address: ResourceAddress,
        user_contribution_amount: Decimal,
        user_volatility_classification: Volatility,
        protocol_contribution_amount: Decimal,
        adapter_specific_information: T,
    ) -> Self {
        let maturity_date = Clock::current_time_rounded_to_minutes()
            .add_seconds(*lockup_period.seconds() as i64)
            .unwrap();

        Self {
            name: "Liquidity Contribution".to_owned(),
            lockup_period: lockup_period.to_string(),
            pool_address,
            user_resource_address,
            user_contribution_amount,
            maturity_date,
            protocol_contribution_amount,
            user_resource_volatility_classification:
                user_volatility_classification,
            adapter_specific_information,
        }
    }
}
