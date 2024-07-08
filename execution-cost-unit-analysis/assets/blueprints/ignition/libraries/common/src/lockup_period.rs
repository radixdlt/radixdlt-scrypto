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
use std::ops::*;

use humantime::format_duration;

/// A type used for the lockup period that can be creates from various time
/// durations and that implements display in the desired way.
#[derive(Clone, Copy, Sbor, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[sbor(transparent)]
pub struct LockupPeriod(u64);

impl LockupPeriod {
    pub const fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    pub const fn from_minutes(minutes: u64) -> Option<Self> {
        let value = minutes.checked_mul(60);
        match value {
            Some(value) => Some(Self::from_seconds(value)),
            None => None,
        }
    }

    pub const fn from_hours(hours: u64) -> Option<Self> {
        let value = hours.checked_mul(60);
        match value {
            Some(value) => Self::from_minutes(value),
            None => None,
        }
    }

    pub const fn from_days(days: u64) -> Option<Self> {
        let value = days.checked_mul(24);
        match value {
            Some(value) => Self::from_hours(value),
            None => None,
        }
    }

    pub const fn from_weeks(weeks: u64) -> Option<Self> {
        let value = weeks.checked_mul(7);
        match value {
            Some(value) => Self::from_days(value),
            None => None,
        }
    }

    // One month approx 30.44 days
    pub const fn from_months(months: u64) -> Option<Self> {
        let value = months.checked_mul(2_630_016);
        match value {
            Some(value) => Some(Self::from_seconds(value)),
            None => None,
        }
    }

    pub const fn seconds(&self) -> &u64 {
        &self.0
    }
}

impl Display for LockupPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&format_duration(std::time::Duration::new(self.0, 0)), f)
    }
}

impl Debug for LockupPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} seconds", self.0)
    }
}

impl Deref for LockupPeriod {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
