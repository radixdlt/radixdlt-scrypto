use crate::internal_prelude::*;
use radix_common::data::scrypto::model::*;
use sbor::rust::collections::IndexSet;
use sbor::rust::fmt::Debug;
use sbor::*;

pub const NON_FUNGIBLE_PROOF_BLUEPRINT: &str = "NonFungibleProof";

pub const NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT: &str = "NonFungibleProof_get_local_ids";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct NonFungibleProofGetLocalIdsInput {}

pub type NonFungibleProofGetLocalIdsManifestInput = NonFungibleProofGetLocalIdsInput;

pub type NonFungibleProofGetLocalIdsOutput = IndexSet<NonFungibleLocalId>;

pub type NonFungibleProofDropInput = ProofDropInput;
pub type NonFungibleProofDropManifestInput = NonFungibleProofDropInput;

pub type NonFungibleProofCloneInput = ProofCloneInput;
pub type NonFungibleProofCloneManifestInput = NonFungibleProofCloneInput;

pub type NonFungibleProofGetAmountInput = ProofGetAmountInput;
pub type NonFungibleProofGetAmountManifestInput = NonFungibleProofGetAmountInput;

pub type NonFungibleProofGetResourceAddressInput = ProofGetResourceAddressInput;
pub type NonFungibleProofGetResourceAddressManifestInput = NonFungibleProofGetResourceAddressInput;
