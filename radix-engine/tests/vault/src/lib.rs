use scrypto::prelude::*;

pub mod non_existent_vault;
pub mod vault;

package_init!(non_existent_vault::blueprint::NonExistentVault::describe(), vault::blueprint::VaultTest::describe());
