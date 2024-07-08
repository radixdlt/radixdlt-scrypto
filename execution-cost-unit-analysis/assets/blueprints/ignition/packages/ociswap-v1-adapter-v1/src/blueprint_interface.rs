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
use scrypto_interface::*;

define_interface! {
    BasicPool as OciswapV1Pool impl [
        ScryptoStub,
        ScryptoTestStub,
        #[cfg(feature = "manifest-builder-stubs")]
        ManifestBuilderStub
    ] {
        fn instantiate(
            a_address: ResourceAddress,
            b_address: ResourceAddress,
            input_fee_rate: Decimal,
            dapp_definition: ComponentAddress,
        ) -> Self;
        fn instantiate_with_liquidity(
            #[manifest_type = "ManifestBucket"]
            a_bucket: Bucket,
            #[manifest_type = "ManifestBucket"]
            b_bucket: Bucket,
            input_fee_rate: Decimal,
            dapp_definition: ComponentAddress,
        ) -> (Self, Bucket, Option<Bucket>);
        fn add_liquidity(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            a_bucket: Bucket,
            #[manifest_type = "ManifestBucket"]
            b_bucket: Bucket
        ) -> (Bucket, Option<Bucket>);
        fn remove_liquidity(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            lp_token: Bucket
        ) -> (Bucket, Bucket);
        fn swap(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            input_bucket: Bucket
        ) -> Bucket;
        fn price_sqrt(&mut self) -> Option<PreciseDecimal>;
        fn liquidity_pool(&self) -> ComponentAddress;
        fn set_liquidity_pool_meta(
            &self,
            pool_address: ComponentAddress,
            lp_address: ResourceAddress,
            dapp_definition: ComponentAddress,
        );
        fn increase_observation_capacity(&mut self, new_capacity: u16);
    }
}

define_interface! {
    TwoResourcePool impl [ScryptoStub, ScryptoTestStub] {
        fn instantiate(
            owner_role: OwnerRole,
            pool_manager_rule: AccessRule,
            resource_addresses: (ResourceAddress, ResourceAddress),
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Self;
        fn contribute(&mut self, buckets: (Bucket, Bucket)) -> (Bucket, Option<Bucket>);
        fn redeem(&mut self, bucket: Bucket) -> (Bucket, Bucket);
        fn protected_deposit(&mut self, bucket: Bucket);
        fn protected_withdraw(
            &mut self,
            resource_address: ResourceAddress,
            amount: Decimal,
            withdraw_strategy: WithdrawStrategy,
        ) -> Bucket;
        fn get_redemption_value(
            &self,
            amount_of_pool_units: Decimal,
        ) -> IndexMap<ResourceAddress, Decimal>;
        fn get_vault_amounts(&self) -> IndexMap<ResourceAddress, Decimal>;
    }
}
