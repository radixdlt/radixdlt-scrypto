use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::math::Decimal;
use crate::*;
use radix_engine_common::types::*;
use sbor::rust::collections::IndexSet;
use sbor::rust::fmt::Debug;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

pub const AUTH_ZONE_BLUEPRINT: &str = "AuthZone";

pub const AUTH_ZONE_POP_IDENT: &str = "pop";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZonePopInput {}

pub type AuthZonePopOutput = Option<Proof>;

pub const AUTH_ZONE_PUSH_IDENT: &str = "push";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZonePushInput {
    pub proof: Proof,
}

pub type AuthZonePushOutput = ();

pub const AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_IDENT: &str = "create_proof_of_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofOfAmountInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofOfAmountOutput = Proof;

pub const AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT: &str = "create_proof_of_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofOfNonFungiblesInput {
    pub ids: IndexSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofOfNonFungiblesOutput = Proof;

pub const AUTH_ZONE_CREATE_PROOF_OF_ALL_IDENT: &str = "create_proof_of_all";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofOfAllInput {
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofOfAllOutput = Proof;

pub const AUTH_ZONE_DROP_PROOFS_IDENT: &str = "drop_proofs";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneDropProofsInput {}

pub type AuthZoneDropProofsOutput = ();

pub const AUTH_ZONE_DROP_SIGNATURE_PROOFS_IDENT: &str = "drop_signature_proofs";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneDropSignatureProofsInput {}

pub type AuthZoneDropSignatureProofsOutput = ();

pub const AUTH_ZONE_DROP_REGULAR_PROOFS_IDENT: &str = "drop_regular_proofs";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneDropRegularProofsInput {}

pub type AuthZoneDropRegularProofsOutput = ();

pub const AUTH_ZONE_DRAIN_IDENT: &str = "drain";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneDrainInput {}

pub type AuthZoneDrainOutput = Vec<Proof>;

pub const AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT: &str = "assert_access_rule";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneAssertAccessRuleInput {
    pub rule: AccessRule,
}

pub type AuthZoneAssertAccessRuleOutput = ();

#[derive(Debug, Eq, PartialEq)]
pub struct AuthZoneRef(pub NodeId);
