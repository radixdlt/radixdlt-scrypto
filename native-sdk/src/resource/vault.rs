use radix_engine_interface::api::api::{EngineApi, Invokable, InvokableModel};
use radix_engine_interface::data::ScryptoDecode;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::fmt::Debug;

pub trait NativeVault: Sized {
    fn sys_new<Y, E: Debug + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: EngineApi<E> + InvokableModel<E>;

    fn sys_put<Y, E: Debug + ScryptoDecode>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: EngineApi<E> + InvokableModel<E>;

    fn sys_amount<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Decimal, E>
    where
        Y: EngineApi<E> + Invokable<VaultGetAmountInvocation, E>;
}

impl NativeVault for Vault {
    fn sys_new<Y, E: Debug + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: EngineApi<E> + InvokableModel<E>,
    {
        api.invoke(ResourceManagerCreateVaultInvocation {
            receiver: resource_address,
        })
    }

    fn sys_put<Y, E: Debug + ScryptoDecode>(&mut self, bucket: Bucket, api: &mut Y) -> Result<(), E>
    where
        Y: EngineApi<E> + InvokableModel<E>,
    {
        api.invoke(VaultPutInvocation {
            receiver: self.0,
            bucket,
        })
    }

    fn sys_amount<Y, E: Debug + ScryptoDecode>(&self, sys_calls: &mut Y) -> Result<Decimal, E>
    where
        Y: EngineApi<E> + Invokable<VaultGetAmountInvocation, E>,
    {
        sys_calls.invoke(VaultGetAmountInvocation { receiver: self.0 })
    }
}
