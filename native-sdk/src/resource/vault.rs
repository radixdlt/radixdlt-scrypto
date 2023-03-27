use radix_engine_interface::api::types::{NodeId, ObjectId};
use radix_engine_interface::api::{ClientApi, ClientObjectApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoDecode};
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

pub struct Vault(pub ObjectId); // native stub

impl Vault {
    pub fn sys_new<Y, E: Debug + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientSubstateApi<E> + ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            &NodeId::GlobalObject(resource_address.into()),
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
        Y: ClientSubstateApi<E> + ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            &NodeId::Object(self.0),
            VAULT_PUT_IDENT,
            scrypto_encode(&VaultPutInput { bucket }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_take<Y, E: Debug + ScryptoDecode>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientSubstateApi<E> + ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            &NodeId::Object(self.0),
            VAULT_TAKE_IDENT,
            scrypto_encode(&VaultTakeInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_take_all<Y, E: Debug + ScryptoDecode>(&mut self, api: &mut Y) -> Result<Bucket, E>
    where
        Y: ClientSubstateApi<E> + ClientObjectApi<E>,
    {
        // TODO: Replace with actual take all blueprint method
        let amount = self.sys_amount(api)?;
        let rtn = api.call_method(
            &NodeId::Object(self.0),
            VAULT_TAKE_IDENT,
            scrypto_encode(&VaultTakeInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_take_non_fungibles<Y, E: Debug + ScryptoDecode>(
        &mut self,
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientSubstateApi<E> + ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            &NodeId::Object(self.0),
            VAULT_TAKE_NON_FUNGIBLES_IDENT,
            scrypto_encode(&VaultTakeNonFungiblesInput {
                non_fungible_local_ids,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_amount<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Decimal, E>
    where
        Y: ClientSubstateApi<E> + ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            &NodeId::Object(self.0),
            VAULT_GET_AMOUNT_IDENT,
            scrypto_encode(&VaultGetAmountInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_create_proof<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Proof, E>
    where
        E: Debug + ScryptoDecode,
        Y: ClientSubstateApi<E> + ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            &NodeId::Object(self.0),
            VAULT_CREATE_PROOF_IDENT,
            scrypto_encode(&VaultCreateProofInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_create_proof_by_amount<Y, E: Debug + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            &NodeId::Object(self.0),
            VAULT_CREATE_PROOF_BY_AMOUNT_IDENT,
            scrypto_encode(&VaultCreateProofByAmountInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_create_proof_by_ids<Y, E: Debug + ScryptoDecode>(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            &NodeId::Object(self.0),
            VAULT_CREATE_PROOF_BY_IDS_IDENT,
            scrypto_encode(&VaultCreateProofByIdsInput { ids }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_lock_fee<Y, E: Debug + ScryptoDecode>(
        &mut self,
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), E>
    where
        Y: ClientSubstateApi<E> + ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            &NodeId::Object(self.0),
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
        Y: ClientSubstateApi<E> + ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            &NodeId::Object(self.0),
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
        Y: ClientSubstateApi<E> + ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            &NodeId::Object(self.0),
            VAULT_GET_RESOURCE_ADDRESS_IDENT,
            scrypto_encode(&VaultGetResourceAddressInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }
}
