use crate::args;
use crate::buffer::scrypto_decode;
use crate::core::SNodeRef;
use sbor::*;

use crate::engine::{api::*, call_engine};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;
use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe)]
pub enum ResourceMethod {
    Mint,
    Burn,
    Withdraw,
    Deposit,
    UpdateMetadata,
    UpdateNonFungibleData,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe)]
pub enum Mutability {
    LOCKED,
    MUTABLE(MethodAuth),
}

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceAddress(pub [u8; 26]);

impl ResourceAddress {}

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub(crate) ResourceAddress);

impl ResourceManager {
    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&self, amount: T) -> Bucket {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "mint".to_string(),
            args: args![MintParams::Fungible {
                amount: amount.into()
            }],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn set_mintable(&self, mint_auth: MethodAuth) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "update_auth".to_string(),
            args: args![Mint, mint_auth],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn lock_mintable(&self) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "lock_auth".to_string(),
            args: args![Mint],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible<T: NonFungibleData>(&self, id: &NonFungibleId, data: T) -> Bucket {
        let mut entries = HashMap::new();
        entries.insert(id.clone(), (data.immutable_data(), data.mutable_data()));

        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "mint".to_string(),
            args: args![MintParams::NonFungible { entries }],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Burns a bucket of resources.
    pub fn burn(&self, bucket: Bucket) {
        bucket.burn()
    }

    pub fn set_burnable(&self, burn_auth: MethodAuth) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "update_auth".to_string(),
            args: args![Burn, burn_auth],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn lock_burnable(&self) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "lock_auth".to_string(),
            args: args![Burn],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Returns the resource type.
    pub fn resource_type(&self) -> ResourceType {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "get_resource_type".to_string(),
            args: args![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn set_withdrawable(&self, withdraw_auth: MethodAuth) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "update_auth".to_string(),
            args: args![Withdraw, withdraw_auth],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn lock_withdrawable(&self) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "lock_auth".to_string(),
            args: args![Withdraw],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn set_depositable(&self, deposit_auth: MethodAuth) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "update_auth".to_string(),
            args: args![Deposit, deposit_auth],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn lock_depositable(&self) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "lock_auth".to_string(),
            args: args![Deposit],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn set_updateable_metadata(&self, update_metadata_auth: MethodAuth) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "update_auth".to_string(),
            args: args![UpdateMetadata, update_metadata_auth],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn lock_updateable_metadata(&self) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "lock_auth".to_string(),
            args: args![UpdateMetadata],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn set_updateable_non_fungible_data(&self, update_metadata_auth: MethodAuth) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "update_auth".to_string(),
            args: args![UpdateNonFungibleData, update_metadata_auth],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn lock_updateable_non_fungible_data(&self) -> () {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "lock_auth".to_string(),
            args: args![UpdateNonFungibleData],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Returns the metadata associated with this resource.
    pub fn metadata(&self) -> HashMap<String, String> {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "get_metadata".to_string(),
            args: args![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Returns the current supply of this resource.
    pub fn total_supply(&self) -> Decimal {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "get_total_supply".to_string(),
            args: args![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleId) -> T {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "get_non_fungible".to_string(),
            args: args![id.clone()],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        let non_fungible: [Vec<u8>; 2] = scrypto_decode(&output.rtn).unwrap();
        T::decode(&non_fungible[0], &non_fungible[1]).unwrap()
    }

    /// Updates the mutable part of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn update_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleId, new_data: T) {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "update_non_fungible_mutable_data".to_string(),
            args: args![id.clone(), new_data.mutable_data()],
        };
        let _: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
    }

    /// Checks if non-fungible unit, with certain key exists or not.
    ///
    pub fn non_fungible_exists(&self, id: &NonFungibleId) -> bool {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "non_fungible_exists".to_string(),
            args: args![id.clone()],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Updates the resource metadata
    pub fn update_metadata(&self, new_metadata: HashMap<String, String>) {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::ResourceRef(self.0),
            function: "update_metadata".to_string(),
            args: args![new_metadata],
        };
        let _: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
    }
}

//========
// error
//========

/// Represents an error when decoding resource address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseResourceAddressError {
    InvalidHex(String),
    InvalidLength(usize),
    InvalidPrefix,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseResourceAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseResourceAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ResourceAddress {
    type Error = ParseResourceAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParseResourceAddressError::InvalidLength(slice.len())),
        }
    }
}

impl ResourceAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(ResourceAddress, ScryptoType::ResourceAddress, Vec::new());

//======
// text
//======

// Before Bech32, we use a fixed prefix for text representation.

impl FromStr for ResourceAddress {
    type Err = ParseResourceAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseResourceAddressError::InvalidHex(s.to_owned()))?;
        if bytes.get(0) != Some(&3u8) {
            return Err(ParseResourceAddressError::InvalidPrefix);
        }
        Self::try_from(&bytes[1..])
    }
}

impl fmt::Display for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(combine(3, &self.0)))
    }
}

impl fmt::Debug for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
