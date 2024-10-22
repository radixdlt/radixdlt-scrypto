mod auth_zone;
mod bucket;
mod fungible;
mod non_fungible;
mod proof;
mod proof_rule;
mod resource;
mod resource_manager;
mod resource_type;
mod role_assignment;
mod vault;
mod worktop;

pub use auth_zone::*;
pub use bucket::*;
pub use fungible::*;
pub use non_fungible::*;
pub use proof::*;
pub use proof_rule::*;
pub use resource::*;
pub use resource_manager::ResourceFeature::*;
pub use resource_manager::*;
pub use resource_type::*;
pub use role_assignment::*;
use sbor::Sbor;
pub use vault::*;
pub use worktop::*;

#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use radix_common::math::*;
use radix_common::{ManifestSbor, ScryptoSbor};
use sbor::rust::prelude::*;

pub fn check_fungible_amount(amount: &Decimal, divisibility: u8) -> bool {
    !amount.is_negative()
        && amount.attos() % I192::from(10i128.pow((18 - divisibility).into())) == I192::from(0)
}

pub fn check_non_fungible_amount(amount: &Decimal) -> Result<u32, ()> {
    // Integers between [0..u32::MAX]
    u32::try_from(amount).map_err(|_| ())
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
        #[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
        pub struct $roles_struct<T> {
            pub $actor_field: T,
            pub $updater_field: T,
        }

        impl $roles_struct<$crate::object_modules::role_assignment::RoleDefinition> {
            pub fn to_role_init(self) -> $crate::blueprints::resource::RoleAssignmentInit {
                let mut roles = $crate::blueprints::resource::RoleAssignmentInit::new();
                roles.define_role($actor_field_name, self.$actor_field);
                roles.define_role($updater_field_name, self.$updater_field);
                roles
            }
        }

        impl Default for $roles_struct<$crate::object_modules::role_assignment::RoleDefinition> {
            fn default() -> Self {
                Self {
                    $actor_field: Some($default_rule),
                    $updater_field: Some(AccessRule::DenyAll),
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
        Some($crate::internal_roles_struct!(MintRoles, $($role => $rule;)*))
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
        Some($crate::internal_roles_struct!(BurnRoles, $($role => $rule;)*))
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
        Some($crate::internal_roles_struct!(RecallRoles, $($role => $rule;)*))
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
        Some($crate::internal_roles_struct!(FreezeRoles, $($role => $rule;)*))
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
        Some($crate::internal_roles_struct!(WithdrawRoles, $($role => $rule;)*))
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
        Some($crate::internal_roles_struct!(DepositRoles, $($role => $rule;)*))
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
        Some($crate::internal_roles_struct!(NonFungibleDataUpdateRoles, $($role => $rule;)*))
    });
}

/// Define the withdraw strategy when request amount does not match underlying
/// resource divisibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub enum WithdrawStrategy {
    Exact,
    Rounded(RoundingMode),
}

pub trait ForWithdrawal {
    fn for_withdrawal(
        &self,
        divisibility: u8,
        withdraw_strategy: WithdrawStrategy,
    ) -> Option<Decimal>;
}

impl ForWithdrawal for Decimal {
    fn for_withdrawal(
        &self,
        divisibility: u8,
        withdraw_strategy: WithdrawStrategy,
    ) -> Option<Decimal> {
        match withdraw_strategy {
            WithdrawStrategy::Exact => Some(self.clone()),
            WithdrawStrategy::Rounded(mode) => self.checked_round(divisibility, mode),
        }
    }
}

impl Default for WithdrawStrategy {
    fn default() -> Self {
        Self::Exact
    }
}
