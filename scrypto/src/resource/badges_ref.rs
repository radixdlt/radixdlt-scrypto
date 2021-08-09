use sbor::{Decode, Describe, Encode};

use crate::kernel::*;
use crate::types::*;

/// A borrowed reference to a `Badges` bucket.
#[derive(Debug, Describe, Encode, Decode)]
pub struct BadgesRef {
    bid: BID,
}

impl From<BID> for BadgesRef {
    fn from(bid: BID) -> Self {
        Self { bid }
    }
}

impl BadgesRef {
    pub fn amount(&self) -> U256 {
        let input = GetBucketAmountInput { bucket: self.bid };
        let output: GetBucketAmountOutput = call_kernel(GET_BUCKET_AMOUNT, input);

        output.amount
    }

    pub fn resource(&self) -> Address {
        let input = GetBucketResourceInput { bucket: self.bid };
        let output: GetBucketResourceOutput = call_kernel(GET_BUCKET_RESOURCE, input);

        output.resource
    }

    pub fn destroy(self) {
        let input = ReturnBucketInput {
            reference: self.bid,
        };
        let _: ReturnBucketOutput = call_kernel(RETURN_BUCKET, input);
    }
}
