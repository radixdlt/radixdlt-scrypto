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

use crate::api::node_modules::auth::RoleDefinition;
use radix_engine_common::math::*;

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
    ) => {
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
    };
}

resource_roles!(
    MintableRoles,
    minter,
    minter_updater,
    MINTER_ROLE,
    MINTER_UPDATER_ROLE
);
#[macro_export]
macro_rules! mintable {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        internal_roles!(MintableRoles, $($role => $rule, $locked;)*)
    });
}

resource_roles!(
    BurnableRoles,
    burner,
    burner_updater,
    BURNER_ROLE,
    BURNER_UPDATER_ROLE
);
#[macro_export]
macro_rules! burnable {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        internal_roles!(BurnableRoles, $($role => $rule, $locked;)*)
    });
}

resource_roles!(
    RecallableRoles,
    recaller,
    recaller_updater,
    RECALLER_ROLE,
    RECALLER_UPDATER_ROLE
);
#[macro_export]
macro_rules! recallable {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        internal_roles!(RecallableRoles, $($role => $rule, $locked;)*)
    });
}

resource_roles!(
    FreezableRoles,
    freezer,
    freezer_updater,
    FREEZER_ROLE,
    FREEZER_UPDATER_ROLE
);
#[macro_export]
macro_rules! freezable {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        internal_roles!(FreezableRoles, $($role => $rule, $locked;)*)
    });
}

resource_roles!(
    WithdrawableRoles,
    withdrawer,
    withdrawer_updater,
    WITHDRAWER_ROLE,
    WITHDRAWER_UPDATER_ROLE
);
#[macro_export]
macro_rules! restrict_withdraw {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        internal_roles!(WithdrawableRoles, $($role => $rule, $locked;)*)
    });
}

resource_roles!(
    DepositableRoles,
    depositor,
    depositor_updater,
    DEPOSITOR_ROLE,
    DEPOSITOR_UPDATER_ROLE
);
#[macro_export]
macro_rules! restrict_deposit {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        internal_roles!(DepositableRoles, $($role => $rule, $locked;)*)
    });
}

resource_roles!(
    UpdatableNonFungibleDataRoles,
    non_fungible_data_updater,
    non_fungible_data_updater_updater,
    NON_FUNGIBLE_DATA_UPDATER_ROLE,
    NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE
);
#[macro_export]
macro_rules! updatable_non_fungible_data {
    {$($role:ident => $rule:expr, $locked:ident;)*} => ({
        internal_roles!(UpdatableNonFungibleDataRoles, $($role => $rule, $locked;)*)
    });
}
