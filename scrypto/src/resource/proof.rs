use sbor::*;
use crate::args;
use crate::buffer::scrypto_decode;
use crate::core::SNodeRef;

use crate::engine::{api::*, call_engine, types::ProofId};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use crate::rust::fmt;
use crate::rust::vec::Vec;
use crate::rust::string::ToString;
use crate::types::*;

/// Represents a proof of owning some resource.
#[derive(Debug)]
pub struct Proof(pub ProofId);

impl Clone for Proof {
    fn clone(&self) -> Self {
        let input = CloneProofInput { proof_id: self.0 };
        let output: CloneProofOutput = call_engine(CLONE_PROOF, input);

        Self(output.proof_id)
    }
}

impl Proof {
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

    /// Returns the resource amount within the bucket.
    pub fn amount(&self) -> Decimal {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ProofRef(self.0),
            function: "get_total_amount".to_string(),
            args: args![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Returns the resource address
    pub fn resource_address(&self) -> ResourceAddress {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ProofRef(self.0),
            function: "get_resource_address".to_string(),
            args: args![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Returns the ids of all non-fungibles in this bucket.
    ///
    /// # Panics
    /// If the bucket is not a non-fungible bucket.
    pub fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ProofRef(self.0),
            function: "get_non_fungible_ids".to_string(),
            args: args![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Destroys this proof.
    pub fn drop(self) {
        let input = DropProofInput { proof_id: self.0 };
        let _: DropProofOutput = call_engine(DROP_PROOF, input);
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
