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
        let input = GetReferenceAmountInput {
            reference: self.reference,
        };
        let output: GetReferenceAmountOutput = call_kernel(GET_REFERENCE_AMOUNT, input);

        output.amount
    }

    pub fn resource(&self) -> Address {
        let input = GetReferenceResourceInput {
            reference: self.reference,
        };
        let output: GetReferenceResourceOutput = call_kernel(GET_REFERENCE_RESOURCE, input);

        output.resource
    }

    pub fn destroy(self) {
        let input = ReturnBucketInput {
            reference: self.reference,
        };
        let _: ReturnBucketOutput = call_kernel(RETURN_BUCKET, input);
    }
}
