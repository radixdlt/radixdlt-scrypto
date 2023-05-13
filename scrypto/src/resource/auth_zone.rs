use radix_engine_interface::api::{ClientAuthApi, ClientObjectApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeSet;
use scrypto::engine::scrypto_env::ScryptoEnv;

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct LocalAuthZone {}

impl LocalAuthZone {
    pub fn push<P: Into<Proof>>(proof: P) {
        let mut env = ScryptoEnv;

        let proof: Proof = proof.into();

        let node_id = env.get_auth_zone().unwrap();
        env.call_method(
            &node_id,
            AUTH_ZONE_PUSH_IDENT,
            scrypto_encode(&AuthZonePushInput { proof }).unwrap(),
        )
        .unwrap();
    }

    pub fn pop() -> Proof {
        let mut env = ScryptoEnv;
        let node_id = env.get_auth_zone().unwrap();
        let rtn = env
            .call_method(
                &node_id,
                AUTH_ZONE_POP_IDENT,
                scrypto_encode(&AuthZonePopInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn create_proof(resource_address: ResourceAddress) -> Proof {
        let mut env = ScryptoEnv;
        let node_id = env.get_auth_zone().unwrap();
        let rtn = env
            .call_method(
                &node_id,
                AUTH_ZONE_CREATE_PROOF_IDENT,
                scrypto_encode(&AuthZoneCreateProofInput { resource_address }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn create_proof_of_amount<A: Into<Decimal>>(
        amount: A,
        resource_address: ResourceAddress,
    ) -> Proof {
        let mut env = ScryptoEnv;
        let node_id = env.get_auth_zone().unwrap();
        let rtn = env
            .call_method(
                &node_id,
                AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_IDENT,
                scrypto_encode(&AuthZoneCreateProofOfAmountInput {
                    resource_address,
                    amount: amount.into(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn create_proof_of_non_fungibles(
        ids: BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        let mut env = ScryptoEnv;
        let node_id = env.get_auth_zone().unwrap();
        let rtn = env
            .call_method(
                &node_id,
                AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
                scrypto_encode(&AuthZoneCreateProofOfNonFungiblesInput {
                    resource_address,
                    ids,
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn create_proof_of_all(resource_address: ResourceAddress) -> Proof {
        let mut env = ScryptoEnv;
        let node_id = env.get_auth_zone().unwrap();
        let rtn = env
            .call_method(
                &node_id,
                AUTH_ZONE_CREATE_PROOF_OF_ALL_IDENT,
                scrypto_encode(&AuthZoneCreateProofOfAllInput { resource_address }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn clear() {
        let mut env = ScryptoEnv;
        let node_id = env.get_auth_zone().unwrap();
        let rtn = env
            .call_method(
                &node_id,
                AUTH_ZONE_CLEAR_IDENT,
                scrypto_encode(&AuthZoneClearInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }
}
