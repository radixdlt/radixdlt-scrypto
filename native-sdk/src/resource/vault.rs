use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoDecode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

// TODO: split impl

pub trait NativeVault {
    fn create<Y, E: Debug + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Vault, E>
    where
        Y: ClientApi<E>;

    fn put<Y, E: Debug + ScryptoDecode>(&mut self, bucket: Bucket, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>;

    fn take<Y, E: Debug + ScryptoDecode>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>;

    fn take_all<Y, E: Debug + ScryptoDecode>(&mut self, api: &mut Y) -> Result<Bucket, E>
    where
        Y: ClientApi<E>;

    fn amount<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Decimal, E>
    where
        Y: ClientApi<E>;

    fn create_proof<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Proof, E>
    where
        Y: ClientApi<E>;

    fn create_proof_of_amount<Y, E: Debug + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>;

    fn resource_address<Y, E: Debug + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>
    where
        Y: ClientApi<E>;

    fn burn<Y, E: Debug + ScryptoDecode>(&mut self, amount: Decimal, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>;
}

pub trait NativeFungibleVault {
    fn lock_fee<Y, E: Debug + ScryptoDecode>(
        &mut self,
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;

    fn lock_contingent_fee<Y, E: Debug + ScryptoDecode>(
        &mut self,
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;
}

pub trait NativeNonFungibleVault {
    fn take_non_fungibles<Y, E: Debug + ScryptoDecode>(
        &mut self,
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>;

    fn create_proof_of_non_fungibles<Y, E: Debug + ScryptoDecode>(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>;

    fn burn_non_fungibles<Y, E: Debug + ScryptoDecode>(
        &mut self,
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;
}

impl NativeVault for Vault {
    fn create<Y, E: Debug + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Vault, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            resource_address.as_node_id(),
            RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT,
            scrypto_encode(&ResourceManagerCreateEmptyVaultInput {}).unwrap(),
        )?;

        let own: Own = scrypto_decode(&rtn).unwrap();
        Ok(Self(own))
    }

    fn put<Y, E: Debug + ScryptoDecode>(&mut self, bucket: Bucket, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_PUT_IDENT,
            scrypto_encode(&VaultPutInput { bucket }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn take<Y, E: Debug + ScryptoDecode>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_TAKE_IDENT,
            scrypto_encode(&VaultTakeInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn take_all<Y, E: Debug + ScryptoDecode>(&mut self, api: &mut Y) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
        // TODO: Replace with actual take all blueprint method
        let amount = self.amount(api)?;
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_TAKE_IDENT,
            scrypto_encode(&VaultTakeInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn amount<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Decimal, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_GET_AMOUNT_IDENT,
            scrypto_encode(&VaultGetAmountInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn create_proof<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_CREATE_PROOF_IDENT,
            scrypto_encode(&VaultCreateProofInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn create_proof_of_amount<Y, E: Debug + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_CREATE_PROOF_OF_AMOUNT_IDENT,
            scrypto_encode(&VaultCreateProofOfAmountInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn resource_address<Y, E: Debug + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>
    where
        Y: ClientApi<E>,
    {
        let info = api.get_object_info(self.0.as_node_id())?;
        Ok(ResourceAddress::try_from(info.get_outer_object().as_ref()).unwrap())
    }

    fn burn<Y, E: Debug + ScryptoDecode>(&mut self, amount: Decimal, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_BURN_IDENT,
            scrypto_encode(&VaultBurnInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }
}

impl NativeFungibleVault for Vault {
    fn lock_fee<Y, E: Debug + ScryptoDecode>(
        &mut self,
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            FUNGIBLE_VAULT_LOCK_FEE_IDENT,
            scrypto_encode(&FungibleVaultLockFeeInput {
                amount,
                contingent: false,
            })
            .unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn lock_contingent_fee<Y, E: Debug + ScryptoDecode>(
        &mut self,
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            FUNGIBLE_VAULT_LOCK_FEE_IDENT,
            scrypto_encode(&FungibleVaultLockFeeInput {
                amount,
                contingent: true,
            })
            .unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}

impl NativeNonFungibleVault for Vault {
    fn take_non_fungibles<Y, E: Debug + ScryptoDecode>(
        &mut self,
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT,
            scrypto_encode(&NonFungibleVaultTakeNonFungiblesInput {
                non_fungible_local_ids,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn create_proof_of_non_fungibles<Y, E: Debug + ScryptoDecode>(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            scrypto_encode(&NonFungibleVaultCreateProofOfNonFungiblesInput { ids }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn burn_non_fungibles<Y, E: Debug + ScryptoDecode>(
        &mut self,
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT,
            scrypto_encode(&NonFungibleVaultBurnNonFungiblesInput {
                non_fungible_local_ids,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }
}
