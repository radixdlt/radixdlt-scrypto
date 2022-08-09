use sbor::rust::collections::HashMap;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::engine::types::RENodeId;

use crate::abi::*;
use crate::address::{AddressError, BECH32_DECODER, BECH32_ENCODER};
use crate::buffer::scrypto_encode;
use crate::core::{FnIdentifier, Receiver};
use crate::engine::{api::*, call_engine};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::sfunctions;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
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
pub struct ResourceManagerCreateInput {
    pub resource_type: ResourceType,
    pub metadata: HashMap<String, String>,
    pub access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
    pub mint_params: Option<MintParams>,
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
pub struct ResourceManagerUpdateNonFungibleDataInput {
    pub id: NonFungibleId,
    pub data: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerNonFungibleExistsInput {
    pub id: NonFungibleId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerGetNonFungibleInput {
    pub id: NonFungibleId,
}

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceAddress(pub [u8; 27]);

impl ResourceAddress {}

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub(crate) ResourceAddress);

impl ResourceManager {
    pub fn set_mintable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("update_auth".to_string()),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::Mint,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn set_burnable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("update_auth".to_string()),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::Burn,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn set_withdrawable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("update_auth".to_string()),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::Withdraw,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn set_depositable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("update_auth".to_string()),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::Deposit,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn set_updateable_metadata(&self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("update_auth".to_string()),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::UpdateMetadata,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn set_updateable_non_fungible_data(&self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("update_auth".to_string()),
            scrypto_encode(&ResourceManagerUpdateAuthInput {
                method: ResourceMethodAuthKey::UpdateNonFungibleData,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn lock_mintable(&mut self) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("lock_auth".to_string()),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::Mint,
            }),
        );
        call_engine(input)
    }

    pub fn lock_burnable(&mut self) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("lock_auth".to_string()),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::Burn,
            }),
        );
        call_engine(input)
    }

    pub fn lock_withdrawable(&mut self) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("lock_auth".to_string()),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::Withdraw,
            }),
        );
        call_engine(input)
    }

    pub fn lock_depositable(&mut self) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("lock_auth".to_string()),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::Deposit,
            }),
        );
        call_engine(input)
    }

    pub fn lock_updateable_metadata(&mut self) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("lock_auth".to_string()),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::UpdateMetadata,
            }),
        );
        call_engine(input)
    }

    pub fn lock_updateable_non_fungible_data(&mut self) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("lock_auth".to_string()),
            scrypto_encode(&ResourceManagerLockAuthInput {
                method: ResourceMethodAuthKey::UpdateNonFungibleData,
            }),
        );
        call_engine(input)
    }

    fn mint_internal(&mut self, mint_params: MintParams) -> Bucket {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("mint".to_string()),
            scrypto_encode(&ResourceManagerMintInput { mint_params }),
        );
        call_engine(input)
    }

    fn update_non_fungible_data_internal(&mut self, id: NonFungibleId, data: Vec<u8>) -> () {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("update_non_fungible_data".to_string()),
            scrypto_encode(&ResourceManagerUpdateNonFungibleDataInput { id, data }),
        );
        call_engine(input)
    }

    fn get_non_fungible_data_internal(&self, id: NonFungibleId) -> [Vec<u8>; 2] {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(self.0)),
            FnIdentifier::Native("non_fungible_data".to_string()),
            scrypto_encode(&ResourceManagerGetNonFungibleInput { id }),
        );
        call_engine(input)
    }

    sfunctions! {
        Receiver::Ref(RENodeId::ResourceManager(self.0)) => {
            pub fn metadata(&self) -> HashMap<String, String> {
                ResourceManagerGetMetadataInput {}
            }
            pub fn resource_type(&self) -> ResourceType {
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
            pub fn non_fungible_exists(&self, id: &NonFungibleId) -> bool {
                ResourceManagerNonFungibleExistsInput {
                    id: id.clone()
                }
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
// binary
//========

impl TryFrom<&[u8]> for ResourceAddress {
    type Error = AddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => Ok(Self(copy_u8_array(slice))),
            _ => Err(AddressError::InvalidLength(slice.len())),
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

impl FromStr for ResourceAddress {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BECH32_DECODER.validate_and_decode_resource_address(s)
    }
}

impl fmt::Display for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            BECH32_ENCODER.encode_resource_address(self).unwrap()
        )
    }
}

impl fmt::Debug for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
