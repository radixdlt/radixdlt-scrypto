use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode};
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use scrypto::engine::scrypto_env::ScryptoEnv;

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    pub fn push<P: Into<Proof>>(proof: P) {
        let mut env = ScryptoEnv;

        let proof: Proof = proof.into();

        env.call_method(
            RENodeId::AuthZoneStack,
            AUTH_ZONE_PUSH_IDENT,
            scrypto_encode(&AuthZonePushInput { proof }).unwrap(),
        )
        .unwrap();
    }

    pub fn pop() -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                RENodeId::AuthZoneStack,
                AUTH_ZONE_POP_IDENT,
                scrypto_encode(&AuthZonePopInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn create_proof(resource_address: ResourceAddress) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                RENodeId::AuthZoneStack,
                AUTH_ZONE_CREATE_PROOF_IDENT,
                scrypto_encode(&AuthZoneCreateProofInput { resource_address }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn create_proof_by_amount(amount: Decimal, resource_address: ResourceAddress) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                RENodeId::AuthZoneStack,
                AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_IDENT,
                scrypto_encode(&AuthZoneCreateProofByAmountInput {
                    resource_address,
                    amount,
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn create_proof_by_ids(
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                RENodeId::AuthZoneStack,
                AUTH_ZONE_CREATE_PROOF_BY_IDS_IDENT,
                scrypto_encode(&AuthZoneCreateProofByIdsInput {
                    resource_address,
                    ids: ids.clone(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn assert_access_rule(access_rule: AccessRule) {
        let mut env = ScryptoEnv;
        env.call_method(
            RENodeId::AuthZoneStack,
            AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT,
            scrypto_encode(&AuthZoneAssertAccessRuleInput { access_rule }).unwrap(),
        )
        .unwrap();
    }
}
