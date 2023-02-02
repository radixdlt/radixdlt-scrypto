use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::Invokable;
use radix_engine_interface::blueprints::resource::*;
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

        env.invoke(AuthZonePushInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            proof,
        })
        .unwrap();
    }

    pub fn pop() -> Proof {
        let mut env = ScryptoEnv;
        env.invoke(AuthZonePopInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
        })
        .unwrap()
    }

    pub fn create_proof(resource_address: ResourceAddress) -> Proof {
        let mut env = ScryptoEnv;
        env.invoke(AuthZoneCreateProofInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            resource_address,
        })
        .unwrap()
    }

    pub fn create_proof_by_amount(amount: Decimal, resource_address: ResourceAddress) -> Proof {
        let mut env = ScryptoEnv;
        env.invoke(AuthZoneCreateProofByAmountInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            amount,
            resource_address,
        })
        .unwrap()
    }

    pub fn create_proof_by_ids(
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        let mut env = ScryptoEnv;
        env.invoke(AuthZoneCreateProofByIdsInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            ids: ids.clone(),
            resource_address,
        })
        .unwrap()
    }

    pub fn assert_access_rule(access_rule: AccessRule) {
        let mut env = ScryptoEnv;
        env.invoke(AuthZoneAssertAccessRuleInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            access_rule,
        })
        .unwrap()
    }
}
