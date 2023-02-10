use radix_engine_interface::api::ClientNativeInvokeApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::{ScryptoCategorize, ScryptoDecode};
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

pub trait SysBucket {
    fn sys_new<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        receiver: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientNativeInvokeApi<E>;

    fn sys_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: ClientNativeInvokeApi<E>;

    fn sys_total_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, E>
    where
        Y: ClientNativeInvokeApi<E>;

    fn sys_put<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        other: Self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientNativeInvokeApi<E>;

    fn sys_take<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientNativeInvokeApi<E>;

    fn sys_take_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &mut self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientNativeInvokeApi<E>;

    fn sys_burn<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientNativeInvokeApi<E> + ClientNativeInvokeApi<E>;

    fn sys_resource_address<Y, E>(&self, api: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: ClientNativeInvokeApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode;

    fn sys_create_proof<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNativeInvokeApi<E>;

    fn sys_is_empty<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<bool, E>
    where
        Y: ClientNativeInvokeApi<E>;
}

impl SysBucket for Bucket {
    fn sys_new<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        receiver: ResourceAddress,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientNativeInvokeApi<E>,
    {
        api.call_native(ResourceManagerCreateBucketInvocation { receiver })
    }

    fn sys_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: ClientNativeInvokeApi<E>,
    {
        api.call_native(BucketGetAmountInvocation { receiver: self.0 })
    }

    fn sys_total_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, E>
    where
        Y: ClientNativeInvokeApi<E>,
    {
        api.call_native(BucketGetNonFungibleLocalIdsInvocation { receiver: self.0 })
    }

    fn sys_put<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        other: Self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientNativeInvokeApi<E>,
    {
        api.call_native(BucketPutInvocation {
            receiver: self.0,
            bucket: other,
        })
    }

    fn sys_take<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientNativeInvokeApi<E>,
    {
        api.call_native(BucketTakeInvocation {
            receiver: self.0,
            amount,
        })
    }

    fn sys_take_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &mut self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientNativeInvokeApi<E>,
    {
        api.call_native(BucketTakeNonFungiblesInvocation {
            receiver: self.0,
            ids,
        })
    }

    fn sys_burn<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(self, api: &mut Y) -> Result<(), E>
    where
        Y: ClientNativeInvokeApi<E> + ClientNativeInvokeApi<E>,
    {
        let receiver = self.sys_resource_address(api)?;
        api.call_native(ResourceManagerBurnInvocation {
            receiver,
            bucket: Bucket(self.0),
        })
    }

    fn sys_resource_address<Y, E>(&self, api: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: ClientNativeInvokeApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.call_native(BucketGetResourceAddressInvocation { receiver: self.0 })
    }

    fn sys_create_proof<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNativeInvokeApi<E>,
    {
        api.call_native(BucketCreateProofInvocation { receiver: self.0 })
    }

    fn sys_is_empty<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<bool, E>
    where
        Y: ClientNativeInvokeApi<E>,
    {
        Ok(api
            .call_native(BucketGetAmountInvocation { receiver: self.0 })?
            .is_zero())
    }
}
