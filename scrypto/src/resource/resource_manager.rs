use crate::{sfunctions};
use sbor::*;
use crate::buffer::scrypto_decode;
use crate::core::SNodeRef;
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

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ResourceManagerFunction {
    Create(
        ResourceType,
        HashMap<String, String>,
        HashMap<ResourceMethod, (MethodAuth, Mutability)>,
        Option<MintParams>,
    )
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ResourceManagerMethod {
    Mint(MintParams),
    UpdateAuth(ResourceMethod, MethodAuth),
    LockAuth(ResourceMethod),
    GetResourceType(),
    GetMetadata(),
    GetTotalSupply(),
    GetNonFungible(NonFungibleId),
    NonFungibleExists(NonFungibleId),
    UpdateNonFungibleData(NonFungibleId, Vec<u8>),
    UpdateMetadata(HashMap<String, String>),
    CreateVault(),
    CreateBucket(),
}

impl ResourceManagerMethod {
    pub fn name(&self) -> &str {
        match self {
            ResourceManagerMethod::Mint(_) => "mint",
            ResourceManagerMethod::UpdateAuth(_, _) => "update_auth",
            ResourceManagerMethod::LockAuth(_) => "lock_auth",
            ResourceManagerMethod::GetResourceType() => "get_resource_type",
            ResourceManagerMethod::GetMetadata() => "get_metadata",
            ResourceManagerMethod::GetTotalSupply() => "get_total_supply",
            ResourceManagerMethod::GetNonFungible(_) => "get_non_fungible",
            ResourceManagerMethod::NonFungibleExists(_) => "non_fungible_exists",
            ResourceManagerMethod::UpdateNonFungibleData(_, _) => "update_non_fungible_data",
            ResourceManagerMethod::UpdateMetadata(_) => "update_metadata",
            ResourceManagerMethod::CreateVault() => "create_vault",
            ResourceManagerMethod::CreateBucket() => "create_bucket",
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
    sfunctions! {
        SNodeRef::ResourceRef(self.0) => {
            fn mint_internal(&mut self, mint_params: MintParams) -> Bucket {
                ResourceManagerMethod::Mint(mint_params)
            }
            pub fn set_mintable(&mut self, mint_auth: MethodAuth) -> () {
                ResourceManagerMethod::UpdateAuth(ResourceMethod::Mint, mint_auth)
            }
            pub fn lock_mintable(&mut self) -> () {
                ResourceManagerMethod::LockAuth(ResourceMethod::Mint)
            }
            pub fn set_burnable(&mut self, burn_auth: MethodAuth) -> () {
                ResourceManagerMethod::UpdateAuth(ResourceMethod::Burn, burn_auth)
            }
            pub fn lock_burnable(&mut self) -> () {
                ResourceManagerMethod::LockAuth(ResourceMethod::Burn)
            }
            pub fn set_withdrawable(&mut self, withdraw_auth: MethodAuth) -> () {
                ResourceManagerMethod::UpdateAuth(ResourceMethod::Withdraw, withdraw_auth)
            }
            pub fn lock_withdrawable(&mut self) -> () {
                ResourceManagerMethod::LockAuth(ResourceMethod::Withdraw)
            }
            pub fn set_depositable(&mut self, deposit_auth: MethodAuth) -> () {
                ResourceManagerMethod::UpdateAuth(ResourceMethod::Deposit, deposit_auth)
            }
            pub fn lock_depositable(&mut self) -> () {
                ResourceManagerMethod::LockAuth(ResourceMethod::Deposit)
            }
            pub fn set_updateable_metadata(&self, update_metadata_auth: MethodAuth) -> () {
                ResourceManagerMethod::UpdateAuth(ResourceMethod::UpdateMetadata, update_metadata_auth)
            }
            pub fn lock_updateable_metadata(&mut self) -> () {
                ResourceManagerMethod::LockAuth(ResourceMethod::UpdateMetadata)
            }
            pub fn set_updateable_non_fungible_data(&self, update_non_fungible_data_auth: MethodAuth) -> () {
                ResourceManagerMethod::UpdateAuth(ResourceMethod::UpdateNonFungibleData, update_non_fungible_data_auth)
            }
            pub fn lock_updateable_non_fungible_data(&mut self) -> () {
                ResourceManagerMethod::LockAuth(ResourceMethod::UpdateNonFungibleData)
            }
            pub fn metadata(&self) -> HashMap<String, String> {
                ResourceManagerMethod::GetMetadata()
            }
            pub fn total_supply(&self) -> Decimal {
                ResourceManagerMethod::GetTotalSupply()
            }
            fn get_non_fungible_data_internal(&self, id: NonFungibleId) -> [Vec<u8>; 2] {
                ResourceManagerMethod::GetNonFungible(id)
            }
            fn update_non_fungible_data_internal(&mut self, id: NonFungibleId, new_data: Vec<u8>) -> () {
                ResourceManagerMethod::UpdateNonFungibleData(id, new_data)
            }
            pub fn non_fungible_exists(&self, id: &NonFungibleId) -> bool {
                ResourceManagerMethod::NonFungibleExists(id.clone())
            }
            pub fn update_metadata(&mut self, new_metadata: HashMap<String, String>) -> () {
                ResourceManagerMethod::UpdateMetadata(new_metadata)
            }
            pub fn resource_type(&self) -> () {
                ResourceManagerMethod::GetResourceType()
            }
        }
    }

    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&mut self, amount: T) -> Bucket {
        self.mint_internal(MintParams::Fungible { amount: amount.into()})
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
    pub fn update_non_fungible_data<T: NonFungibleData>(&mut self, id: &NonFungibleId, new_data: T) {
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
