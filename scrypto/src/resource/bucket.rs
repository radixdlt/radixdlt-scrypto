use sbor::{describe::Type, *};

use crate::constants::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// Represents resources of some quantity.
#[derive(Debug, Encode, Decode)]
pub struct Bucket {
    bid: BID,
}

impl Describe for Bucket {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BUCKET.to_owned(),
        }
    }
}

impl From<BID> for Bucket {
    fn from(bid: BID) -> Self {
        Self { bid }
    }
}

impl From<Bucket> for BID {
    fn from(a: Bucket) -> BID {
        a.bid
    }
}

impl Bucket {
    pub fn new(resource: Address) -> Self {
        let input = NewEmptyBucketInput { resource };
        let output: NewEmptyBucketOutput = call_kernel(NEW_EMPTY_BUCKET, input);

        output.bucket.into()
    }

    pub fn put(&self, other: Self) {
        let input = CombineBucketsInput {
            bucket: self.bid,
            other: other.bid,
        };
        let _: CombineBucketsOutput = call_kernel(COMBINE_BUCKETS, input);
    }

    pub fn take<A: Into<U256>>(&self, amount: A) -> Self {
        let input = SplitBucketInput {
            bucket: self.bid,
            amount: amount.into(),
        };
        let output: SplitBucketOutput = call_kernel(SPLIT_BUCKET, input);

        output.bucket.into()
    }

    pub fn borrow(&self) -> BucketRef {
        let input = BorrowImmutableInput { bucket: self.bid };
        let output: BorrowImmutableOutput = call_kernel(BORROW_IMMUTABLE, input);

        output.reference.into()
    }

    pub fn amount(&self) -> U256 {
        let input = GetAmountInput { bucket: self.bid };
        let output: GetAmountOutput = call_kernel(GET_AMOUNT, input);

        output.amount
    }

    pub fn resource(&self) -> Address {
        let input = GetResourceInput { bucket: self.bid };
        let output: GetResourceOutput = call_kernel(GET_RESOURCE, input);

        output.resource
    }
}
