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
pub use resource_manager::ResourceFeature::*;
pub use resource_manager::*;
pub use resource_type::*;
pub use vault::*;
pub use worktop::*;

use crate::api::node_modules::auth::RoleDefinition;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::math::*;
use radix_engine_common::{ManifestSbor, ScryptoSbor};
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

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
        $updater_field_name:expr,
        $default_rule:expr
    ) => {
        #[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
        pub struct $roles_struct<T> {
            pub $actor_field: T,
            pub $updater_field: T,
        }

        impl $roles_struct<RoleDefinition> {
            pub fn to_role_init(self) -> $crate::blueprints::resource::RolesInit {
                let mut roles = $crate::blueprints::resource::RolesInit::new();
                roles.set_entry($actor_field_name, self.$actor_field);
                roles.set_entry($updater_field_name, self.$updater_field);
                roles
            }
        }

        impl Default for $roles_struct<RoleDefinition> {
            fn default() -> Self {
                Self {
                    $actor_field: RoleDefinition::locked($default_rule),
                    $updater_field: RoleDefinition::locked(AccessRule::DenyAll),
                }
            }
        }
    };
}

resource_roles!(
    MintRoles,
    minter,
    minter_updater,
    MINTER_ROLE,
    MINTER_UPDATER_ROLE,
    AccessRule::DenyAll
);
#[macro_export]
macro_rules! mint_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(internal_roles_struct!(MintRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    BurnRoles,
    burner,
    burner_updater,
    BURNER_ROLE,
    BURNER_UPDATER_ROLE,
    AccessRule::DenyAll
);
#[macro_export]
macro_rules! burn_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(internal_roles_struct!(BurnRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    RecallRoles,
    recaller,
    recaller_updater,
    RECALLER_ROLE,
    RECALLER_UPDATER_ROLE,
    AccessRule::DenyAll
);
#[macro_export]
macro_rules! recall_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(internal_roles_struct!(RecallRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    FreezeRoles,
    freezer,
    freezer_updater,
    FREEZER_ROLE,
    FREEZER_UPDATER_ROLE,
    AccessRule::DenyAll
);
#[macro_export]
macro_rules! freeze_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(internal_roles_struct!(FreezeRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    WithdrawRoles,
    withdrawer,
    withdrawer_updater,
    WITHDRAWER_ROLE,
    WITHDRAWER_UPDATER_ROLE,
    AccessRule::AllowAll
);
#[macro_export]
macro_rules! withdraw_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(internal_roles_struct!(WithdrawRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    DepositRoles,
    depositor,
    depositor_updater,
    DEPOSITOR_ROLE,
    DEPOSITOR_UPDATER_ROLE,
    AccessRule::AllowAll
);
#[macro_export]
macro_rules! deposit_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(internal_roles_struct!(DepositRoles, $($role => $rule;)*))
    });
}

resource_roles!(
    NonFungibleDataUpdateRoles,
    non_fungible_data_updater,
    non_fungible_data_updater_updater,
    NON_FUNGIBLE_DATA_UPDATER_ROLE,
    NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE,
    AccessRule::DenyAll
);
#[macro_export]
macro_rules! non_fungible_data_update_roles {
    {$($role:ident => $rule:expr;)*} => ({
        Some(internal_roles_struct!(NonFungibleDataUpdateRoles, $($role => $rule;)*))
    });
}
