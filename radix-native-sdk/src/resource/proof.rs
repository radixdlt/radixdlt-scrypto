use radix_common::constants::RESOURCE_PACKAGE;
use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::math::Decimal;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use sbor::rust::collections::IndexSet;

use super::ResourceManager;

// TODO: Move the fungible/non-fungible parts out of NativeProof,
//       and require the user opt in with `as_fungible` / `as_non_fungible` like in Scrypto.
//       This will be a breaking change, so likely need some communication.

pub trait NativeProof {
    type ResourceManager;

    fn amount<Y: SystemObjectApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Decimal, E>;

    fn resource_address<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>;

    fn resource_manager<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<Self::ResourceManager, E>;

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
    type ResourceManager = ResourceManager;

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

    fn resource_manager<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceManager, E> {
        Ok(ResourceManager(self.resource_address(api)?))
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

pub trait SpecializedProof: AsRef<Proof> + Into<Proof> {
    type ResourceManager: From<ResourceManager>;

    /// Purposefully not From because we want to only use this when
    /// we are confident it's the correct type
    fn from_proof_of_correct_type(proof: Proof) -> Self;
}

impl SpecializedProof for FungibleProof {
    // Change when we have a native FungibleResourceManager
    type ResourceManager = ResourceManager;

    fn from_proof_of_correct_type(proof: Proof) -> Self {
        Self(proof)
    }
}

impl SpecializedProof for NonFungibleProof {
    // Change when we have a native NonFungibleResourceManager
    type ResourceManager = ResourceManager;

    fn from_proof_of_correct_type(proof: Proof) -> Self {
        Self(proof)
    }
}

impl<T: SpecializedProof> NativeProof for T {
    type ResourceManager = <Self as SpecializedProof>::ResourceManager;

    fn amount<Y: SystemObjectApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Decimal, E> {
        self.as_ref().amount(api)
    }

    fn resource_address<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E> {
        self.as_ref().resource_address(api)
    }

    fn resource_manager<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<Self::ResourceManager, E> {
        Ok(ResourceManager(self.resource_address(api)?).into())
    }

    fn clone<Y: SystemObjectApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Proof, E> {
        self.as_ref().clone(api)
    }

    fn drop<Y: SystemApi<E>, E: SystemApiError>(self, api: &mut Y) -> Result<(), E> {
        self.into().drop(api)
    }
}

impl NativeFungibleProof for Proof {}

impl NativeFungibleProof for FungibleProof {}

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

impl NativeNonFungibleProof for NonFungibleProof {
    fn non_fungible_local_ids<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, E> {
        self.as_ref().non_fungible_local_ids(api)
    }
}
