use radix_engine_interface::api::api::Invokable;
use radix_engine_interface::data::{ScryptoDecode, ScryptoTypeId};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::fmt::Debug;
use sbor::rust::collections::BTreeSet;

pub trait SysBucket {
    fn sys_new<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        receiver: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: Invokable<ResourceManagerCreateBucketInvocation, E>;

    fn sys_amount<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: Invokable<BucketGetAmountInvocation, E>;

    fn sys_total_ids<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleId>, E>
    where
        Y: Invokable<BucketGetNonFungibleIdsInvocation, E>;

    fn sys_put<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        other: Self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: Invokable<BucketPutInvocation, E>;

    fn sys_take<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: Invokable<BucketTakeInvocation, E>;

    fn sys_take_non_fungibles<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &mut self,
        ids: BTreeSet<NonFungibleId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: Invokable<BucketTakeNonFungiblesInvocation, E>;

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

    fn sys_is_empty<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<bool, E>
    where
        Y: Invokable<BucketGetAmountInvocation, E>;
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
        api: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: Invokable<BucketGetAmountInvocation, E>,
    {
        api.invoke(BucketGetAmountInvocation { receiver: self.0 })
    }

    fn sys_total_ids<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleId>, E>
    where
        Y: Invokable<BucketGetNonFungibleIdsInvocation, E>,
    {
        api.invoke(BucketGetNonFungibleIdsInvocation { receiver: self.0 })
    }

    fn sys_put<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        other: Self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: Invokable<BucketPutInvocation, E>,
    {
        api.invoke(BucketPutInvocation {
            receiver: self.0,
            bucket: other,
        })
    }

    fn sys_take<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: Invokable<BucketTakeInvocation, E>,
    {
        api.invoke(BucketTakeInvocation {
            receiver: self.0,
            amount,
        })
    }

    fn sys_take_non_fungibles<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &mut self,
        ids: BTreeSet<NonFungibleId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: Invokable<BucketTakeNonFungiblesInvocation, E>,
    {
        api.invoke(BucketTakeNonFungiblesInvocation {
            receiver: self.0,
            ids,
        })
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

    fn sys_resource_address<Y, E>(&self, env: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: Invokable<BucketGetResourceAddressInvocation, E>,
        E: Debug + ScryptoTypeId + ScryptoDecode,
    {
        env.invoke(BucketGetResourceAddressInvocation { receiver: self.0 })
    }

    fn sys_create_proof<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: Invokable<BucketCreateProofInvocation, E>,
    {
        sys_calls.invoke(BucketCreateProofInvocation { receiver: self.0 })
    }

    fn sys_is_empty<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<bool, E>
    where
        Y: Invokable<BucketGetAmountInvocation, E>,
    {
        Ok(api
            .invoke(BucketGetAmountInvocation { receiver: self.0 })?
            .is_zero())
    }
}
