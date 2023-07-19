use radix_engine_interface::api::{ClientApi, ClientObjectApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoCategorize, ScryptoDecode,
};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

use super::ResourceManager;

// TODO: split impl

pub trait NativeBucket {
    fn create<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        receiver: ResourceAddress,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>;

    fn amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: ClientApi<E>;

    fn put<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        other: Self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;

    fn take<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>;

    fn take_advanced<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>;

    fn burn<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(self, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>;

    fn package_burn<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;

    fn resource_address<Y, E>(&self, api: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode;

    fn create_proof_of_all<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>;

    fn is_empty<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<bool, E>
    where
        Y: ClientApi<E>;

    fn drop_empty<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;
}

pub trait NativeFungibleBucket {
    fn create_proof_of_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>;
}

pub trait NativeNonFungibleBucket {
    fn non_fungible_local_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, E>
    where
        Y: ClientApi<E>;

    fn take_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>;

    fn create_proof_of_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>;
}

impl NativeBucket for Bucket {
    fn drop_empty<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
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

    fn create<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        receiver: ResourceAddress,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            receiver.as_node_id(),
            RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT,
            scrypto_encode(&ResourceManagerCreateEmptyBucketInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_GET_AMOUNT_IDENT,
            scrypto_encode(&BucketGetAmountInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn put<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        other: Self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let _rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_PUT_IDENT,
            scrypto_encode(&BucketPutInput { bucket: other }).unwrap(),
        )?;

        Ok(())
    }

    fn take<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_TAKE_IDENT,
            scrypto_encode(&BucketTakeInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn take_advanced<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
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

    fn burn<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(self, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let resource_address = self.resource_address(api)?;
        ResourceManager(resource_address).burn(Bucket(self.0), api)
    }

    fn package_burn<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let resource_address = self.resource_address(api)?;
        ResourceManager(resource_address).package_burn(Bucket(self.0), api)
    }

    fn resource_address<Y, E>(&self, api: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let resource_address = ResourceAddress::new_or_panic(
            api.get_outer_object(self.0.as_node_id())
                .unwrap()
                .as_node_id()
                .0,
        );

        Ok(resource_address)
    }

    fn create_proof_of_all<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_CREATE_PROOF_OF_ALL_IDENT,
            scrypto_encode(&BucketCreateProofOfAllInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn is_empty<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<bool, E>
    where
        Y: ClientApi<E>,
    {
        Ok(self.amount(api)?.is_zero())
    }
}

impl NativeFungibleBucket for Bucket {
    fn create_proof_of_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT,
            scrypto_encode(&FungibleBucketCreateProofOfAmountInput { amount }).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}

impl NativeNonFungibleBucket for Bucket {
    fn non_fungible_local_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
            scrypto_encode(&BucketGetNonFungibleLocalIdsInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn take_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_IDENT,
            scrypto_encode(&BucketTakeNonFungiblesInput { ids }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn create_proof_of_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            scrypto_encode(&NonFungibleBucketCreateProofOfNonFungiblesInput { ids }).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}
