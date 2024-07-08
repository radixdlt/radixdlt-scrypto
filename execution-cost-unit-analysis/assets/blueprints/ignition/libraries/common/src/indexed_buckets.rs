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

/// Buckets indexed and aggregated by the resource address.
#[derive(Debug, ScryptoSbor)]
pub struct IndexedBuckets(IndexMap<ResourceAddress, Bucket>);

impl IndexedBuckets {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn from_bucket(bucket: impl Into<Bucket>) -> Self {
        let mut this = Self::new();
        this.insert(bucket);
        this
    }

    pub fn from_buckets(
        buckets: impl IntoIterator<Item = impl Into<Bucket>>,
    ) -> Self {
        let mut this = Self::new();
        for bucket in buckets.into_iter() {
            this.insert(bucket);
        }
        this
    }

    pub fn insert(&mut self, bucket: impl Into<Bucket>) {
        let bucket = bucket.into();
        let resource_address = bucket.resource_address();
        if let Some(existing_bucket) = self.0.get_mut(&resource_address) {
            existing_bucket.put(bucket)
        } else {
            self.0.insert(resource_address, bucket);
        };
    }

    pub fn native_from_bucket<Y, E>(
        bucket: impl Into<Bucket>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: SystemApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let mut this = Self::new();
        this.native_insert(bucket, api)?;
        Ok(this)
    }
    pub fn native_from_buckets<Y, E>(
        buckets: impl IntoIterator<Item = impl Into<Bucket>>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: SystemApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let mut this = Self::new();
        for bucket in buckets.into_iter() {
            this.native_insert(bucket, api)?;
        }
        Ok(this)
    }
    pub fn native_insert<Y, E>(
        &mut self,
        bucket: impl Into<Bucket>,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: SystemApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let bucket = bucket.into();
        let resource_address =
            radix_native_sdk::resource::NativeBucket::resource_address(
                &bucket, api,
            )?;
        if let Some(existing_bucket) = self.0.get(&resource_address) {
            radix_native_sdk::resource::NativeBucket::put(
                existing_bucket,
                bucket,
                api,
            )?;
        } else {
            self.0.insert(resource_address, bucket);
        };
        Ok(())
    }

    pub fn get(&self, resource_address: &ResourceAddress) -> Option<&Bucket> {
        self.0.get(resource_address)
    }

    pub fn get_mut(
        &mut self,
        resource_address: &ResourceAddress,
    ) -> Option<&mut Bucket> {
        self.0.get_mut(resource_address)
    }

    pub fn keys(&self) -> impl Iterator<Item = &ResourceAddress> {
        self.0.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &Bucket> {
        self.0.values()
    }

    pub fn into_values(self) -> impl Iterator<Item = Bucket> {
        self.0.into_values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Bucket> {
        self.0.values_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn remove(
        &mut self,
        resource_address: &ResourceAddress,
    ) -> Option<Bucket> {
        self.0.swap_remove(resource_address)
    }

    pub fn into_inner(self) -> IndexMap<ResourceAddress, Bucket> {
        self.0
    }

    pub fn combine(mut self, other: Self) -> Self {
        for bucket in other.0.into_values() {
            self.insert(bucket)
        }
        self
    }
}

impl Default for IndexedBuckets {
    fn default() -> Self {
        Self::new()
    }
}
