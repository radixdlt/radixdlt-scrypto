use crate::internal_prelude::*;

pub const FUNGIBLE_PROOF_BLUEPRINT: &str = "FungibleProof";

pub type FungibleProofDropInput = ProofDropInput;
pub type FungibleProofDropManifestInput = FungibleProofDropInput;

pub type FungibleProofCloneInput = ProofCloneInput;
pub type FungibleProofCloneManifestInput = FungibleProofCloneInput;

pub type FungibleProofGetAmountInput = ProofGetAmountInput;
pub type FungibleProofGetAmountManifestInput = FungibleProofGetAmountInput;

pub type FungibleProofGetResourceAddressInput = ProofGetResourceAddressInput;
pub type FungibleProofGetResourceAddressManifestInput = FungibleProofGetResourceAddressInput;
