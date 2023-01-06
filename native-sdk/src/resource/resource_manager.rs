use crate::runtime::Runtime;
use radix_engine_interface::api::api::{EngineApi, Invokable, InvokableModel};
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use std::collections::BTreeMap;
use std::fmt::Debug;

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub ResourceAddress);

impl ResourceManager {
    pub fn new_empty_bucket<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Bucket, E> where
        Y: Invokable<ResourceManagerCreateBucketInvocation, E> {
        api.invoke(ResourceManagerCreateBucketInvocation {
            receiver: self.0,
        })
    }
}
