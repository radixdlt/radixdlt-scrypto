use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// Represents a transient resource container.
#[derive(Debug, TypeId, Encode, Decode)]
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
    pub fn new(resource_address: Address) -> Self {
        let input = CreateEmptyBucketInput { resource_address };
        let output: CreateEmptyBucketOutput = call_kernel(CREATE_EMPTY_BUCKET, input);

        output.bucket.into()
    }

    pub fn put(&self, other: Self) {
        let input = PutIntoBucketInput {
            bucket: self.bid,
            other: other.bid,
        };
        let _: PutIntoBucketOutput = call_kernel(PUT_INTO_BUCKET, input);
    }

    pub fn take<A: Into<Amount>>(&self, amount: A) -> Self {
        let input = TakeFromBucketInput {
            bucket: self.bid,
            amount: amount.into(),
        };
        let output: TakeFromBucketOutput = call_kernel(TAKE_FROM_BUCKET, input);

        output.bucket.into()
    }

    pub fn borrow(&self) -> BucketRef {
        let input = CreateReferenceInput { bucket: self.bid };
        let output: CreateReferenceOutput = call_kernel(CREATE_REFERENCE, input);

        output.reference.into()
    }

    pub fn amount(&self) -> Amount {
        let input = GetBucketAmountInput { bucket: self.bid };
        let output: GetBucketAmountOutput = call_kernel(GET_BUCKET_AMOUNT, input);

        output.amount
    }

    pub fn resource_def(&self) -> ResourceDef {
        let input = GetBucketResourceAddressInput { bucket: self.bid };
        let output: GetBucketResourceAddressOutput =
            call_kernel(GET_BUCKET_RESOURCE_ADDRESS, input);

        output.resource_address.into()
    }

    pub fn burn(self) {
        ResourceDef::burn(self);
    }

    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }
}
