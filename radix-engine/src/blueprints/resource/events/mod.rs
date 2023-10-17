pub mod fungible_vault;
pub mod non_fungible_vault;
mod resource_manager;

pub use fungible_vault::{
    FungibleVaultDepositEvent, FungibleVaultLockFeeEvent, FungibleVaultPayFeeEvent,
    FungibleVaultRecallEvent, FungibleVaultWithdrawEvent,
};
pub use non_fungible_vault::{
    NonFungibleVaultDepositEvent, NonFungibleVaultRecallEvent, NonFungibleVaultWithdrawEvent,
};
pub use resource_manager::*;
