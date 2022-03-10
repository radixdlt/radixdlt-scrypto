use sbor::{describe::Type, *};

use crate::engine::{api::*, call_engine, types::ProofId};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
#[cfg(not(feature = "alloc"))]
use crate::rust::fmt;
use crate::rust::vec::Vec;
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
    /// Checks if the referenced bucket contains the given resource, and aborts if not so.
    pub fn check(&self, resource_def_id: ResourceDefId) {
        if !self.contains(resource_def_id) {
            panic!("Proof check failed");
        }
    }

    pub fn check_non_fungible_id(&self, non_fungible_id: &NonFungibleId) {
        self.check(non_fungible_id.resource_def_id());
        if !self
            .get_non_fungible_keys()
            .iter()
            .any(|k| k.eq(&non_fungible_id.key()))
        {
            panic!("Proof check failed");
        }
    }

    /// Checks if the referenced bucket contains the given resource.
    pub fn contains(&self, resource_def_id: ResourceDefId) -> bool {
        self.amount() > 0.into() && self.resource_def_id() == resource_def_id
    }

    /// Returns the resource amount within the bucket.
    pub fn amount(&self) -> Decimal {
        let input = GetProofAmountInput { proof_id: self.0 };
        let output: GetProofAmountOutput = call_engine(GET_PROOF_AMOUNT, input);

        output.amount
    }

    /// Returns the resource definition of resources within the bucket.
    pub fn resource_def_id(&self) -> ResourceDefId {
        let input = GetProofResourceDefIdInput { proof_id: self.0 };
        let output: GetProofResourceDefIdOutput = call_engine(GET_PROOF_RESOURCE_DEF_ID, input);

        output.resource_def_id
    }

    /// Returns the key of a singleton non-fungible.
    ///
    /// # Panic
    /// If the bucket is empty or contains more than one non-fungibles.
    pub fn get_non_fungible_key(&self) -> NonFungibleKey {
        let keys = self.get_non_fungible_keys();
        assert!(
            keys.len() == 1,
            "1 non-fungible expected, but {} found",
            keys.len()
        );
        keys[0].clone()
    }

    /// Returns the keys of all non-fungibles in this bucket.
    ///
    /// # Panics
    /// If the bucket is not a non-fungible bucket.
    pub fn get_non_fungible_keys(&self) -> Vec<NonFungibleKey> {
        let input = GetNonFungibleKeysInProofInput { proof_id: self.0 };
        let output: GetNonFungibleKeysInProofOutput =
            call_engine(GET_NON_FUNGIBLE_KEYS_IN_PROOF, input);

        output.keys
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

custom_type!(Proof, CustomType::Proof, Vec::new());
