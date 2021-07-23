use serde::{Deserialize, Serialize};

use crate::constructs::*;
use crate::types::*;

/// An abstraction of token minting logic.
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenMint {
    out_of_circulation: Tokens,
    maximum_supply: U256,
}

impl TokenMint {
    pub fn create(reserve_supply: Tokens) -> Self {
        let maximum_supply = reserve_supply.amount();
        Self {
            out_of_circulation: reserve_supply,
            maximum_supply,
        }
    }

    pub fn get_token_address(&self) -> Address {
        self.out_of_circulation.address()
    }

    pub fn mint(&mut self, amount: U256) -> Tokens {
        self.out_of_circulation.take(amount)
    }

    pub fn burn(&mut self, tokens: Tokens) {
        self.out_of_circulation.put(tokens);
    }

    pub fn maximum_supply(&self) -> U256 {
        self.maximum_supply
    }

    pub fn total_circulating_supply(&self) -> U256 {
        self.maximum_supply - self.out_of_circulation.amount()
    }
}
