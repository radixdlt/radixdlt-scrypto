use radix_common::constants::RESOURCE_PACKAGE;
use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::math::Decimal;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use sbor::rust::collections::IndexSet;

pub trait NativeProof {
    fn amount<Y: SystemObjectApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Decimal, E>;

    fn resource_address<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>;

    fn clone<Y: SystemObjectApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Proof, E>;

    fn drop<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E>;
}

pub trait NativeFungibleProof {}

pub trait NativeNonFungibleProof {
    fn non_fungible_local_ids<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, E>;
}

impl NativeProof for Proof {
    fn amount<Y: SystemObjectApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Decimal, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            PROOF_GET_AMOUNT_IDENT,
            scrypto_encode(&ProofGetAmountInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn resource_address<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E> {
        let address = api.get_outer_object(self.0.as_node_id())?;
        Ok(ResourceAddress::try_from(address).unwrap())
    }

    fn clone<Y: SystemObjectApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Proof, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            PROOF_CLONE_IDENT,
            scrypto_encode(&ProofCloneInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn drop<Y: SystemObjectApi<E> + SystemBlueprintApi<E>, E: SystemApiError>(
        self,
        api: &mut Y,
    ) -> Result<(), E> {
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
    fn non_fungible_local_ids<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT,
            scrypto_encode(&NonFungibleProofGetLocalIdsInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}
