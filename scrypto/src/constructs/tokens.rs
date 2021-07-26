use crate::constructs::*;
use crate::kernel::*;
use crate::types::*;

use serde::{Deserialize, Serialize};

/// A bucket that holds token resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct Tokens {
    rid: RID,
}

impl From<RID> for Tokens {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl Into<RID> for Tokens {
    fn into(self) -> RID {
        self.rid.clone()
    }
}

impl Tokens {
    pub fn new(amount: U256, resource: &Resource) -> Self {
        let input = MintTokensInput {
            amount,
            resource: resource.address(),
        };
        let output: MintTokensOutput = syscall(MINT_TOKENS, input);

        output.tokens.into()
    }

    pub fn put(&mut self, other: Self) {
        let input = CombineTokensInput {
            tokens: self.rid.clone(),
            other: other.rid.clone(),
        };
        let _: CombineTokensOutput = syscall(COMBINE_TOKENS, input);
    }

    pub fn take(&mut self, amount: U256) -> Self {
        let input = SplitTokensInput {
            tokens: self.rid.clone(),
            amount,
        };
        let output: SplitTokensOutput = syscall(SPLIT_TOKENS, input);

        output.tokens.into()
    }

    pub fn amount(&self) -> U256 {
        let input = GetTokensAmountInput {
            tokens: self.rid.clone(),
        };
        let output: GetTokensAmountOutput = syscall(GET_TOKENS_AMOUNT, input);

        output.amount
    }

    pub fn resource(&self) -> Resource {
        let input = GetTokensResourceInput {
            tokens: self.rid.clone(),
        };
        let output: GetTokensResourceOutput = syscall(GET_TOKENS_RESOURCE, input);

        output.resource.into()
    }

    pub fn address(&self) -> Address {
        self.resource().address()
    }
}
