use radix_engine_interface::api::types::{ScryptoReceiver, VaultId};
use radix_engine_interface::api::{
    ClientComponentApi, ClientNativeInvokeApi, ClientNodeApi, ClientSubstateApi,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::types::Own;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoDecode};
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

pub struct Vault(pub VaultId); // native stub

impl Vault {
    pub fn sys_new<Y, E: Debug + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientNodeApi<E>
            + ClientSubstateApi<E>
            + ClientNativeInvokeApi<E>
            + ClientComponentApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::Resource(resource_address),
            RESOURCE_MANAGER_CREATE_VAULT_IDENT,
            scrypto_encode(&ResourceManagerCreateVaultInput {}).unwrap(),
        )?;

        let own: Own = scrypto_decode(&rtn).unwrap();
        Ok(Self(own.vault_id()))
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
        Y: ClientNodeApi<E>
            + ClientSubstateApi<E>
            + ClientNativeInvokeApi<E>
            + ClientComponentApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::Vault(self.0),
            VAULT_TAKE_IDENT,
            scrypto_encode(&VaultTakeInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_take_all<Y, E: Debug + ScryptoDecode>(&mut self, api: &mut Y) -> Result<Bucket, E>
    where
        Y: ClientNodeApi<E>
            + ClientSubstateApi<E>
            + ClientNativeInvokeApi<E>
            + ClientComponentApi<E>,
    {
        // TODO: Replace with actual take all blueprint method
        let amount = self.sys_amount(api)?;
        let rtn = api.call_method(
            ScryptoReceiver::Vault(self.0),
            VAULT_TAKE_IDENT,
            scrypto_encode(&VaultTakeInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_take_non_fungibles<Y, E: Debug + ScryptoDecode>(
        &mut self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        api.call_native(VaultTakeNonFungiblesInvocation {
            receiver: self.0,
            non_fungible_local_ids: ids,
        })
    }

    pub fn sys_amount<Y, E: Debug + ScryptoDecode>(&self, sys_calls: &mut Y) -> Result<Decimal, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        sys_calls.call_native(VaultGetAmountInvocation { receiver: self.0 })
    }

    pub fn sys_create_proof<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Proof, E>
    where
        E: Debug + ScryptoDecode,
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        let amount = self.sys_amount(api)?;
        api.call_native(VaultCreateProofByAmountInvocation {
            receiver: self.0,
            amount,
        })
    }

    pub fn sys_create_proof_by_amount<Y, E: Debug + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        api.call_native(VaultCreateProofByAmountInvocation {
            receiver: self.0,
            amount,
        })
    }

    pub fn sys_create_proof_by_ids<Y, E: Debug + ScryptoDecode>(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        api.call_native(VaultCreateProofByIdsInvocation {
            receiver: self.0,
            ids,
        })
    }

    pub fn sys_lock_fee<Y, E: Debug + ScryptoDecode>(
        &mut self,
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), E>
    where
        Y: ClientNodeApi<E>
            + ClientSubstateApi<E>
            + ClientNativeInvokeApi<E>
            + ClientComponentApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::Vault(self.0),
            VAULT_LOCK_FEE_IDENT,
            scrypto_encode(&VaultLockFeeInput {
                amount,
                contingent: false,
            })
            .unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_lock_contingent_fee<Y, E: Debug + ScryptoDecode>(
        &mut self,
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), E>
    where
        Y: ClientNodeApi<E>
            + ClientSubstateApi<E>
            + ClientNativeInvokeApi<E>
            + ClientComponentApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::Vault(self.0),
            VAULT_LOCK_FEE_IDENT,
            scrypto_encode(&VaultLockFeeInput {
                amount,
                contingent: true,
            })
            .unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_resource_address<Y, E: Debug + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        api.call_native(VaultGetResourceAddressInvocation { receiver: self.0 })
    }
}
