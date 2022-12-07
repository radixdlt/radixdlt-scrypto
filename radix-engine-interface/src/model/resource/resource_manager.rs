use sbor::rust::collections::HashMap;
use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto_abi::*;
use utils::{copy_u8_array, ContextualDisplay};

use crate::address::*;
use crate::api::api::*;
use crate::data::ScryptoCustomTypeId;
use crate::math::*;
use crate::model::*;
use crate::scrypto_type;
use crate::wasm::*;

use radix_engine_derive::scrypto;

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

// TODO: Move this enum to another crate.
// The Radix Engine does not rely on or use this enum in any way. It is mainly syntactic sugar for
// Scrypto and the manifest builder. Therefore, it does not make sense to have this be in the
// radix-engine-interface crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum Mutability {
    LOCKED,
    MUTABLE(AccessRule),
}

impl From<Mutability> for AccessRule {
    fn from(val: Mutability) -> Self {
        match val {
            LOCKED => AccessRule::DenyAll,
            MUTABLE(rule) => rule,
        }
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerCreateInvocation {
    pub resource_type: ResourceType,
    pub metadata: HashMap<String, String>,
    pub access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub mint_params: Option<MintParams>,
}

impl Invocation for ResourceManagerCreateInvocation {
    type Output = (ResourceAddress, Option<Bucket>);
}

impl SerializableInvocation for ResourceManagerCreateInvocation {
    type ScryptoOutput = (ResourceAddress, Option<Bucket>);
}

impl Into<SerializedInvocation> for ResourceManagerCreateInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::ResourceManager(
            ResourceManagerFunctionInvocation::Create(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerCreateWithOwnerInvocation {
    pub resource_type: ResourceType,
    pub metadata: HashMap<String, String>,
    pub owner_badge: NonFungibleAddress,
    pub mint_params: Option<MintParams>,
}

impl Invocation for ResourceManagerCreateWithOwnerInvocation {
    type Output = (ResourceAddress, Option<Bucket>);
}

impl SerializableInvocation for ResourceManagerCreateWithOwnerInvocation {
    type ScryptoOutput = (ResourceAddress, Option<Bucket>);
}

impl Into<SerializedInvocation> for ResourceManagerCreateWithOwnerInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::ResourceManager(
            ResourceManagerFunctionInvocation::CreateWithOwner(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerBucketBurnInvocation {
    pub bucket: Bucket,
}

impl Invocation for ResourceManagerBucketBurnInvocation {
    type Output = ();
}

impl SerializableInvocation for ResourceManagerBucketBurnInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ResourceManagerBucketBurnInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::ResourceManager(
            ResourceManagerFunctionInvocation::BurnBucket(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerBurnInvocation {
    pub receiver: ResourceAddress,
    pub bucket: Bucket,
}

impl Invocation for ResourceManagerBurnInvocation {
    type Output = ();
}

impl SerializableInvocation for ResourceManagerBurnInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ResourceManagerBurnInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::Burn(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::UpdateVaultAuth(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::LockVaultAuth(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerCreateVaultInvocation {
    pub receiver: ResourceAddress,
}

impl Invocation for ResourceManagerCreateVaultInvocation {
    type Output = Vault;
}

impl SerializableInvocation for ResourceManagerCreateVaultInvocation {
    type ScryptoOutput = Vault;
}

impl Into<SerializedInvocation> for ResourceManagerCreateVaultInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::CreateVault(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::CreateBucket(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::Mint(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::GetResourceType(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::GetTotalSupply(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::UpdateNonFungibleData(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::NonFungibleExists(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::GetNonFungible(self),
        ))
        .into()
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
