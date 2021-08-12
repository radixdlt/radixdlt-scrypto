use sbor::{Decode, Describe, Encode};

use crate::kernel::*;
use crate::types::*;

/// A borrowed reference to a `Badges` bucket.
#[derive(Debug, Describe, Encode, Decode)]
pub struct BadgesRef {
    reference: Reference,
}

impl From<Reference> for BadgesRef {
    fn from(reference: Reference) -> Self {
        Self { reference }
    }
}

impl BadgesRef {
    pub fn amount(&self) -> U256 {
        let input = GetAmountRefInput {
            reference: self.reference,
        };
        let output: GetAmountRefOutput = call_kernel(GET_AMOUNT_REF, input);

        output.amount
    }

    pub fn resource(&self) -> Address {
        let input = GetResourceRefInput {
            reference: self.reference,
        };
        let output: GetResourceRefOutput = call_kernel(GET_RESOURCE_REF, input);

        output.resource
    }

    pub fn destroy(self) {
        let input = ReturnReferenceInput {
            reference: self.reference,
        };
        let _: ReturnReferenceOutput = call_kernel(RETURN_REFERENCE, input);
    }
}
