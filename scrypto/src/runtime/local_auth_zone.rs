use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeSet;
use scrypto::engine::scrypto_env::ScryptoVmV1Api;

use crate::resource::ScryptoAuthZone;

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct LocalAuthZone {}

impl LocalAuthZone {
    pub fn push<P: Into<Proof>>(proof: P) {
        let proof: Proof = proof.into();
        let node_id = ScryptoVmV1Api.actor_get_auth_zone();
        AuthZoneRef(node_id).push(proof)
    }

    pub fn pop() -> Proof {
        let node_id = ScryptoVmV1Api.actor_get_auth_zone();
        AuthZoneRef(node_id).pop()
    }

    pub fn create_proof_of_amount<A: Into<Decimal>>(
        amount: A,
        resource_address: ResourceAddress,
    ) -> Proof {
        let node_id = ScryptoVmV1Api.actor_get_auth_zone();
        AuthZoneRef(node_id).create_proof_of_amount(amount, resource_address)
    }

    pub fn create_proof_of_non_fungibles(
        ids: BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        let node_id = ScryptoVmV1Api.actor_get_auth_zone();
        AuthZoneRef(node_id).create_proof_of_non_fungibles(ids, resource_address)
    }

    pub fn create_proof_of_all(resource_address: ResourceAddress) -> Proof {
        let node_id = ScryptoVmV1Api.actor_get_auth_zone();
        AuthZoneRef(node_id).create_proof_of_all(resource_address)
    }

    pub fn drop_proofs() {
        let node_id = ScryptoVmV1Api.actor_get_auth_zone();
        AuthZoneRef(node_id).drop_proofs()
    }

    pub fn drop_signature_proofs() {
        let node_id = ScryptoVmV1Api.actor_get_auth_zone();
        AuthZoneRef(node_id).drop_signature_proofs()
    }

    pub fn drop_regular_proofs() {
        let node_id = ScryptoVmV1Api.actor_get_auth_zone();
        AuthZoneRef(node_id).drop_regular_proofs()
    }
}
