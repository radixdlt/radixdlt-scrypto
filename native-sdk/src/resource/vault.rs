use radix_engine_interface::api::types::VaultId;
use radix_engine_interface::api::{ClientNativeInvokeApi, ClientNodeApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoDecode;
use radix_engine_interface::math::Decimal;
use sbor::rust::fmt::Debug;

pub struct Vault(pub VaultId); // native stub

impl Vault {
    pub fn sys_new<Y, E: Debug + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        let vault_id = api
            .call_native(ResourceManagerCreateVaultInvocation {
                receiver: resource_address,
            })?
            .vault_id();

        Ok(Self(vault_id))
    }

    pub fn sys_put<Y, E: Debug + ScryptoDecode>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        api.call_native(VaultPutInvocation {
            receiver: self.0,
            bucket,
        })
    }

    pub fn sys_take<Y, E: Debug + ScryptoDecode>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        api.call_native(VaultTakeInvocation {
            receiver: self.0,
            amount,
        })
    }

    pub fn sys_amount<Y, E: Debug + ScryptoDecode>(&self, sys_calls: &mut Y) -> Result<Decimal, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        sys_calls.call_native(VaultGetAmountInvocation { receiver: self.0 })
    }

    pub fn sys_create_proof<Y, E>(&self, sys_calls: &mut Y) -> Result<Proof, E>
    where
        E: Debug + ScryptoDecode,
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        sys_calls.call_native(VaultCreateProofInvocation { receiver: self.0 })
    }
}
