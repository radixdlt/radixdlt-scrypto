use radix_engine_interface::api::{ClientApi, ClientBlueprintApi, ClientObjectApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::RESOURCE_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoCategorize, ScryptoDecode,
};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use sbor::rust::collections::BTreeSet;
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
        Y: ClientApi<E>;
}

pub trait NativeFungibleProof {}

pub trait NativeNonFungibleProof {
    fn non_fungible_local_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, E>
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
        let rtn = api.call_method(
            self.0.as_node_id(),
            PROOF_GET_RESOURCE_ADDRESS_IDENT,
            scrypto_encode(&ProofGetResourceAddressInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
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
        let info = api.get_object_info(self.0.as_node_id())?;
        let blueprint = info.blueprint_id.blueprint_name;
        api.call_function(
            RESOURCE_PACKAGE,
            blueprint.as_str(),
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
    ) -> Result<BTreeSet<NonFungibleLocalId>, E>
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
