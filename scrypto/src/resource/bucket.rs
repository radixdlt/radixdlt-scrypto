use crate::kernel::*;
use crate::resource::*;
use crate::types::*;

/// Represents a basket of resources.
pub trait Bucket<T: BucketRef> {
    fn new_empty(resource: Address) -> Self;

    fn put(&self, other: Self);

    fn take(&self, amount: U256) -> Self;

    fn amount(&self) -> U256;

    fn resource(&self) -> Address;

    fn borrow(&self) -> T;
}

impl Bucket<RID> for BID {
    fn new_empty(resource: Address) -> Self {
        let input = NewEmptyBucketInput { resource };
        let output: NewEmptyBucketOutput = call_kernel(NEW_EMPTY_BUCKET, input);

        output.bucket
    }

    fn put(&self, other: Self) {
        let input = CombineBucketsInput {
            bucket: *self,
            other: other,
        };
        let _: CombineBucketsOutput = call_kernel(COMBINE_BUCKETS, input);
    }

    fn take(&self, amount: U256) -> Self {
        let input = SplitBucketInput {
            bucket: *self,
            amount,
        };
        let output: SplitBucketOutput = call_kernel(SPLIT_BUCKET, input);

        output.bucket
    }

    fn borrow(&self) -> RID {
        let input = BorrowImmutableInput { bucket: *self };
        let output: BorrowImmutableOutput = call_kernel(BORROW_IMMUTABLE, input);

        output.reference
    }

    fn amount(&self) -> U256 {
        let input = GetAmountInput { bucket: *self };
        let output: GetAmountOutput = call_kernel(GET_AMOUNT, input);

        output.amount
    }

    fn resource(&self) -> Address {
        let input = GetResourceInput { bucket: *self };
        let output: GetResourceOutput = call_kernel(GET_RESOURCE, input);

        output.resource
    }
}
