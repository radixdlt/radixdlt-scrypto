use radix_engine_interface::api::api::SysNativeInvokable;
use radix_engine_interface::data::{ScryptoDecode, ScryptoTypeId};
use radix_engine_interface::model::*;
use sbor::rust::fmt::Debug;

pub trait SysBucket {
    fn sys_new<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        receiver: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: SysNativeInvokable<ResourceManagerCreateBucketInvocation, E>;

    fn sys_burn<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(self, env: &mut Y) -> Result<(), E>
    where
        Y: SysNativeInvokable<ResourceManagerBurnInvocation, E>
            + SysNativeInvokable<BucketGetResourceAddressInvocation, E>;

    fn sys_resource_address<Y, E>(&self, env: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: SysNativeInvokable<BucketGetResourceAddressInvocation, E>,
        E: Debug + ScryptoTypeId + ScryptoDecode;

    fn sys_create_proof<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: SysNativeInvokable<BucketCreateProofInvocation, E>;
}

impl SysBucket for Bucket {
    fn sys_new<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        receiver: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: SysNativeInvokable<ResourceManagerCreateBucketInvocation, E>,
    {
        sys_calls.sys_invoke(ResourceManagerCreateBucketInvocation { receiver })
    }

    fn sys_burn<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(self, env: &mut Y) -> Result<(), E>
    where
        Y: SysNativeInvokable<ResourceManagerBurnInvocation, E>
            + SysNativeInvokable<BucketGetResourceAddressInvocation, E>,
    {
        let receiver = self.sys_resource_address(env)?;
        env.sys_invoke(ResourceManagerBurnInvocation {
            receiver,
            bucket: Bucket(self.0),
        })
    }

    fn sys_resource_address<Y, E>(&self, env: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: SysNativeInvokable<BucketGetResourceAddressInvocation, E>,
        E: Debug + ScryptoTypeId + ScryptoDecode,
    {
        env.sys_invoke(BucketGetResourceAddressInvocation { receiver: self.0 })
    }

    fn sys_create_proof<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: SysNativeInvokable<BucketCreateProofInvocation, E>,
    {
        sys_calls.sys_invoke(BucketCreateProofInvocation { receiver: self.0 })
    }
}
