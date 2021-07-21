use crate::constructs::*;
use crate::types::*;

/// A utility structure for creating a basic badge.
pub struct BasicBadge {}

impl BasicBadge {
    pub fn create(symbol: &str, amount: U256) -> Badges {
        let resource = Resource::new(symbol, "", "", "", "", None, Some(amount));

        Badges::new(amount, &resource)
    }
}
