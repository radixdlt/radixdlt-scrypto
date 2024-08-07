use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::math::Decimal;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use sbor::rust::collections::IndexSet;

use super::{ResourceManager, SpecializedProof};

// TODO: Move the fungible/non-fungible parts out of NativeBucket,
//       and require the user opt in with `as_fungible` / `as_non_fungible` like in Scrypto.
//       This will be a breaking change, so likely need some communication.

pub trait NativeBucket {
    type ResourceManager;
    type Proof;

    fn create<Y: SystemApi<E>, E: SystemApiError>(
        receiver: ResourceAddress,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Self: Sized;

    fn amount<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Decimal, E>;

    fn put<Y: SystemApi<E>, E: SystemApiError>(&self, other: Self, api: &mut Y) -> Result<(), E>;

    fn take<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Self: Sized;

    fn take_advanced<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Self: Sized;

    fn burn<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E>;

    fn package_burn<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E>;

    fn resource_address<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>;

    fn resource_manager<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<Self::ResourceManager, E>;

    fn create_proof_of_all<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<Self::Proof, E>;

    fn is_empty<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<bool, E>;

    fn drop_empty<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E>;
}

pub trait NativeFungibleBucket {
    fn create_proof_of_amount<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<FungibleProof, E>;
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
    ) -> Result<NonFungibleBucket, E>;

    fn create_proof_of_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<NonFungibleProof, E>;
}

impl NativeBucket for Bucket {
    type ResourceManager = ResourceManager;
    type Proof = Proof;

    fn drop_empty<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E> {
        let resource_address = self.resource_address(api)?;
        let rtn = api.call_method(
            resource_address.as_node_id(),
            RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT,
            scrypto_encode(&ResourceManagerDropEmptyBucketInput { bucket: self }).unwrap(),
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

    fn resource_manager<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceManager, E> {
        Ok(ResourceManager(self.resource_address(api)?))
    }

    fn is_empty<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<bool, E> {
        Ok(self.amount(api)?.is_zero())
    }
}

pub trait SpecificBucket: AsRef<Bucket> + Into<Bucket> {
    type ResourceManager: From<ResourceManager>;
    type Proof: SpecializedProof;
    fn from_bucket_of_correct_type(bucket: Bucket) -> Self;
}

impl SpecificBucket for NonFungibleBucket {
    type ResourceManager = ResourceManager; // Change when we have a native NonFungibleResourceManager
    type Proof = NonFungibleProof;

    fn from_bucket_of_correct_type(bucket: Bucket) -> Self {
        Self(bucket)
    }
}

impl SpecificBucket for FungibleBucket {
    type ResourceManager = ResourceManager; // Change when we have a native FungibleResourceManager
    type Proof = FungibleProof;

    fn from_bucket_of_correct_type(bucket: Bucket) -> Self {
        Self(bucket)
    }
}

impl<T: SpecificBucket> NativeBucket for T {
    type Proof = <Self as SpecificBucket>::Proof;
    type ResourceManager = <Self as SpecificBucket>::ResourceManager;

    fn create<Y: SystemApi<E>, E: SystemApiError>(
        receiver: ResourceAddress,
        api: &mut Y,
    ) -> Result<Self, E> {
        Ok(Self::from_bucket_of_correct_type(Bucket::create(
            receiver, api,
        )?))
    }

    fn amount<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Decimal, E> {
        self.as_ref().amount(api)
    }

    fn put<Y: SystemApi<E>, E: SystemApiError>(&self, other: Self, api: &mut Y) -> Result<(), E> {
        self.as_ref().put(other.into(), api)
    }

    fn take<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Self: Sized,
    {
        let bucket = self.as_ref().take(amount, api)?;
        Ok(Self::from_bucket_of_correct_type(bucket))
    }

    fn take_advanced<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Self: Sized,
    {
        let bucket = self
            .as_ref()
            .take_advanced(amount, withdraw_strategy, api)?;
        Ok(Self::from_bucket_of_correct_type(bucket))
    }

    fn burn<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E> {
        self.into().burn(api)
    }

    fn package_burn<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E> {
        self.into().package_burn(api)
    }

    fn resource_address<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E> {
        self.as_ref().resource_address(api)
    }

    fn resource_manager<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<Self::ResourceManager, E> {
        Ok(self.as_ref().resource_manager(api)?.into())
    }

    fn create_proof_of_all<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<Self::Proof, E> {
        let proof = self.as_ref().create_proof_of_all(api)?;
        Ok(<Self::Proof as SpecializedProof>::from_proof_of_correct_type(proof))
    }

    fn is_empty<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<bool, E> {
        self.as_ref().is_empty(api)
    }

    fn drop_empty<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E> {
        self.into().drop_empty(api)
    }
}

impl NativeFungibleBucket for Bucket {
    fn create_proof_of_amount<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<FungibleProof, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT,
            scrypto_encode(&FungibleBucketCreateProofOfAmountInput { amount }).unwrap(),
        )?;
        Ok(FungibleProof(scrypto_decode(&rtn).unwrap()))
    }
}

impl NativeFungibleBucket for FungibleBucket {
    fn create_proof_of_amount<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<FungibleProof, E> {
        Ok(self.as_ref().create_proof_of_amount(amount, api)?.into())
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
    ) -> Result<NonFungibleBucket, E> {
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
    ) -> Result<NonFungibleProof, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            scrypto_encode(&NonFungibleBucketCreateProofOfNonFungiblesInput { ids }).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}

impl NativeNonFungibleBucket for NonFungibleBucket {
    fn non_fungible_local_ids<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, E> {
        self.as_ref().non_fungible_local_ids(api)
    }

    fn take_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<NonFungibleBucket, E> {
        self.as_ref().take_non_fungibles(ids, api)
    }

    fn create_proof_of_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<NonFungibleProof, E> {
        self.as_ref().create_proof_of_non_fungibles(ids, api)
    }
}
