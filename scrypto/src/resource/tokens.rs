use crate::kernel::*;
use crate::types::*;

use sbor::{Decode, Describe, Encode};

/// A bucket that holds token resource.
#[derive(Debug, Describe, Encode, Decode)]
pub struct Tokens {
    bid: BID,
}

impl From<BID> for Tokens {
    fn from(bid: BID) -> Self {
        Self { bid }
    }
}

impl Into<BID> for Tokens {
    fn into(self) -> BID {
        self.bid
    }
}

impl Tokens {
    pub fn new(amount: U256, resource: Address) -> Self {
        let input = MintTokensInput { amount, resource };
        let output: MintTokensOutput = call_kernel(MINT_TOKENS, input);

        output.tokens.into()
    }

    pub fn put(&mut self, other: Self) {
        let input = CombineTokensInput {
            tokens: self.bid,
            other: other.bid,
        };
        let _: CombineTokensOutput = call_kernel(COMBINE_TOKENS, input);
    }

    pub fn take(&mut self, amount: U256) -> Self {
        let input = SplitTokensInput {
            tokens: self.bid,
            amount,
        };
        let output: SplitTokensOutput = call_kernel(SPLIT_TOKENS, input);

        output.tokens.into()
    }

    pub fn amount(&self) -> U256 {
        let input = GetTokensAmountInput { tokens: self.bid };
        let output: GetTokensAmountOutput = call_kernel(GET_TOKENS_AMOUNT, input);

        output.amount
    }

    pub fn resource(&self) -> Address {
        let input = GetTokensResourceInput { tokens: self.bid };
        let output: GetTokensResourceOutput = call_kernel(GET_TOKENS_RESOURCE, input);

        output.resource
    }
}
