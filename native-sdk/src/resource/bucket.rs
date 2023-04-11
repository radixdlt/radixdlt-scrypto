use radix_engine_interface::api::{ClientApi, ClientObjectApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::RESOURCE_MANAGER_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoCategorize, ScryptoDecode,
};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

pub trait SysBucket {
    fn sys_new<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        receiver: ResourceAddress,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>;

    fn sys_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: ClientApi<E>;

    fn sys_non_fungible_local_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, E>
    where
        Y: ClientApi<E>;

    fn sys_put<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        other: Self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;

    fn sys_take<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>;

    fn sys_take_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>;

    fn sys_burn<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;

    fn sys_resource_address<Y, E>(&self, api: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode;

    fn sys_create_proof<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>;

    fn sys_is_empty<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<bool, E>
    where
        Y: ClientApi<E>;

    fn sys_drop_empty<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;
}

impl SysBucket for Bucket {
    fn sys_drop_empty<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_function(
            RESOURCE_MANAGER_PACKAGE,
            BUCKET_BLUEPRINT,
            BUCKET_DROP_EMPTY_IDENT,
            scrypto_encode(&BucketDropEmptyInput {
                bucket: Bucket(self.0),
            })
            .unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn sys_new<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        receiver: ResourceAddress,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            receiver.as_node_id(),
            RESOURCE_MANAGER_CREATE_BUCKET_IDENT,
            scrypto_encode(&ResourceManagerCreateBucketInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn sys_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
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

    fn sys_non_fungible_local_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
            scrypto_encode(&BucketGetNonFungibleLocalIdsInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn sys_put<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
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

    fn sys_take<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
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

    fn sys_take_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_TAKE_NON_FUNGIBLES_IDENT,
            scrypto_encode(&BucketTakeNonFungiblesInput { ids }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn sys_burn<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(self, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_function(
            RESOURCE_MANAGER_PACKAGE,
            BUCKET_BLUEPRINT,
            BUCKET_BURN_IDENT,
            scrypto_encode(&BucketBurnInput {
                bucket: Bucket(self.0),
            })
            .unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn sys_resource_address<Y, E>(&self, api: &mut Y) -> Result<ResourceAddress, E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_GET_RESOURCE_ADDRESS_IDENT,
            scrypto_encode(&BucketGetResourceAddressInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn sys_create_proof<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            BUCKET_CREATE_PROOF_IDENT,
            scrypto_encode(&BucketCreateProofInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn sys_is_empty<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<bool, E>
    where
        Y: ClientApi<E>,
    {
        Ok(self.sys_amount(api)?.is_zero())
    }
}
