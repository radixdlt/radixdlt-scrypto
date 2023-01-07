use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto_abi::*;
use utils::{copy_u8_array, ContextualDisplay};

use crate::address::*;
use crate::api::api::*;
use crate::data::types::Own;
use crate::data::ScryptoCustomTypeId;
use crate::math::*;
use crate::model::*;
use crate::scrypto_type;
use crate::wasm::*;

use crate::scrypto;
use crate::Describe;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum VaultMethodAuthKey {
    Withdraw,
    Deposit,
    Recall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum ResourceMethodAuthKey {
    Mint,
    Burn,
    UpdateNonFungibleData,
    UpdateMetadata,
    Withdraw,
    Deposit,
    Recall,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerCreateInvocation {
    pub resource_type: ResourceType,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

impl Invocation for ResourceManagerCreateInvocation {
    type Output = ResourceAddress;
}

impl SerializableInvocation for ResourceManagerCreateInvocation {
    type ScryptoOutput = ResourceAddress;
}

impl Into<SerializedInvocation> for ResourceManagerCreateInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::Create(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerCreateWithInitialSupplyInvocation {
    pub resource_type: ResourceType,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub mint_params: MintParams,
}

impl Invocation for ResourceManagerCreateWithInitialSupplyInvocation {
    type Output = (ResourceAddress, Bucket);
}

impl SerializableInvocation for ResourceManagerCreateWithInitialSupplyInvocation {
    type ScryptoOutput = (ResourceAddress, Bucket);
}

impl Into<SerializedInvocation> for ResourceManagerCreateWithInitialSupplyInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::CreateWithInitialSupply(self)).into()
    }
}

#[derive(Debug, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerBucketBurnInvocation {
    pub bucket: Bucket,
}

impl Clone for ResourceManagerBucketBurnInvocation {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

impl Invocation for ResourceManagerBucketBurnInvocation {
    type Output = ();
}

impl SerializableInvocation for ResourceManagerBucketBurnInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ResourceManagerBucketBurnInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::BurnBucket(self)).into()
    }
}

#[derive(Debug, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerBurnInvocation {
    pub receiver: ResourceAddress,
    pub bucket: Bucket,
}

impl Clone for ResourceManagerBurnInvocation {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver,
            bucket: Bucket(self.bucket.0),
        }
    }
}

impl Invocation for ResourceManagerBurnInvocation {
    type Output = ();
}

