use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::math::Decimal;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use sbor::rust::collections::IndexSet;

use super::ResourceManager;

// TODO: split impl

pub trait NativeBucket {
    fn create<Y: SystemApi<E>, E: SystemApiError>(
        receiver: ResourceAddress,
        api: &mut Y,
    ) -> Result<Bucket, E>;

    fn amount<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Decimal, E>;

    fn put<Y: SystemApi<E>, E: SystemApiError>(&self, other: Self, api: &mut Y) -> Result<(), E>;

    fn take<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>;

    fn take_advanced<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, E>;

    fn burn<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E>;

    fn package_burn<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E>;

    fn resource_address<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>;

    fn create_proof_of_all<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>;

    fn is_empty<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<bool, E>;

    fn drop_empty<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E>;
}

pub trait NativeFungibleBucket {
    fn create_proof_of_amount<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, E>;
}

pub trait NativeNonFungibleBucket {
    fn non_fungible_local_ids<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, E>;

    fn take_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>;

    fn create_proof_of_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, E>;
}

impl NativeBucket for Bucket {
    fn drop_empty<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E> {
        let resource_address = self.resource_address(api)?;
        let rtn = api.call_method(
            resource_address.as_node_id(),
            RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT,
            scrypto_encode(&ResourceManagerDropEmptyBucketInput {
                bucket: Bucket(self.0),
            })
            .unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn create<Y: SystemObjectApi<E>, E: SystemApiError>(
        receiver: ResourceAddress,
        api: &mut Y,
    ) -> Result<Bucket, E> {
        let rtn = api.call_method(
            receiver.as_node_id(),
            RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT,
            scrypto_encode(&ResourceManagerCreateEmptyBucketInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn amount<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Decimal, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_GET_AMOUNT_IDENT,
            scrypto_encode(&BucketGetAmountInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn put<Y: SystemApi<E>, E: SystemApiError>(&self, other: Self, api: &mut Y) -> Result<(), E> {
        let _rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_PUT_IDENT,
            scrypto_encode(&BucketPutInput { bucket: other }).unwrap(),
        )?;

        Ok(())
    }

    fn take<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_TAKE_IDENT,
            scrypto_encode(&BucketTakeInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn take_advanced<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_TAKE_ADVANCED_IDENT,
            scrypto_encode(&BucketTakeAdvancedInput {
                amount,
                withdraw_strategy,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn burn<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E> {
        let resource_address = self.resource_address(api)?;
        ResourceManager(resource_address).burn(Bucket(self.0), api)
    }

    fn package_burn<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E> {
        let resource_address = self.resource_address(api)?;
        ResourceManager(resource_address).package_burn(Bucket(self.0), api)
    }

    fn resource_address<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E> {
        let resource_address = ResourceAddress::new_or_panic(
            api.get_outer_object(self.0.as_node_id())?.as_node_id().0,
        );

        Ok(resource_address)
    }

    fn create_proof_of_all<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_CREATE_PROOF_OF_ALL_IDENT,
            scrypto_encode(&BucketCreateProofOfAllInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn is_empty<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<bool, E> {
        Ok(self.amount(api)?.is_zero())
    }
}

impl NativeFungibleBucket for Bucket {
    fn create_proof_of_amount<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT,
            scrypto_encode(&FungibleBucketCreateProofOfAmountInput { amount }).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}

impl NativeNonFungibleBucket for Bucket {
    fn non_fungible_local_ids<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
            scrypto_encode(&BucketGetNonFungibleLocalIdsInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn take_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_IDENT,
            scrypto_encode(&BucketTakeNonFungiblesInput { ids }).unwrap(),
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
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            scrypto_encode(&NonFungibleBucketCreateProofOfNonFungiblesInput { ids }).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}
