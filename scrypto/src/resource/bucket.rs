use sbor::{describe::Type, *};

use crate::engine::*;
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::rust::fmt;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a transient resource container.
#[derive(Debug)]
pub struct Bucket(u32);

impl Bucket {
    fn this(&self) -> Self {
        Self(self.0)
    }

    /// Creates a new bucket to hold resources of the given definition.
    pub fn new(resource_def: ResourceDefRef) -> Self {
        let input = CreateEmptyBucketInput { resource_def };
        let output: CreateEmptyBucketOutput = call_engine(CREATE_EMPTY_BUCKET, input);

        output.bucket
    }

    /// Puts resources from another bucket into this bucket.
    pub fn put(&mut self, other: Self) {
        let input = PutIntoBucketInput {
            bucket: self.this(),
            other,
        };
        let _: PutIntoBucketOutput = call_engine(PUT_INTO_BUCKET, input);
    }

    /// Takes some amount of resources from this bucket.
    pub fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        let input = TakeFromBucketInput {
            bucket: self.this(),
            amount: amount.into(),
        };
        let output: TakeFromBucketOutput = call_engine(TAKE_FROM_BUCKET, input);

        output.bucket
    }

    /// Creates an immutable reference to this bucket.
    pub fn present(&self) -> BucketRef {
        let input = CreateBucketRefInput {
            bucket: self.this(),
        };
        let output: CreateBucketRefOutput = call_engine(CREATE_BUCKET_REF, input);

        output.bucket_ref
    }

    /// Returns the amount of resources in this bucket.
    pub fn amount(&self) -> Decimal {
        let input = GetBucketDecimalInput {
            bucket: self.this(),
        };
        let output: GetBucketDecimalOutput = call_engine(GET_BUCKET_AMOUNT, input);

        output.amount
    }

    /// Returns the resource definition of resources in this bucket.
    pub fn resource_def(&self) -> ResourceDefRef {
        let input = GetBucketResourceAddressInput {
            bucket: self.this(),
        };
        let output: GetBucketResourceAddressOutput =
            call_engine(GET_BUCKET_RESOURCE_ADDRESS, input);

        output.resource_def
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
            bucket: self.this(),
            key: key.clone(),
        };
        let output: TakeNonFungibleFromBucketOutput =
            call_engine(TAKE_NON_FUNGIBLE_FROM_BUCKET, input);

        output.bucket
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn get_non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let input = GetNonFungibleKeysInBucketInput {
            bucket: self.this(),
        };
        let output: GetNonFungibleKeysInBucketOutput =
            call_engine(GET_NON_FUNGIBLE_KEYS_IN_BUCKET, input);
        let resource_def = self.resource_def();
        output
            .keys
            .iter()
            .map(|id| NonFungible::from((resource_def, id.clone())))
            .collect()
    }

    /// Get all non-fungible IDs in this bucket.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn get_non_fungible_keys(&self) -> Vec<NonFungibleKey> {
        let input = GetNonFungibleKeysInBucketInput {
            bucket: self.this(),
        };
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
// error
//========

#[derive(Debug, Clone)]
pub enum ParseBucketError {
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBucketError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseBucketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Bucket {
    type Error = ParseBucketError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            4 => Ok(Self(u32::from_le_bytes(copy_u8_array(slice)))),
            _ => Err(ParseBucketError::InvalidLength(slice.len())),
        }
    }
}

impl Bucket {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

custom_type!(Bucket, CustomType::Bucket, Vec::new());
