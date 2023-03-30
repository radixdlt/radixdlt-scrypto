use crate::math::*;
use crate::*;
use sbor::rust::prelude::*;
use sbor::*;

pub const FUNGIBLE_VAULT_BLUEPRINT: &str = "FungibleVault";

pub const FUNGIBLE_VAULT_LOCK_FEE_IDENT: &str = "lock_fee";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleVaultLockFeeInput {
    pub amount: Decimal,
    pub contingent: bool,
}

pub type FungibleVaultLockFeeOutput = ();