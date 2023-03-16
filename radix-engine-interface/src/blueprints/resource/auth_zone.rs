use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::math::Decimal;
use crate::*;
use sbor::rust::prelude::*;

pub const AUTH_ZONE_BLUEPRINT: &str = "AuthZone";

pub const AUTH_ZONE_POP_IDENT: &str = "pop";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZonePopInput {}

pub type AuthZonePopOutput = Proof;

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

pub type AuthZonePushOutput = ();

pub const AUTH_ZONE_CREATE_PROOF_IDENT: &str = "create_proof";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofInput {
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofOutput = Proof;

pub const AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_IDENT: &str = "create_proof_by_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofByAmountInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofByAmountOutput = Proof;

pub const AUTH_ZONE_CREATE_PROOF_BY_IDS_IDENT: &str = "create_proof_by_ids";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofByIdsInput {
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofByIdsOutput = Proof;

pub const AUTH_ZONE_CLEAR_IDENT: &str = "clear";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneClearInput {}

pub type AuthZoneClearOutput = ();

pub const AUTH_ZONE_CLEAR_VIRTUAL_PROOFS_IDENT: &str = "clear_virtual_proofs";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneClearVirtualProofsInput {}

pub type AuthZoneClearVirtualProofsOutput = ();

pub const AUTH_ZONE_DRAIN_IDENT: &str = "drain";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneDrainInput {}

pub type AuthZoneDrainOutput = Vec<Proof>;
