use crate::blueprints::resource::Proof;
use crate::math::*;
use crate::*;
use sbor::rust::prelude::*;

pub const FUNGIBLE_VAULT_BLUEPRINT: &str = "FungibleVault";

pub const FUNGIBLE_VAULT_LOCK_FEE_IDENT: &str = "lock_fee";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleVaultLockFeeInput {
    pub amount: Decimal,
    pub contingent: bool,
}

pub type FungibleVaultLockFeeOutput = ();

pub const FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT: &str = "lock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleVaultLockFungibleAmountInput {
    pub amount: Decimal,
}

pub type FungibleVaultLockFungibleAmountOutput = ();

pub const FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT: &str = "unlock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleVaultUnlockFungibleAmountInput {
    pub amount: Decimal,
}

pub type FungibleVaultUnlockFungibleAmountOutput = ();

pub const FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_IDENT: &str = "create_proof_of_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleVaultCreateProofOfAmountInput {
    pub amount: Decimal,
}

pub type FungibleVaultCreateProofOfAmountOutput = Proof;
