use radix_common::constants::RESOURCE_PACKAGE;
use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoCategorize, ScryptoDecode,
};
use radix_common::math::Decimal;
use radix_engine_interface::api::{SystemApi, ClientBlueprintApi, ClientObjectApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use sbor::rust::collections::IndexSet;
use sbor::rust::fmt::Debug;

pub trait NativeProof {
    fn amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: ClientObjectApi<E>;

    fn resource_address<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>
    where
        Y: ClientObjectApi<E>;

    fn clone<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientObjectApi<E>;

    fn drop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(self, api: &mut Y) -> Result<(), E>
    where
        Y: SystemApi<E>;
}

pub trait NativeFungibleProof {}

pub trait NativeNonFungibleProof {
    fn non_fungible_local_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, E>
    where
        Y: ClientObjectApi<E>;
}

impl NativeProof for Proof {
    fn amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            PROOF_GET_AMOUNT_IDENT,
            scrypto_encode(&ProofGetAmountInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn resource_address<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>
    where
        Y: ClientObjectApi<E>,
    {
        let address = api.get_outer_object(self.0.as_node_id())?;
        Ok(ResourceAddress::try_from(address).unwrap())
    }

    fn clone<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            PROOF_CLONE_IDENT,
            scrypto_encode(&ProofCloneInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn drop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(self, api: &mut Y) -> Result<(), E>
    where
        Y: ClientObjectApi<E> + ClientBlueprintApi<E>,
    {
        let blueprint_id = api.get_blueprint_id(self.0.as_node_id())?;
        api.call_function(
            RESOURCE_PACKAGE,
            blueprint_id.blueprint_name.as_str(),
            PROOF_DROP_IDENT,
            scrypto_encode(&ProofDropInput {
                proof: Proof(self.0),
            })
            .unwrap(),
        )?;
        Ok(())
    }
}

impl NativeFungibleProof for Proof {}

impl NativeNonFungibleProof for Proof {
    fn non_fungible_local_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT,
            scrypto_encode(&NonFungibleProofGetLocalIdsInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}
