use sbor::*;
use crate::core::SNodeRef;

use crate::engine::{api::*, call_engine, types::BucketId};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::{args, resource_manager};
use crate::buffer::scrypto_decode;
use crate::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use crate::rust::fmt;
use crate::rust::vec::Vec;
use crate::rust::string::ToString;
use crate::types::*;

/// Represents a transient resource container.
#[derive(Debug)]
pub struct Bucket(pub BucketId);

impl Bucket {
    /// Creates a new bucket to hold resources of the given definition.
    pub fn new(resource_address: ResourceAddress) -> Self {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(resource_address),
            function: "create_empty_bucket".to_string(),
            args: args![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Puts resources from another bucket into this bucket.
    pub fn put(&mut self, other: Self) {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::BucketRef(self.0),
            function: "put_into_bucket".to_string(),
            args: args![other],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Takes some amount of resources from this bucket.
    pub fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        let amount: Decimal = amount.into();
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::BucketRef(self.0),
            function: "take_from_bucket".to_string(),
            args: args![amount],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Takes a specific non-fungible from this bucket.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    pub fn take_non_fungible(&mut self, non_fungible_id: &NonFungibleId) -> Bucket {
        self.take_non_fungibles(&BTreeSet::from([non_fungible_id.clone()]))
    }

    /// Takes non-fungibles from this bucket.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    pub fn take_non_fungibles(&mut self, non_fungible_ids: &BTreeSet<NonFungibleId>) -> Bucket {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::BucketRef(self.0),
            function: "take_non_fungibles_from_bucket".to_string(),
            args: args![non_fungible_ids.clone()],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Burns resource within this bucket.
    pub fn burn(self) {
        resource_manager!(self.resource_address()).burn(self);
    }

    /// Creates an ownership proof of this bucket.
    pub fn create_proof(&self) -> Proof {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::BucketRef(self.0),
            function: "create_bucket_proof".to_string(),
            args: args![]
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
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
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::BucketRef(self.0),
            function: "get_bucket_amount".to_string(),
            args: args![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::BucketRef(self.0),
            function: "get_bucket_resource_address".to_string(),
            args: args![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Checks if this bucket is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Returns all the non-fungible ids contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::BucketRef(self.0),
            function: "get_non_fungible_ids_in_bucket".to_string(),
            args: args![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.resource_address();
        self.non_fungible_ids()
            .iter()
            .map(|id| NonFungible::from(NonFungibleAddress::new(resource_address, id.clone())))
            .collect()
    }
}

//========
// error
//========

/// Represents an error when decoding bucket.
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

scrypto_type!(Bucket, ScryptoType::Bucket, Vec::new());
