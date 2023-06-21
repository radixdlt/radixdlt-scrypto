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

pub fn check_fungible_amount(amount: &Decimal, divisibility: u8) -> bool {
    !amount.is_negative()
        && amount.0 % BnumI256::from(10i128.pow((18 - divisibility).into())) == BnumI256::from(0)
}

pub fn check_non_fungible_amount(amount: &Decimal) -> bool {
    !amount.is_negative() && amount.0 % BnumI256::from(10i128.pow(18)) == BnumI256::from(0)
}
