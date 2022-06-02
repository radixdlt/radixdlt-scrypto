use sbor::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::buffer::scrypto_decode;
use crate::core::SNodeRef;
use crate::engine::{api::*, call_engine, types::ProofId};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::sfunctions;
use crate::types::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ConsumingProofMethod {
    Drop(),
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ProofMethod {
    Amount(),
    ResourceAddress(),
    NonFungibleIds(),
    Clone(),
}

/// Represents a proof of owning some resource.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Proof(pub ProofId);

impl Clone for Proof {
    sfunctions! {
        SNodeRef::ProofRef(self.0) => {
            fn clone(&self) -> Self { ProofMethod::Clone() }
        }
    }
}

impl Proof {
    sfunctions! {
        SNodeRef::ProofRef(self.0) => {
            pub fn amount(&self) -> Decimal { ProofMethod::Amount() }
            pub fn resource_address(&self) -> ResourceAddress { ProofMethod::ResourceAddress() }
            pub fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId> { ProofMethod::NonFungibleIds() }
        }
    }

    sfunctions! {
        SNodeRef::Proof(self.0) => {
            pub fn drop(self) -> () { ConsumingProofMethod::Drop() }
        }
    }

    /// Whether this proof includes an ownership proof of any of the given resource.
    pub fn contains(&self, resource_address: ResourceAddress) -> bool {
        self.resource_address() == resource_address
    }

    /// Whether this proof includes an ownership proof of at least the given amount of resource.
    pub fn contains_resource(&self, amount: Decimal, resource_address: ResourceAddress) -> bool {
        self.resource_address() == resource_address && self.amount() > amount
    }

    /// Whether this proof includes an ownership proof of the given non-fungible.
    pub fn contains_non_fungible(&self, non_fungible_address: &NonFungibleAddress) -> bool {
        if self.resource_address() != non_fungible_address.resource_address() {
            return false;
        }

        self.non_fungible_ids()
            .iter()
            .any(|k| k.eq(&non_fungible_address.non_fungible_id()))
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible proof.
    pub fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.resource_address();
        self.non_fungible_ids()
            .iter()
            .map(|id| NonFungible::from(NonFungibleAddress::new(resource_address, id.clone())))
            .collect()
    }

    /// Returns a singleton non-fungible.
    ///
    /// # Panics
    /// Panics if this is not a singleton proof
    pub fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        let non_fungibles = self.non_fungibles();
        if non_fungibles.len() != 1 {
            panic!("Expecting singleton NFT proof");
        }
        non_fungibles.into_iter().next().unwrap()
    }

    /// Checks if the referenced bucket is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }
}

//========
// error
//========

/// Represents an error when decoding proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseProofError {
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseProofError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseProofError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Proof {
    type Error = ParseProofError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            4 => Ok(Self(u32::from_le_bytes(copy_u8_array(slice)))),
            _ => Err(ParseProofError::InvalidLength(slice.len())),
        }
    }
}

impl Proof {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

scrypto_type!(Proof, ScryptoType::Proof, Vec::new());
