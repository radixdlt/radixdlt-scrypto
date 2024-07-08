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

//! Defines the interface that oracles must implement to be callable from
//! project ignition. This interface can be implemented by the oracles
//! or their adapters.

use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    OracleAdapter impl [
        #[cfg(feature = "trait")]
        Trait,
        #[cfg(feature = "scrypto-stubs")]
        ScryptoStub,
        #[cfg(feature = "scrypto-test-stubs")]
        ScryptoTestStub,
        #[cfg(feature = "manifest-builder-stubs")]
        ManifestBuilderStub
    ] {
        /// Gets the price of one asset in terms of another.
        ///
        /// Returns the price of the provided base and quote assets. This is the
        /// amount of the quote required to buy one of the base, so the units
        /// are actually reversed from the standard Base/Quote representation.
        ///
        /// # Arguments
        ///
        /// `base`: [`ResourceAddress`] - The address of the base asset.
        /// `quote`: [`ResourceAddress`] - The address of the quote asset.
        ///
        /// # Returns
        ///
        /// [`Decimal`] - The price of the asset. If the caller desires a
        /// [`Price`] object then its their responsibility to construct it.
        /// [`Instant`] - The instant when the price was updated, used in
        /// staleness calculations.
        fn get_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
        ) -> (Decimal, Instant);
    }
}
