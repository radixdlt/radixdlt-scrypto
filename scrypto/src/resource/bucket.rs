use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::types::*;

/// Represents a transient resource container.
#[derive(Debug)]
pub struct Bucket {
    bid: Bid,
}

impl From<Bid> for Bucket {
    fn from(bid: Bid) -> Self {
        Self { bid }
    }
}

impl From<Bucket> for Bid {
    fn from(a: Bucket) -> Bid {
        a.bid
    }
}

impl Bucket {
    /// Creates a new bucket to hold resources of the given definition.
    pub fn new<A: Into<ResourceDef>>(resource_def: A) -> Self {
        let resource_def: ResourceDef = resource_def.into();
        let input = CreateEmptyBucketInput {
            resource_def: resource_def.into(),
        };
        let output: CreateEmptyBucketOutput = call_kernel(CREATE_EMPTY_BUCKET, input);

        output.bucket.into()
    }

    /// Puts resources from another bucket into this bucket.
    pub fn put(&self, other: Self) {
        let input = PutIntoBucketInput {
            bucket: self.bid,
            other: other.bid,
        };
        let _: PutIntoBucketOutput = call_kernel(PUT_INTO_BUCKET, input);
    }

    /// Takes some amount of resources from this bucket.
    pub fn take<A: Into<Decimal>>(&self, amount: A) -> Self {
        let input = TakeFromBucketInput {
            bucket: self.bid,
            amount: amount.into(),
        };
        let output: TakeFromBucketOutput = call_kernel(TAKE_FROM_BUCKET, input);

        output.bucket.into()
    }

    /// Creates an immutable reference to this bucket.
    pub fn borrow(&self) -> BucketRef {
        let input = CreateBucketRefInput { bucket: self.bid };
        let output: CreateBucketRefOutput = call_kernel(CREATE_BUCKET_REF, input);

        output.bucket_ref.into()
    }

    /// Returns the amount of resources in this bucket.
    pub fn amount(&self) -> Decimal {
        let input = GetBucketDecimalInput { bucket: self.bid };
        let output: GetBucketDecimalOutput = call_kernel(GET_BUCKET_AMOUNT, input);

        output.amount
    }

    /// Returns the resource definition of resources in this bucket.
    pub fn resource_def(&self) -> ResourceDef {
        let input = GetBucketResourceAddressInput { bucket: self.bid };
        let output: GetBucketResourceAddressOutput = call_kernel(GET_BUCKET_RESOURCE_DEF, input);

        output.resource_def.into()
    }

    /// Returns the resource definition address.
    pub fn resource_address(&self) -> Address {
        self.resource_def().address()
    }

    /// Burns resource within this bucket.
    pub fn burn(self, minter: BucketRef) {
        self.resource_def().burn(self, minter);
    }

    /// Checks if this bucket is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

     /// Use resources in this bucket as authorization for an operation.
     pub fn authorize<F: FnOnce(BucketRef) -> O, O>(&self, f: F) -> O {
        f(self.borrow())
    }
}

//========
// SBOR
//========

impl TypeId for Bucket {
    fn type_id() -> u8 {
        Bid::type_id()
    }
}

impl Encode for Bucket {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.bid.encode_value(encoder);
    }
}

impl Decode for Bucket {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Bid::decode_value(decoder).map(Into::into)
    }
}

impl Describe for Bucket {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BUCKET.to_owned(),
            generics: vec![],
        }
    }
}
