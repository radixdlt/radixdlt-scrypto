use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::math::Decimal;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use sbor::rust::collections::IndexSet;

// TODO: split impl

pub trait NativeVault {
    fn create<Y: SystemApi<E>, E: SystemApiError>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Vault, E>;

    fn put<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), E>;

    fn take<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>;

    fn take_advanced<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, E>;

    fn take_all<Y: SystemApi<E>, E: SystemApiError>(&mut self, api: &mut Y) -> Result<Bucket, E>;

    fn amount<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Decimal, E>;

    fn resource_address<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>;

    fn burn<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), E>;
}

pub trait NativeFungibleVault {
    fn lock_fee<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), E>;

    fn lock_contingent_fee<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), E>;

    fn create_proof_of_amount<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, E>;
}

pub trait NativeNonFungibleVault {
    fn non_fungible_local_ids<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        limit: u32,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, E>;

    fn take_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        non_fungible_local_ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>;

    fn create_proof_of_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, E>;

    fn burn_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        non_fungible_local_ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), E>;

    fn contains_non_fungible<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        local_id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<bool, E>;
}

impl NativeVault for Vault {
    fn create<Y: SystemApi<E>, E: SystemApiError>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Vault, E> {
        let rtn = api.call_method(
            resource_address.as_node_id(),
            RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT,
            scrypto_encode(&ResourceManagerCreateEmptyVaultInput {}).unwrap(),
        )?;

        let own: Own = scrypto_decode(&rtn).unwrap();
        Ok(Self(own))
    }

    fn put<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_PUT_IDENT,
            scrypto_encode(&VaultPutInput { bucket }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn take<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_TAKE_IDENT,
            scrypto_encode(&VaultTakeInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn take_advanced<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_TAKE_ADVANCED_IDENT,
            scrypto_encode(&VaultTakeAdvancedInput {
                amount,
                withdraw_strategy,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn take_all<Y: SystemApi<E>, E: SystemApiError>(&mut self, api: &mut Y) -> Result<Bucket, E> {
        // TODO: Replace with actual take all blueprint method
        let amount = self.amount(api)?;
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_TAKE_IDENT,
            scrypto_encode(&VaultTakeInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn amount<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Decimal, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_GET_AMOUNT_IDENT,
            scrypto_encode(&VaultGetAmountInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn resource_address<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E> {
        let address = api.get_outer_object(self.0.as_node_id())?;
        Ok(ResourceAddress::try_from(address.into_node_id().0).unwrap())
    }

    fn burn<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VAULT_BURN_IDENT,
            scrypto_encode(&VaultBurnInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }
}

impl NativeFungibleVault for Vault {
    fn lock_fee<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), E> {
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

    fn lock_contingent_fee<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        api: &mut Y,
        amount: Decimal,
    ) -> Result<(), E> {
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

    fn create_proof_of_amount<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_IDENT,
            scrypto_encode(&FungibleVaultCreateProofOfAmountInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }
}

impl NativeNonFungibleVault for Vault {
    fn take_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        non_fungible_local_ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E> {
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

    fn create_proof_of_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            scrypto_encode(&NonFungibleVaultCreateProofOfNonFungiblesInput { ids }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn burn_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        non_fungible_local_ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), E> {
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

    fn non_fungible_local_ids<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        limit: u32,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
            scrypto_encode(&NonFungibleVaultGetNonFungibleLocalIdsInput { limit }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn contains_non_fungible<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        local_id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<bool, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_VAULT_CONTAINS_NON_FUNGIBLE_IDENT,
            scrypto_encode(&NonFungibleVaultContainsNonFungibleInput { id: local_id }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }
}
