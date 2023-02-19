use crate::blueprints::resource::*;
use crate::math::*;
use crate::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

pub const VAULT_BLUEPRINT: &str = "Vault";

pub const VAULT_PUT_IDENT: &str = "put";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct VaultPutInput {
    pub bucket: Bucket,
}

impl Clone for VaultPutInput {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

pub const VAULT_TAKE_IDENT: &str = "take";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultTakeInput {
    pub amount: Decimal,
}

pub const VAULT_TAKE_NON_FUNGIBLES_IDENT: &str = "take_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultTakeNonFungiblesInput {
    pub non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
}

pub const VAULT_LOCK_FEE_IDENT: &str = "lock_fee";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultLockFeeInput {
    pub amount: Decimal,
    pub contingent: bool,
}

pub const VAULT_RECALL_IDENT: &str = "recall";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultRecallInput {
    pub amount: Decimal,
}

pub const VAULT_RECALL_NON_FUNGIBLES_IDENT: &str = "recall_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultRecallNonFungiblesInput {
    pub non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
}

pub const VAULT_GET_AMOUNT_IDENT: &str = "get_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultGetAmountInput {}

pub const VAULT_GET_RESOURCE_ADDRESS_IDENT: &str = "get_resource_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultGetResourceAddressInput {}

pub const VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT: &str = "get_non_fungible_local_ids";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultGetNonFungibleLocalIdsInput {}

pub const VAULT_CREATE_PROOF_IDENT: &str = "create_proof";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultCreateProofInput {}

pub const VAULT_CREATE_PROOF_BY_AMOUNT_IDENT: &str = "create_proof_by_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultCreateProofByAmountInput {
    pub amount: Decimal,
}

pub const VAULT_CREATE_PROOF_BY_IDS_IDENT: &str = "create_proof_by_ids";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VaultCreateProofByIdsInput {
    pub ids: BTreeSet<NonFungibleLocalId>,
}
