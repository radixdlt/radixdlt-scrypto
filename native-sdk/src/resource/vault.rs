use radix_engine_interface::api::types::VaultId;
use radix_engine_interface::api::{EngineApi, Invokable, InvokableModel};
use radix_engine_interface::data::ScryptoDecode;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::fmt::Debug;

pub struct Vault(pub VaultId); // native stub

impl Vault {
    pub fn sys_new<Y, E: Debug + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: EngineApi<E> + InvokableModel<E>,
    {
        let vault_id = api
            .invoke(ResourceManagerCreateVaultInvocation {
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
        Y: EngineApi<E> + InvokableModel<E>,
    {
        api.invoke(VaultPutInvocation {
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
        Y: EngineApi<E> + InvokableModel<E>,
    {
        api.invoke(VaultTakeInvocation {
            receiver: self.0,
            amount,
        })
    }

    pub fn sys_amount<Y, E: Debug + ScryptoDecode>(&self, sys_calls: &mut Y) -> Result<Decimal, E>
    where
        Y: EngineApi<E> + Invokable<VaultGetAmountInvocation, E>,
    {
        sys_calls.invoke(VaultGetAmountInvocation { receiver: self.0 })
    }

    pub fn sys_create_proof<Y, E>(&self, sys_calls: &mut Y) -> Result<Proof, E>
    where
        E: Debug + ScryptoDecode,
        Y: EngineApi<E> + Invokable<VaultCreateProofInvocation, E>,
    {
        sys_calls.invoke(VaultCreateProofInvocation { receiver: self.0 })
    }
}
