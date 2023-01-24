use radix_engine_interface::api::blueprints::resource::*;
use radix_engine_interface::api::Invokable;
use radix_engine_interface::data::ScryptoDecode;
use sbor::rust::fmt::Debug;

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub ResourceAddress);

impl ResourceManager {
    pub fn new_empty_bucket<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Bucket, E>
    where
        Y: Invokable<ResourceManagerCreateBucketInvocation, E>,
    {
        api.invoke(ResourceManagerCreateBucketInvocation { receiver: self.0 })
    }
}
