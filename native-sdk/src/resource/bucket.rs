use radix_engine_interface::api::api::Invokable;
use radix_engine_interface::data::{ScryptoDecode, ScryptoTypeId};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::fmt::Debug;
use std::collections::BTreeSet;

pub trait SysBucket {
    fn sys_new<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        receiver: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: Invokable<ResourceManagerCreateBucketInvocation, E>;

    fn sys_amount<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        env: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: Invokable<BucketGetAmountInvocation, E>;

    fn sys_burn<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(self, env: &mut Y) -> Result<(), E>
    where
        Y: Invokable<ResourceManagerBurnInvocation, E>
            + Invokable<BucketGetResourceAddressInvocation, E>;

    fn sys_resource_address<Y, E>(&self, env: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: Invokable<BucketGetResourceAddressInvocation, E>,
        E: Debug + ScryptoTypeId + ScryptoDecode;

    fn sys_create_proof<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: Invokable<BucketCreateProofInvocation, E>;

    fn sys_non_fungible_ids<Y, E>(&self, env: &mut Y) -> Result<BTreeSet<NonFungibleId>, E>
    where
        Y: Invokable<BucketGetNonFungibleIdsInvocation, E>,
        E: Debug + ScryptoTypeId + ScryptoDecode;
}

impl SysBucket for Bucket {
    fn sys_new<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        receiver: ResourceAddress,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: Invokable<ResourceManagerCreateBucketInvocation, E>,
    {
        api.invoke(ResourceManagerCreateBucketInvocation { receiver })
    }

    fn sys_amount<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        env: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: Invokable<BucketGetAmountInvocation, E>,
    {
        env.invoke(BucketGetAmountInvocation { receiver: self.0 })
    }

    fn sys_burn<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(self, env: &mut Y) -> Result<(), E>
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

    fn sys_non_fungible_ids<Y, E>(&self, api: &mut Y) -> Result<BTreeSet<NonFungibleId>, E>
    where
        Y: Invokable<BucketGetNonFungibleIdsInvocation, E>,
        E: Debug + ScryptoTypeId + ScryptoDecode,
    {
        api.invoke(BucketGetNonFungibleIdsInvocation { receiver: self.0 })
    }

    fn sys_resource_address<Y, E>(&self, api: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: Invokable<BucketGetResourceAddressInvocation, E>,
        E: Debug + ScryptoTypeId + ScryptoDecode,
    {
        api.invoke(BucketGetResourceAddressInvocation { receiver: self.0 })
    }

    fn sys_create_proof<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: Invokable<BucketCreateProofInvocation, E>,
    {
        api.invoke(BucketCreateProofInvocation { receiver: self.0 })
    }
}
