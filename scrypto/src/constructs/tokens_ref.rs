use sbor::{Decode, Encode};

use crate::constructs::*;
use crate::kernel::*;
use crate::types::*;

/// A borrowed reference to a `Tokens` bucket.
#[derive(Debug, Encode, Decode)]
pub struct TokensRef {
    rid: RID,
}

impl From<RID> for TokensRef {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl TokensRef {
    pub fn amount(&self) -> U256 {
        let input = GetTokensAmountInput { tokens: self.rid };
        let output: GetTokensAmountOutput = call_kernel(GET_TOKENS_AMOUNT, input);

        output.amount
    }

    pub fn resource(&self) -> Resource {
        let input = GetTokensResourceInput { tokens: self.rid };
        let output: GetTokensResourceOutput = call_kernel(GET_TOKENS_RESOURCE, input);

        output.resource.into()
    }

    pub fn destroy(self) {
        let input = ReturnTokensInput {
            reference: self.rid,
        };
        let _: ReturnTokensOutput = call_kernel(RETURN_TOKENS, input);
    }
}