impl SerializableInvocation for ResourceManagerBurnInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ResourceManagerBurnInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::Burn(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerUpdateVaultAuthInvocation {
    pub receiver: ResourceAddress,
    pub method: VaultMethodAuthKey,
    pub access_rule: AccessRule,
}

impl Invocation for ResourceManagerUpdateVaultAuthInvocation {
    type Output = ();
}

impl SerializableInvocation for ResourceManagerUpdateVaultAuthInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ResourceManagerUpdateVaultAuthInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::UpdateVaultAuth(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerSetVaultAuthMutabilityInvocation {
    pub receiver: ResourceAddress,
    pub method: VaultMethodAuthKey,
    pub mutability: AccessRule,
}

impl Invocation for ResourceManagerSetVaultAuthMutabilityInvocation {
    type Output = ();
}

impl SerializableInvocation for ResourceManagerSetVaultAuthMutabilityInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ResourceManagerSetVaultAuthMutabilityInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::LockVaultAuth(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerCreateVaultInvocation {
    pub receiver: ResourceAddress,
}

impl Invocation for ResourceManagerCreateVaultInvocation {
    type Output = Own;
}

impl SerializableInvocation for ResourceManagerCreateVaultInvocation {
    type ScryptoOutput = Own;
}

impl Into<SerializedInvocation> for ResourceManagerCreateVaultInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::CreateVault(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerCreateBucketInvocation {
    pub receiver: ResourceAddress,
}

impl Invocation for ResourceManagerCreateBucketInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for ResourceManagerCreateBucketInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<SerializedInvocation> for ResourceManagerCreateBucketInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::CreateBucket(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerMintInvocation {
    pub receiver: ResourceAddress,
    pub mint_params: MintParams,
}

impl Invocation for ResourceManagerMintInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for ResourceManagerMintInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<SerializedInvocation> for ResourceManagerMintInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::Mint(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerGetResourceTypeInvocation {
    pub receiver: ResourceAddress,
}

impl Invocation for ResourceManagerGetResourceTypeInvocation {
    type Output = ResourceType;
}

impl SerializableInvocation for ResourceManagerGetResourceTypeInvocation {
    type ScryptoOutput = ResourceType;
}

impl Into<SerializedInvocation> for ResourceManagerGetResourceTypeInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::GetResourceType(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerGetTotalSupplyInvocation {
    pub receiver: ResourceAddress,
}

impl Invocation for ResourceManagerGetTotalSupplyInvocation {
    type Output = Decimal;
}

impl SerializableInvocation for ResourceManagerGetTotalSupplyInvocation {
    type ScryptoOutput = Decimal;
}

impl Into<SerializedInvocation> for ResourceManagerGetTotalSupplyInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::GetTotalSupply(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerUpdateNonFungibleDataInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleId,
    pub data: Vec<u8>,
}

impl Invocation for ResourceManagerUpdateNonFungibleDataInvocation {
    type Output = ();
}

impl SerializableInvocation for ResourceManagerUpdateNonFungibleDataInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ResourceManagerUpdateNonFungibleDataInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::UpdateNonFungibleData(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerNonFungibleExistsInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleId,
}

impl Invocation for ResourceManagerNonFungibleExistsInvocation {
    type Output = bool;
}

impl SerializableInvocation for ResourceManagerNonFungibleExistsInvocation {
    type ScryptoOutput = bool;
}

impl Into<SerializedInvocation> for ResourceManagerNonFungibleExistsInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::NonFungibleExists(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerGetNonFungibleInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleId,
}

impl Invocation for ResourceManagerGetNonFungibleInvocation {
    type Output = [Vec<u8>; 2];
}

impl SerializableInvocation for ResourceManagerGetNonFungibleInvocation {
    type ScryptoOutput = [Vec<u8>; 2];
}

impl Into<SerializedInvocation> for ResourceManagerGetNonFungibleInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::GetNonFungible(self)).into()
    }
}

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceAddress {
    Normal([u8; 26]),
}

//========
// binary
//========

impl TryFrom<&[u8]> for ResourceAddress {
    type Error = AddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => match EntityType::try_from(slice[0])
                .map_err(|_| AddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::Resource => Ok(Self::Normal(copy_u8_array(&slice[1..]))),
                _ => Err(AddressError::InvalidEntityTypeId(slice[0])),
            },
            _ => Err(AddressError::InvalidLength(slice.len())),
        }
    }
}

impl ResourceAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(EntityType::resource(self).id());
        match self {
            Self::Normal(v) => buf.extend(v),
        }
        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_vec())
    }

    pub fn try_from_hex(hex_str: &str) -> Result<Self, AddressError> {
        let bytes = hex::decode(hex_str).map_err(|_| AddressError::HexDecodingError)?;

        Self::try_from(bytes.as_ref())
    }
}

scrypto_type!(
    ResourceAddress,
    ScryptoCustomTypeId::ResourceAddress,
    Type::ResourceAddress,
    27
);

//======
// text
//======

impl fmt::Debug for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for ResourceAddress {
    type Error = AddressError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            return encoder.encode_resource_address_to_fmt(f, self);
        }

        // This could be made more performant by streaming the hex into the formatter
        match self {
            ResourceAddress::Normal(_) => {
                write!(f, "NormalResource[{}]", self.to_hex())
            }
        }
        .map_err(|err| AddressError::FormatError(err))
    }
}
