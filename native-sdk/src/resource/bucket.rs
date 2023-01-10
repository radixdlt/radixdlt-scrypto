use radix_engine_interface::api::api::Invokable;
use radix_engine_interface::data::{ScryptoCategorize, ScryptoDecode};
use radix_engine_interface::model::*;
use sbor::rust::fmt::Debug;

pub trait SysBucket {
    fn sys_new<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        receiver: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: Invokable<ResourceManagerCreateBucketInvocation, E>;

    fn sys_burn<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        env: &mut Y,
    ) -> Result<(), E>
    where
        Y: Invokable<ResourceManagerBurnInvocation, E>
            + Invokable<BucketGetResourceAddressInvocation, E>;

    fn sys_resource_address<Y, E>(&self, env: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: Invokable<BucketGetResourceAddressInvocation, E>,
        E: Debug + ScryptoCategorize + ScryptoDecode;

    fn sys_create_proof<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: Invokable<BucketCreateProofInvocation, E>;
}

impl SysBucket for Bucket {
    fn sys_new<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        receiver: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: Invokable<ResourceManagerCreateBucketInvocation, E>,
    {
        sys_calls.invoke(ResourceManagerCreateBucketInvocation { receiver })
    }

    fn sys_burn<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(self, env: &mut Y) -> Result<(), E>
    where
        Y: Invokable<ResourceManagerBurnInvocation, E>
            + Invokable<BucketGetResourceAddressInvocation, E>,
    {
        let receiver = self.sys_resource_address(env)?;
        env.invoke(ResourceManagerBurnInvocation {
            receiver,
            bucket: Bucket(self.0),
        })
    }

    fn sys_resource_address<Y, E>(&self, env: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: Invokable<BucketGetResourceAddressInvocation, E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        env.invoke(BucketGetResourceAddressInvocation { receiver: self.0 })
    }

    fn sys_create_proof<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: Invokable<BucketCreateProofInvocation, E>,
    {
        sys_calls.invoke(BucketCreateProofInvocation { receiver: self.0 })
    }
}
