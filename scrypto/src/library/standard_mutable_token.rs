use crate::constructs::*;
use crate::library::*;
use crate::types::*;

/// A utility structure for creating a standard mutable token.
pub struct StandardMutableToken {}

impl StandardMutableToken {
    pub fn create(
        symbol: &str,
        name: &str,
        url: &str,
        initial_supply: U256,
        maximum_supply: U256,
    ) -> (TokenMint, Tokens) {
        // Create the maximum quantity of tokens that will ever be permitted to exist, with the metadata defined by StandardToken
        let new_tokens = StandardToken::create(name, symbol, url, maximum_supply);

        // Instantiate our TokenMint component, passing it the complete supply of the new token
        let mut token_mint = TokenMint::create(new_tokens);

        // "Mint" our desired initial supply
        let initial_tokens = token_mint.mint(initial_supply);

        // Return our Stateless TokenMint, as well as the initial supply of tokens
        return (token_mint, initial_tokens);
    }
}
