use sbor::rust::borrow::ToOwned;
use sbor::rust::collections::HashMap;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::buffer::{scrypto_decode, scrypto_encode};
use crate::core::SNodeRef;
use crate::engine::{api::*, call_engine};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::{sfunctions, sfunctions2};
use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe)]
pub enum ResourceMethodAuthKey {
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
    MUTABLE(AccessRule),
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ResourceManagerFunction {
    Create(
        ResourceType,
        HashMap<String, String>,
        HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
        Option<MintParams>,
    ),
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerUpdateAuthInput {
    pub method: ResourceMethodAuthKey,
    pub access_rule: AccessRule,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerLockAuthInput {
    pub method: ResourceMethodAuthKey,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerCreateVaultInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerCreateBucketInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerMintInput {
    pub mint_params: MintParams,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerGetMetadataInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerGetResourceTypeInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerGetTotalSupplyInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerUpdateMetadataInput {
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ResourceManagerMethod {
    GetNonFungible(NonFungibleId),
    NonFungibleExists(NonFungibleId),
    UpdateNonFungibleData(NonFungibleId, Vec<u8>),
}

impl ResourceManagerMethod {
    pub fn name(&self) -> &str {
        match self {
            ResourceManagerMethod::GetNonFungible(_) => "get_non_fungible",
            ResourceManagerMethod::NonFungibleExists(_) => "non_fungible_exists",
            ResourceManagerMethod::UpdateNonFungibleData(_, _) => "update_non_fungible_data",
        }
    }
}

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceAddress(pub [u8; 26]);

impl ResourceAddress {}

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub(crate) ResourceAddress);

impl ResourceManager {
    pub fn set_mintable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "update_auth".to_string(),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::Mint,
                access_rule,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    pub fn set_burnable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "update_auth".to_string(),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::Burn,
                access_rule,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    pub fn set_withdrawable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "update_auth".to_string(),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::Withdraw,
                access_rule,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    pub fn set_depositable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "update_auth".to_string(),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::Deposit,
                access_rule,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    pub fn set_updateable_metadata(&self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "update_auth".to_string(),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::UpdateMetadata,
                access_rule,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    pub fn set_updateable_non_fungible_data(&self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "update_auth".to_string(),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::UpdateNonFungibleData,
                access_rule,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    pub fn lock_mintable(&mut self) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "lock_auth".to_string(),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::Mint,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    pub fn lock_burnable(&mut self) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "lock_auth".to_string(),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::Burn,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    pub fn lock_withdrawable(&mut self) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "lock_auth".to_string(),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::Withdraw,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    pub fn lock_depositable(&mut self) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "lock_auth".to_string(),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::Deposit,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    pub fn lock_updateable_metadata(&mut self) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "lock_auth".to_string(),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::UpdateMetadata,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }

    pub fn lock_updateable_non_fungible_data(&mut self) -> () {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "lock_auth".to_string(),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::UpdateNonFungibleData,
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }


    fn mint_internal(&mut self, mint_params: MintParams) -> Bucket {
        let input = RadixEngineInput::InvokeSNode2(
            SNodeRef::ResourceRef(self.0),
            "mint".to_string(),
            scrypto_encode(&ResourceManagerMintInput {
                mint_params
            }),
        );
        let output: Vec<u8> = call_engine(input);
        scrypto_decode(&output).unwrap()
    }


    sfunctions2! {
        SNodeRef::ResourceRef(self.0) => {
            pub fn metadata(&self) -> HashMap<String, String> {
                ResourceManagerGetMetadataInput {}
            }
            pub fn resource_type(&self) -> () {
                ResourceManagerGetResourceTypeInput {}
            }
            pub fn total_supply(&self) -> Decimal {
                ResourceManagerGetTotalSupplyInput {}
            }
            pub fn update_metadata(&mut self, metadata: HashMap<String, String>) -> () {
                ResourceManagerUpdateMetadataInput {
                    metadata
                }
            }
        }
    }

    sfunctions! {
        SNodeRef::ResourceRef(self.0) => {
            fn get_non_fungible_data_internal(&self, id: NonFungibleId) -> [Vec<u8>; 2] {
                ResourceManagerMethod::GetNonFungible(id)
            }
            fn update_non_fungible_data_internal(&mut self, id: NonFungibleId, new_data: Vec<u8>) -> () {
                ResourceManagerMethod::UpdateNonFungibleData(id, new_data)
            }
            pub fn non_fungible_exists(&self, id: &NonFungibleId) -> bool {
                ResourceManagerMethod::NonFungibleExists(id.clone())
            }
        }
    }

    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&mut self, amount: T) -> Bucket {
        self.mint_internal(MintParams::Fungible {
            amount: amount.into(),
        })
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible<T: NonFungibleData>(&mut self, id: &NonFungibleId, data: T) -> Bucket {
        let mut entries = HashMap::new();
        entries.insert(id.clone(), (data.immutable_data(), data.mutable_data()));
        self.mint_internal(MintParams::NonFungible { entries })
    }

    /// Burns a bucket of resources.
    pub fn burn(&self, bucket: Bucket) {
        bucket.burn()
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleId) -> T {
        let non_fungible = self.get_non_fungible_data_internal(id.clone());
        T::decode(&non_fungible[0], &non_fungible[1]).unwrap()
    }

    /// Updates the mutable part of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn update_non_fungible_data<T: NonFungibleData>(
        &mut self,
        id: &NonFungibleId,
        new_data: T,
    ) {
        self.update_non_fungible_data_internal(id.clone(), new_data.mutable_data())
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
