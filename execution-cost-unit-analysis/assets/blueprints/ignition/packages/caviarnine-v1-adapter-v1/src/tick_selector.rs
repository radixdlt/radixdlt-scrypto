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

//! A module implementing the logic for the selection of the ticks to contribute
//! liquidity to based on the current active bin, the bin span, and the maximum
//! number of allowed ticks.

/// Ticks are in the range [0, 54000].
const MINIMUM_TICK_VALUE: usize = 0;
const MAXIMUM_TICK_VALUE: usize = 54000;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SelectedTicks {
    pub active_tick: u32,
    pub lowest_tick: u32,
    pub highest_tick: u32,
    pub lower_ticks: Vec<u32>,
    pub higher_ticks: Vec<u32>,
}

impl SelectedTicks {
    /// Selects the ticks that the positions should be made out of.
    ///
    /// Given the pool's active bin, bin span, and the preferred number of ticks
    /// we wish to have, this function determines which ticks the liquidity
    /// should go to without determining the exact formation of the liquidity
    /// (e.g., flat or triangle). This function is specifically suited to handle
    /// edge cases when selecting the bin that could lead to failures, and
    /// handles such cases in a correct graceful manner.
    ///
    /// In simple cases where the active bin is in the middle of the bin range
    /// and the bin span is small enough, the selection of the lower and higher
    /// ticks is simple enough: preferred number of ticks (divided by 2) to the
    /// left and the same on the right. This gives the caller the number of
    /// ticks that they wish to have on the left and the right of the active
    /// bin.
    ///
    /// There are cases however where the number of lower ticks can't be equal
    /// to the number of higher ticks. Specifically, cases when the active
    /// bin's value is too small or too large or cases when the bin span is
    /// too large. In such cases, this function attempts to compensate the
    /// other side. As an example, if we wish to have 10 ticks and can only
    /// have 2 lower ticks, then the higher ticks will have 8, thus
    /// achieving the preferred number of lower and higher ticks specified
    /// by the caller. A similar thing happens if the current active bin is
    /// too close to the maximum.
    ///
    /// There are cases when the proffered number of ticks can not be achieved
    /// by the function, specifically, cases when the bin span is too large
    /// that any bin to the right or the left will be outside of the range
    /// is allowed ticks. In such cases, this function returns a number of
    /// left and right ticks that is less than the desired.
    ///
    /// # Examples
    ///
    /// This section has examples with concrete numbers to to explain the
    /// behavior of this function better.
    ///
    /// ## Example 1: Simple Case
    ///
    /// * `active_tick`: 100
    /// * `bin_span`: 10
    /// * `preferred_total_number_of_higher_and_lower_ticks`: 4
    ///
    /// This function will return the following:
    ///
    /// * `active_tick`: 100
    /// * `lower_ticks`: [90, 80]
    /// * `higher_ticks`: [110, 120]
    ///
    /// ## Example 2: Left Skew
    ///
    /// * `active_tick`: 20
    /// * `bin_span`: 10
    /// * `preferred_total_number_of_higher_and_lower_ticks`: 6
    ///
    /// This function will return the following:
    ///
    /// * `active_tick`: 20
    /// * `lower_ticks`: [10, 0]
    /// * `higher_ticks`: [30, 40, 50, 60]
    ///
    /// At this currently active bin, there can only exist 2 ticks on the lower
    /// side. Thus, these ticks are selected and left's remaining share of the
    /// ticks is given to the right. This results in a total of 6 ticks.
    ///
    /// ## Example 3: Right Skew
    ///
    /// * `active_tick`: 53980
    /// * `bin_span`: 10
    /// * `preferred_total_number_of_higher_and_lower_ticks`: 6
    ///
    /// This function will return the following:
    ///
    /// * `active_tick`: 53980
    /// * `lower_ticks`: [53970, 53960, 53950, 53940]
    /// * `higher_ticks`: [53990, 54000]
    ///
    /// At this currently active bin, there can only exist 2 ticks on the higher
    /// side. Thus, these ticks are selected and right's remaining share of the
    /// ticks is given to the left. This results in a total of 6 ticks.
    ///
    /// Example 4: Bin Size too large
    ///
    /// * `active_tick`: 27000
    /// * `bin_span`: 54000
    /// * `preferred_total_number_of_higher_and_lower_ticks`: 6
    ///
    /// This function will return the following:
    ///
    /// * `active_tick`: 27000
    /// * `lower_ticks`: []
    /// * `higher_ticks`: []
    ///
    /// Given this pool's bin span, we can not get any ticks that are lower or
    /// higher and so we return just the active bin and return no lower or
    /// higher ticks.
    ///
    /// Example 4: Bin Size too large with a skew
    ///
    /// * `active_tick`: 54000
    /// * `bin_span`: 30000
    /// * `preferred_total_number_of_higher_and_lower_ticks`: 6
    ///
    /// This function will return the following:
    ///
    /// * `active_tick`: 54000
    /// * `lower_ticks`: [24000]
    /// * `higher_ticks`: []
    ///
    /// # Arguments:
    ///
    /// * `active_tick`: [`u32`] - The pool's currently active bin.
    /// * `bin_span`: [`u32`] - The span between each bin and another or the
    /// distance between them.
    /// * `preferred_total_number_of_higher_and_lower_ticks`: [`u32`] - The
    /// total number of ticks the caller wishes to have on the right and the
    /// left (summed). As detailed above, this may or may not be achieved
    /// depending on the pool's current bin and bin span.
    ///
    /// # Returns:
    ///
    /// [`SelectedTicks`] - A struct with the ticks that have been selected by
    /// this function.
    // TODO: Look into making this non-iterative.
    pub fn select(
        active_tick: u32,
        bin_span: u32,
        preferred_total_number_of_higher_and_lower_ticks: u32,
    ) -> Self {
        let mut selected_ticks = Self {
            active_tick,
            higher_ticks: vec![],
            lower_ticks: vec![],
            lowest_tick: active_tick,
            highest_tick: active_tick,
        };

        let mut remaining = preferred_total_number_of_higher_and_lower_ticks;

        let mut forward_counter =
            BoundedU32::<MINIMUM_TICK_VALUE, MAXIMUM_TICK_VALUE>(active_tick);
        let mut backward_counter =
            BoundedU32::<MINIMUM_TICK_VALUE, MAXIMUM_TICK_VALUE>(active_tick);

        while remaining > 0 {
            let mut forward_counter_incremented = false;
            let mut backward_counter_decremented = false;

            if forward_counter.checked_add_assign(bin_span).is_some() {
                remaining = remaining
                    .checked_sub(1)
                    .expect("Impossible, we do the check somewhere else");
                selected_ticks.higher_ticks.push(forward_counter.0);
                forward_counter_incremented = true;
            }
            if remaining > 0
                && backward_counter.checked_sub_assign(bin_span).is_some()
            {
                remaining = remaining
                    .checked_sub(1)
                    .expect("Impossible, we do the check somewhere else");
                selected_ticks.lower_ticks.push(backward_counter.0);
                backward_counter_decremented = true;
            }

            if !forward_counter_incremented && !backward_counter_decremented {
                break;
            }
        }

        selected_ticks.highest_tick = forward_counter.0;
        selected_ticks.lowest_tick = backward_counter.0;

        selected_ticks
    }
}

struct BoundedU32<const MIN: usize, const MAX: usize>(u32);

impl<const MIN: usize, const MAX: usize> BoundedU32<MIN, MAX> {
    pub fn checked_add_assign(&mut self, other: impl Into<u32>) -> Option<()> {
        if let Some(value) = self.checked_add(other) {
            *self = value;
            Some(())
        } else {
            None
        }
    }

    pub fn checked_sub_assign(&mut self, other: impl Into<u32>) -> Option<()> {
        if let Some(value) = self.checked_sub(other) {
            *self = value;
            Some(())
        } else {
            None
        }
    }

    pub fn checked_add(&self, other: impl Into<u32>) -> Option<Self> {
        let this = self.0;
        let other = other.into();

        if let Some(result) = this.checked_add(other) {
            if result as usize > MAX {
                None
            } else {
                Some(Self(result))
            }
        } else {
            None
        }
    }

    pub fn checked_sub(&self, other: impl Into<u32>) -> Option<Self> {
        let this = self.0;
        let other = other.into();

        if let Some(result) = this.checked_sub(other) {
            if (result as usize).lt(&MIN) {
                None
            } else {
                Some(Self(result))
            }
        } else {
            None
        }
    }
}
