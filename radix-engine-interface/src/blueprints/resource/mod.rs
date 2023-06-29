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


#[macro_export]
macro_rules! resource_roles {
    (
        $roles_struct:ident,
        $actor_field:ident,
        $updater_field:ident,
        $actor_field_name:expr,
        $updater_field_name:expr
    ) => (
        pub struct $roles_struct<T> {
            pub $actor_field: T,
            pub $updater_field: T,
        }

        impl<T> $roles_struct<T> {
            pub fn list(self) -> Vec<(&'static str, T)> {
                vec![
                    ($actor_field_name, self.$actor_field),
                    ($updater_field_name, self.$updater_field),
                ]
            }
        }

        impl $roles_struct<(Option<AccessRule>, bool)> {
            pub fn to_role_init(self) -> ResourceActionRoleInit {
                ResourceActionRoleInit {
                    actor: RoleDefinition {
                        value: self.$actor_field.0,
                        lock: self.$actor_field.1,
                    },
                    updater: RoleDefinition {
                        value: self.$updater_field.0,
                        lock: self.$updater_field.1,
                    },
                }
            }
        }
    );
}

resource_roles!(MintableRoles, minter, minter_updater, MINTER_ROLE, MINTER_UPDATER_ROLE);
#[macro_export]
macro_rules! mintable {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        let mintable_roles = internal_roles_struct!(MintableRoles, $($role => $rule, $locked;)*);
        mintable_roles.to_role_init()
    });
}

resource_roles!(BurnableRoles, burner, burner_updater, BURNER_ROLE, BURNER_UPDATER_ROLE);
#[macro_export]
macro_rules! burnable {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        let burnable_roles = internal_roles_struct!(BurnableRoles, $($role => $rule, $locked;)*);
        burnable_roles.to_role_init()
    });
}

resource_roles!(RecallableRoles, recaller, recaller_updater, RECALLER_ROLE, RECALLER_UPDATER_ROLE);
#[macro_export]
macro_rules! recallable {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        let recallable_roles = internal_roles_struct!(RecallableRoles, $($role => $rule, $locked;)*);
        recallable_roles.to_role_init()
    });
}

resource_roles!(FreezableRoles, freezer, freezer_updater, FREEZER_ROLE, FREEZER_UPDATER_ROLE);
#[macro_export]
macro_rules! freezable {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        let freezable_roles = internal_roles_struct!(FreezableRoles, $($role => $rule, $locked;)*);
        freezable_roles.to_role_init()
    });
}

resource_roles!(WithdrawableRoles, withdrawer, withdrawer_updater, WITHDRAWER_ROLE, WITHDRAWER_UPDATER_ROLE);
#[macro_export]
macro_rules! restrict_withdraw {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        let withdrawable_roles = internal_roles_struct!(WithdrawableRoles, $($role => $rule, $locked;)*);
        withdrawable_roles.to_role_init()
    });
}

resource_roles!(DepositableRoles, depositor, depositor_updater, DEPOSITOR_ROLE, DEPOSITOR_UPDATER_ROLE);
#[macro_export]
macro_rules! restrict_deposit {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        let depositable_roles = internal_roles_struct!(DepositableRoles, $($role => $rule, $locked;)*);
        depositable_roles.to_role_init()
    });
}
