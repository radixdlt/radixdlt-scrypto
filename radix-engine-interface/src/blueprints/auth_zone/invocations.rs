use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::math::Decimal;
use crate::*;
use sbor::rust::collections::{BTreeMap, BTreeSet};
use sbor::rust::fmt::Debug;
use scrypto_abi::BlueprintAbi;

pub struct AuthZoneAbi;

impl AuthZoneAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        BTreeMap::new()
    }
}

pub const AUTH_ZONE_BLUEPRINT: &str = "AuthZone";

pub const AUTH_ZONE_POP_IDENT: &str = "pop";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZonePopInput {}

pub const AUTH_ZONE_PUSH_IDENT: &str = "push";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZonePushInput {
    pub proof: Proof,
}

impl Clone for AuthZonePushInput {
    fn clone(&self) -> Self {
        Self {
            proof: Proof(self.proof.0),
        }
    }
}

pub const AUTH_ZONE_CREATE_PROOF_IDENT: &str = "create_proof";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofInput {
    pub resource_address: ResourceAddress,
}

pub const AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_IDENT: &str = "create_proof_by_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofByAmountInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

pub const AUTH_ZONE_CREATE_PROOF_BY_IDS_IDENT: &str = "create_proof_by_ids";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofByIdsInput {
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

pub const AUTH_ZONE_CLEAR_IDENT: &str = "clear";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneClearInput {}

pub const AUTH_ZONE_DRAIN_IDENT: &str = "drain";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneDrainInput {}

pub const AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT: &str = "assert_access_rule";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneAssertAccessRuleInput {
    pub access_rule: AccessRule,
}
