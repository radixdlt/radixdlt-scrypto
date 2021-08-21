use sbor::model::*;
use sbor::{Decode, Describe, Encode};

use crate::kernel::*;
use crate::types::rust::borrow::ToOwned;
use crate::types::*;

/// A borrowed rid to a `Tokens` bucket.
#[derive(Debug, Encode, Decode)]
pub struct TokensRef {
    rid: RID,
}

impl Describe for TokensRef {
    fn describe() -> Type {
        Type::SystemType {
            name: "::scrypto::resource::TokensRef".to_owned(),
        }
    }
}

impl From<RID> for TokensRef {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl TokensRef {
    pub fn amount(&self) -> U256 {
        let input = GetAmountRefInput {
            reference: self.rid,
        };
        let output: GetAmountRefOutput = call_kernel(GET_AMOUNT_REF, input);

        output.amount
    }

    pub fn resource(&self) -> Address {
        let input = GetResourceRefInput {
            reference: self.rid,
        };
        let output: GetResourceRefOutput = call_kernel(GET_RESOURCE_REF, input);

        output.resource
    }

    pub fn destroy(self) {
        let input = DropReferenceInput {
            reference: self.rid,
        };
        let _: DropReferenceOutput = call_kernel(DROP_REFERENCE, input);
    }
}
