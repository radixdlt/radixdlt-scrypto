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

use scrypto::prelude::*;
use scrypto_math::*;

pub const BASE: Decimal = dec!(1.0005);
pub const MIN_TICK: u32 = 0;
pub const MAX_TICK: u32 = 54000;

/// Converts a tick to spot price through checked math.
pub fn tick_to_spot(tick: u32) -> Option<Decimal> {
    if tick > MAX_TICK {
        None
    } else {
        BASE.checked_powi((tick as i64).checked_sub(27000)?.checked_mul(2)?)
    }
}

pub fn spot_to_tick(price: Decimal) -> Option<u32> {
    price
        .log_base(BASE)
        .and_then(|value| value.checked_div(dec!(2)))
        .and_then(|value| value.checked_add(dec!(27000)))
        .and_then(|value| value.0.checked_div(Decimal::ONE.0))
        .and_then(|value| u32::try_from(value).ok())
}
