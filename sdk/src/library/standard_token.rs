use crate::constructs::*;
use crate::types::*;

/// A utility structure for creating a standard token.
pub struct StandardToken {}

impl StandardToken {
    pub fn create(symbol: &str, name: &str, url: &str, amount: U256) -> Tokens {
        let resource = Resource::new(symbol, name, "", url, "", None, Some(amount));

        Tokens::new(amount, &resource)
    }
}
