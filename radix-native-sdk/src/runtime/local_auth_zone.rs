use crate::resource::NativeAuthZone;
use radix_common::data::scrypto::model::*;
use radix_common::math::Decimal;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use sbor::rust::prelude::*;

pub struct LocalAuthZone {}

impl LocalAuthZone {
    pub fn drain<Y: SystemApi<E>, E: SystemApiError>(api: &mut Y) -> Result<Vec<Proof>, E> {
        let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
        AuthZoneRef(auth_zone).drain(api)
    }

    pub fn drop_proofs<Y: SystemApi<E>, E: SystemApiError>(api: &mut Y) -> Result<(), E> {
        let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
        AuthZoneRef(auth_zone).drop_proofs(api)
    }

    pub fn drop_regular_proofs<Y: SystemApi<E>, E: SystemApiError>(api: &mut Y) -> Result<(), E> {
        let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
        AuthZoneRef(auth_zone).drop_regular_proofs(api)
    }

    pub fn drop_signature_proofs<Y: SystemApi<E>, E: SystemApiError>(api: &mut Y) -> Result<(), E> {
        let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
        AuthZoneRef(auth_zone).drop_signature_proofs(api)
    }

    pub fn pop<Y: SystemApi<E>, E: SystemApiError>(api: &mut Y) -> Result<Option<Proof>, E> {
        let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
        AuthZoneRef(auth_zone).pop(api)
    }

    pub fn create_proof_of_amount<Y: SystemApi<E>, E: SystemApiError>(
        amount: Decimal,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E> {
        let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
        AuthZoneRef(auth_zone).create_proof_of_amount(amount, resource_address, api)
    }

    pub fn create_proof_of_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        ids: &IndexSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E> {
        let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
        AuthZoneRef(auth_zone).create_proof_of_non_fungibles(ids, resource_address, api)
    }

    pub fn create_proof_of_all<Y: SystemApi<E>, E: SystemApiError>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E> {
        let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
        AuthZoneRef(auth_zone).create_proof_of_all(resource_address, api)
    }

    pub fn push<Y: SystemApi<E>, E: SystemApiError, P: Into<Proof>>(
        proof: P,
        api: &mut Y,
    ) -> Result<(), E> {
        let proof: Proof = proof.into();

        let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
        AuthZoneRef(auth_zone).push(proof, api)
    }
}
