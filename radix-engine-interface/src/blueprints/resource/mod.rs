mod access_rules;
mod auth_zone;
mod bucket;
mod fungible;
mod non_fungible;
mod non_fungible_global_id;
mod proof;
mod proof_rule;
mod resource;
mod resource_manager;
mod resource_type;
mod vault;
mod worktop;

pub use access_rules::*;
pub use auth_zone::*;
pub use bucket::*;
pub use fungible::*;
pub use non_fungible::*;
pub use non_fungible_global_id::*;
pub use proof::*;
pub use proof_rule::*;
pub use resource::*;
pub use resource_manager::ResourceAction::*;
pub use resource_manager::*;
pub use resource_type::*;
pub use vault::*;
pub use worktop::*;

use radix_engine_common::math::*;
use crate::api::node_modules::auth::RoleDefinition;

pub fn check_fungible_amount(amount: &Decimal, divisibility: u8) -> bool {
    !amount.is_negative()
        && amount.0 % BnumI256::from(10i128.pow((18 - divisibility).into())) == BnumI256::from(0)
}

pub fn check_non_fungible_amount(amount: &Decimal) -> bool {
    !amount.is_negative() && amount.0 % BnumI256::from(10i128.pow(18)) == BnumI256::from(0)
}


pub struct MintableRoles<T> {
    pub minter: T,
    pub minter_updater: T,
}

impl<T> MintableRoles<T> {
    pub fn list(self) -> Vec<(&'static str, T)> {
        vec![
            (MINTER_ROLE, self.minter),
            (MINTER_UPDATER_ROLE, self.minter_updater),
        ]
    }
}

impl MintableRoles<(Option<AccessRule>, bool)> {
    pub fn to_role_init(self) -> ResourceActionRoleInit {
        ResourceActionRoleInit {
            actor: RoleDefinition {
                value: self.minter.0,
                lock: self.minter.1,
            },
            updater: RoleDefinition {
                value: self.minter_updater.0,
                lock: self.minter_updater.1,
            },
        }
    }
}


#[macro_export]
macro_rules! mintable {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        let mintable_roles = internal_roles_struct!(MintableRoles, $($role => $rule, $locked;)*);
        mintable_roles.to_role_init()
    });
}


pub struct BurnableRoles<T> {
    pub burner: T,
    pub burner_updater: T,
}

impl<T> BurnableRoles<T> {
    pub fn list(self) -> Vec<(&'static str, T)> {
        vec![
            (BURNER_ROLE, self.burner),
            (BURNER_UPDATER_ROLE, self.burner_updater),
        ]
    }
}

impl BurnableRoles<(Option<AccessRule>, bool)> {
    pub fn to_role_init(self) -> ResourceActionRoleInit {
        ResourceActionRoleInit {
            actor: RoleDefinition {
                value: self.burner.0,
                lock: self.burner.1,
            },
            updater: RoleDefinition {
                value: self.burner_updater.0,
                lock: self.burner_updater.1,
            },
        }
    }
}


#[macro_export]
macro_rules! burnable {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        let burnable_roles = internal_roles_struct!(BurnableRoles, $($role => $rule, $locked;)*);
        burnable_roles.to_role_init()
    });
}



