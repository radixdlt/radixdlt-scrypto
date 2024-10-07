use crate::internal_prelude::*;

pub const AUTH_ZONE_BLUEPRINT: &str = "AuthZone";

pub const AUTH_ZONE_POP_IDENT: &str = "pop";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AuthZonePopInput {}

pub type AuthZonePopManifestInput = AuthZonePopInput;

pub type AuthZonePopOutput = Option<Proof>;

pub const AUTH_ZONE_PUSH_IDENT: &str = "push";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct AuthZonePushInput {
    pub proof: Proof,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct AuthZonePushManifestInput {
    pub proof: ManifestProof,
}

pub type AuthZonePushOutput = ();

pub const AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_IDENT: &str = "create_proof_of_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AuthZoneCreateProofOfAmountInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofOfAmountManifestInput = AuthZoneCreateProofOfAmountInput;

pub type AuthZoneCreateProofOfAmountOutput = Proof;

pub const AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT: &str = "create_proof_of_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AuthZoneCreateProofOfNonFungiblesInput {
    pub ids: IndexSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofOfNonFungiblesManifestInput = AuthZoneCreateProofOfNonFungiblesInput;

pub type AuthZoneCreateProofOfNonFungiblesOutput = Proof;

pub const AUTH_ZONE_CREATE_PROOF_OF_ALL_IDENT: &str = "create_proof_of_all";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AuthZoneCreateProofOfAllInput {
    pub resource_address: ResourceAddress,
}

pub type AuthZoneCreateProofOfAllManifestInput = AuthZoneCreateProofOfAllInput;

pub type AuthZoneCreateProofOfAllOutput = Proof;

pub const AUTH_ZONE_DROP_PROOFS_IDENT: &str = "drop_proofs";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AuthZoneDropProofsInput {}

pub type AuthZoneDropProofsManifestInput = AuthZoneDropProofsInput;

pub type AuthZoneDropProofsOutput = ();

pub const AUTH_ZONE_DROP_SIGNATURE_PROOFS_IDENT: &str = "drop_signature_proofs";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AuthZoneDropSignatureProofsInput {}

pub type AuthZoneDropSignatureProofsManifestInput = AuthZoneDropSignatureProofsInput;

pub type AuthZoneDropSignatureProofsOutput = ();

pub const AUTH_ZONE_DROP_REGULAR_PROOFS_IDENT: &str = "drop_regular_proofs";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AuthZoneDropRegularProofsInput {}

pub type AuthZoneDropRegularProofsManifestInput = AuthZoneDropRegularProofsInput;

pub type AuthZoneDropRegularProofsOutput = ();

pub const AUTH_ZONE_DRAIN_IDENT: &str = "drain";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AuthZoneDrainInput {}

pub type AuthZoneDrainManifestInput = AuthZoneDrainInput;

pub type AuthZoneDrainOutput = Vec<Proof>;

pub const AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT: &str = "assert_access_rule";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct AuthZoneAssertAccessRuleInput {
    pub rule: AccessRule,
}

pub type AuthZoneAssertAccessRuleManifestInput = AuthZoneAssertAccessRuleInput;

pub type AuthZoneAssertAccessRuleOutput = ();

#[derive(Debug, Eq, PartialEq)]
pub struct AuthZoneRef(pub NodeId);
