use crate::blueprints::resource::Proof;
use crate::internal_prelude::*;
use radix_common::math::*;
use sbor::rust::prelude::*;

pub const FUNGIBLE_VAULT_BLUEPRINT: &str = "FungibleVault";

pub const FUNGIBLE_VAULT_LOCK_FEE_IDENT: &str = "lock_fee";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleVaultLockFeeInput {
    pub amount: Decimal,
    pub contingent: bool,
}

pub type FungibleVaultLockFeeManifestInput = FungibleVaultLockFeeInput;

pub type FungibleVaultLockFeeOutput = ();

pub const FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT: &str = "lock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleVaultLockFungibleAmountInput {
    pub amount: Decimal,
}

pub type FungibleVaultLockFungibleAmountManifestInput = FungibleVaultLockFungibleAmountInput;

pub type FungibleVaultLockFungibleAmountOutput = ();

pub const FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT: &str = "unlock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleVaultUnlockFungibleAmountInput {
    pub amount: Decimal,
}

pub type FungibleVaultUnlockFungibleAmountManifestInput = FungibleVaultUnlockFungibleAmountInput;

pub type FungibleVaultUnlockFungibleAmountOutput = ();

pub const FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_IDENT: &str = "create_proof_of_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleVaultCreateProofOfAmountInput {
    pub amount: Decimal,
}

pub type FungibleVaultCreateProofOfAmountManifestInput = FungibleVaultCreateProofOfAmountInput;

pub type FungibleVaultCreateProofOfAmountOutput = Proof;

pub type FungibleVaultPutInput = VaultPutInput;
pub type FungibleVaultPutManifestInput = VaultPutManifestInput;

pub type FungibleVaultFreezeInput = VaultFreezeInput;
pub type FungibleVaultFreezeManifestInput = VaultFreezeManifestInput;

pub type FungibleVaultUnfreezeInput = VaultUnfreezeInput;
pub type FungibleVaultUnfreezeManifestInput = VaultUnfreezeManifestInput;

pub type FungibleVaultGetAmountInput = VaultGetAmountInput;
pub type FungibleVaultGetAmountManifestInput = VaultGetAmountManifestInput;

pub type FungibleVaultBurnInput = VaultBurnInput;
pub type FungibleVaultBurnManifestInput = VaultBurnManifestInput;
