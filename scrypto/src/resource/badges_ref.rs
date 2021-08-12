use sbor::{Decode, Describe, Encode};

use crate::kernel::*;
use crate::types::*;

/// A borrowed rid to a `Badges` bucket.
#[derive(Debug, Describe, Encode, Decode)]
pub struct BadgesRef {
    rid: RID,
}

impl From<RID> for BadgesRef {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl BadgesRef {
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
        let input = ReturnReferenceInput {
            reference: self.rid,
        };
        let _: ReturnReferenceOutput = call_kernel(RETURN_REFERENCE, input);
    }
}
