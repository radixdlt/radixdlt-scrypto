use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::engine::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::rust::vec::Vec;
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
        let input = CreateEmptyBucketInput {
            resource_address: resource_def.into().address(),
        };
        let output: CreateEmptyBucketOutput = call_engine(CREATE_EMPTY_BUCKET, input);

        output.bid.into()
    }

    /// Puts resources from another bucket into this bucket.
    pub fn put(&mut self, other: Self) {
        let input = PutIntoBucketInput {
            bid: self.bid,
            other: other.bid,
        };
        let _: PutIntoBucketOutput = call_engine(PUT_INTO_BUCKET, input);
    }

    /// Takes some amount of resources from this bucket.
    pub fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        let input = TakeFromBucketInput {
            bid: self.bid,
            amount: amount.into(),
        };
        let output: TakeFromBucketOutput = call_engine(TAKE_FROM_BUCKET, input);

        output.bid.into()
    }

    /// Creates an immutable reference to this bucket.
    pub fn present(&self) -> BucketRef {
        let input = CreateBucketRefInput { bid: self.bid };
        let output: CreateBucketRefOutput = call_engine(CREATE_BUCKET_REF, input);

        output.rid.into()
    }

    /// Returns the amount of resources in this bucket.
    pub fn amount(&self) -> Decimal {
        let input = GetBucketDecimalInput { bid: self.bid };
        let output: GetBucketDecimalOutput = call_engine(GET_BUCKET_AMOUNT, input);

        output.amount
    }

    /// Returns the resource definition of resources in this bucket.
    pub fn resource_def(&self) -> ResourceDef {
        let input = GetBucketResourceAddressInput { bid: self.bid };
        let output: GetBucketResourceAddressOutput =
            call_engine(GET_BUCKET_RESOURCE_ADDRESS, input);

        output.resource_address.into()
    }

    /// Returns the resource definition address.
    pub fn resource_address(&self) -> Address {
        self.resource_def().address()
    }

    /// Burns resource within this bucket.
    pub fn burn(self) {
        self.resource_def().burn(self);
    }

    /// Burns resource within this bucket.
    pub fn burn_with_auth(self, auth: BucketRef) {
        self.resource_def().burn_with_auth(self, auth);
    }

    /// Checks if this bucket is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Uses resources in this bucket as authorization for an operation.
    pub fn authorize<F: FnOnce(BucketRef) -> O, O>(&self, f: F) -> O {
        f(self.present())
    }

    /// Takes a non-fungible from this bucket, by id.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    pub fn take_non_fungible(&mut self, key: &NonFungibleKey) -> Bucket {
        let input = TakeNonFungibleFromBucketInput {
            bid: self.bid,
            key: key.clone(),
        };
        let output: TakeNonFungibleFromBucketOutput =
            call_engine(TAKE_NON_FUNGIBLE_FROM_BUCKET, input);

        output.bid.into()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn get_non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let input = GetNonFungibleKeysInBucketInput { bid: self.bid };
        let output: GetNonFungibleKeysInBucketOutput =
            call_engine(GET_NON_FUNGIBLE_KEYS_IN_BUCKET, input);
        let resource_address = self.resource_address();
        output
            .keys
            .iter()
            .map(|id| NonFungible::from((resource_address, id.clone())))
            .collect()
    }

    /// Get all non-fungible IDs in this bucket.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn get_non_fungible_keys(&self) -> Vec<NonFungibleKey> {
        let input = GetNonFungibleKeysInBucketInput { bid: self.bid };
        let output: GetNonFungibleKeysInBucketOutput =
            call_engine(GET_NON_FUNGIBLE_KEYS_IN_BUCKET, input);

        output.keys
    }

    /// Get the non-fungible id and panic if not singleton.
    pub fn get_non_fungible_key(&self) -> NonFungibleKey {
        let keys = self.get_non_fungible_keys();
        assert!(
            keys.len() == 1,
            "Expect 1 non-fungible, but found {}",
            keys.len()
        );
        keys[0].clone()
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, key: &NonFungibleKey) -> T {
        self.resource_def().get_non_fungible_data(key)
    }

    /// Updates the mutable part of the data of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    pub fn update_non_fungible_data<T: NonFungibleData>(
        &mut self,
        key: &NonFungibleKey,
        new_data: T,
        auth: BucketRef,
    ) {
        self.resource_def()
            .update_non_fungible_data(key, new_data, auth)
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
