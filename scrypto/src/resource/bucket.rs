use sbor::*;

use crate::engine::{api::*, call_engine, types::BucketId};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::resource_manager;
use crate::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use crate::rust::fmt;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a transient resource container.
#[derive(Debug)]
pub struct Bucket(pub BucketId);

impl Bucket {
    /// Creates a new bucket to hold resources of the given definition.
    pub fn new(resource_address: ResourceAddress) -> Self {
        let input = CreateEmptyBucketInput {
            resource_address: resource_address,
        };
        let output: CreateEmptyBucketOutput = call_engine(CREATE_EMPTY_BUCKET, input);

        Self(output.bucket_id)
    }

    /// Puts resources from another bucket into this bucket.
    pub fn put(&mut self, other: Self) {
        let input = PutIntoBucketInput {
            bucket_id: self.0,
            other: other.0,
        };
        let _: PutIntoBucketOutput = call_engine(PUT_INTO_BUCKET, input);
    }

    /// Takes some amount of resources from this bucket.
    pub fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        let input = TakeFromBucketInput {
            bucket_id: self.0,
            amount: amount.into(),
        };
        let output: TakeFromBucketOutput = call_engine(TAKE_FROM_BUCKET, input);

        Self(output.bucket_id)
    }

    /// Creates an ownership proof of this bucket.
    pub fn create_proof(&self) -> Proof {
        let input = CreateBucketProofInput { bucket_id: self.0 };
        let output: CreateBucketProofOutput = call_engine(CREATE_BUCKET_PROOF, input);

        Proof(output.proof_id)
    }

    /// Uses resources in this bucket as authorization for an operation.
    pub fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        AuthZone::push(self.create_proof());
        let output = f();
        AuthZone::pop().drop();
        output
    }

    /// Returns the amount of resources in this bucket.
    pub fn amount(&self) -> Decimal {
        let input = GetBucketAmountInput { bucket_id: self.0 };
        let output: GetBucketAmountOutput = call_engine(GET_BUCKET_AMOUNT, input);

        output.amount
    }

    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        let input = GetBucketResourceAddressInput { bucket_id: self.0 };
        let output: GetBucketResourceAddressOutput =
            call_engine(GET_BUCKET_RESOURCE_ADDRESS, input);

        output.resource_address
    }

    /// Burns resource within this bucket.
    pub fn burn(self) {
        resource_manager!(self.resource_address()).burn(self);
    }

    /// Checks if this bucket is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Takes a non-fungible from this bucket, by id.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    pub fn take_non_fungible(&mut self, non_fungible_id: &NonFungibleId) -> Bucket {
        let input = TakeNonFungibleFromBucketInput {
            bucket_id: self.0,
            non_fungible_id: non_fungible_id.clone(),
        };
        let output: TakeNonFungibleFromBucketOutput =
            call_engine(TAKE_NON_FUNGIBLE_FROM_BUCKET, input);

        Self(output.bucket_id)
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn get_non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let input = GetNonFungibleIdsInBucketInput { bucket_id: self.0 };
        let output: GetNonFungibleIdsInBucketOutput =
            call_engine(GET_NON_FUNGIBLE_IDS_IN_BUCKET, input);
        let resource_address = self.resource_address();
        output
            .non_fungible_ids
            .iter()
            .map(|id| NonFungible::from(NonFungibleAddress::new(resource_address, id.clone())))
            .collect()
    }

    /// Returns the address of  a singleton non-fungible.
    ///
    /// # Panic
    /// If this bucket is empty or contains more than one non-fungibles.
    pub fn get_non_fungible_id(&self) -> NonFungibleId {
        let non_fungible_ids = self.get_non_fungible_ids();
        assert!(
            non_fungible_ids.len() == 1,
            "1 non-fungible expected, but {} found",
            non_fungible_ids.len()
        );
        non_fungible_ids.into_iter().next().unwrap()
    }

    /// Returns the ids of all non-fungibles in this bucket.
    ///
    /// # Panics
    /// If this bucket is not a non-fungible bucket.
    pub fn get_non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
        let input = GetNonFungibleIdsInBucketInput { bucket_id: self.0 };
        let output: GetNonFungibleIdsInBucketOutput =
            call_engine(GET_NON_FUNGIBLE_IDS_IN_BUCKET, input);

        output.non_fungible_ids
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, non_fungible_id: &NonFungibleId) -> T {
        resource_manager!(self.resource_address()).get_non_fungible_data(non_fungible_id)
    }

    /// Updates the mutable part of the data of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    pub fn update_non_fungible_data<T: NonFungibleData>(
        &mut self,
        non_fungible_id: &NonFungibleId,
        new_data: T,
    ) {
        resource_manager!(self.resource_address())
            .update_non_fungible_data(non_fungible_id, new_data)
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
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
