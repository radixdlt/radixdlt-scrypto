use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::math::Decimal;
use crate::*;
use radix_engine_common::types::*;
use sbor::rust::prelude::*;

pub const AUTH_ZONE_BLUEPRINT: &str = "AuthZone";

pub const AUTH_ZONE_POP_IDENT: &str = "pop";

pub const AUTH_ZONE_POP_EXPORT_NAME: &str = "AuthZone_pop";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZonePopInput {}

pub type AuthZonePopOutput = Proof;

pub const AUTH_ZONE_PUSH_IDENT: &str = "push";

pub const AUTH_ZONE_PUSH_EXPORT_NAME: &str = "AuthZone_push";

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

pub const AUTH_ZONE_CREATE_PROOF_EXPORT_NAME: &str = "AuthZone_create_proof";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofInput {
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofOutput = Proof;

pub const AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_IDENT: &str = "create_proof_by_amount";

pub const AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_EXPORT_NAME: &str = "AuthZone_create_proof_by_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofByAmountInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofByAmountOutput = Proof;

pub const AUTH_ZONE_CREATE_PROOF_BY_IDS_IDENT: &str = "create_proof_by_ids";

pub const AUTH_ZONE_CREATE_PROOF_BY_IDS_EXPORT_NAME: &str = "AuthZone_create_proof_by_ids";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneCreateProofByIdsInput {
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofByIdsOutput = Proof;

pub const AUTH_ZONE_CLEAR_IDENT: &str = "clear";

pub const AUTH_ZONE_CLEAR_EXPORT_NAME: &str = "AuthZone_clear";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneClearInput {}

pub type AuthZoneClearOutput = ();

pub const AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_IDENT: &str = "clear_signature_proofs";

pub const AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_EXPORT_NAME: &str = "AuthZone_clear_signature_proofs";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneClearVirtualProofsInput {}

pub type AuthZoneClearVirtualProofsOutput = ();

pub const AUTH_ZONE_DRAIN_IDENT: &str = "drain";

pub const AUTH_ZONE_DRAIN_EXPORT_NAME: &str = "AuthZone_drain";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZoneDrainInput {}

pub type AuthZoneDrainOutput = Vec<Proof>;
