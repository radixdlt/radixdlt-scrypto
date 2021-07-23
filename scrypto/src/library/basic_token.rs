use crate::constructs::*;
use crate::types::*;

/// A utility structure for creating a basic token.
pub struct BasicToken {}

impl BasicToken {
    pub fn create(symbol: &str, amount: U256) -> Tokens {
        let resource = Resource::new(symbol, "", "", "", "", None, Some(amount));

        Tokens::new(amount, &resource)
    }
}
