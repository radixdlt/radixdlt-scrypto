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
    UpdateMetadata,
    UpdateNonFungibleData,
    VaultMethodKey(VaultMethodAuthKey),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum Mutability {
    LOCKED,
    MUTABLE(AccessRule),
}

impl Into<AccessRule> for Mutability {
    fn into(self) -> AccessRule {
        match self {
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
    pub access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
    pub mint_params: Option<MintParams>,
}

impl Invocation for ResourceManagerCreateInvocation {
    type Output = (ResourceAddress, Option<Bucket>);
}

impl ScryptoNativeInvocation for ResourceManagerCreateInvocation {
    type ScryptoOutput = (ResourceAddress, Option<Bucket>);
}

impl Into<NativeFnInvocation> for ResourceManagerCreateInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::ResourceManager(
            ResourceManagerFunctionInvocation::Create(self),
        ))
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

impl ScryptoNativeInvocation for ResourceManagerBucketBurnInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for ResourceManagerBucketBurnInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::ResourceManager(
            ResourceManagerFunctionInvocation::BurnBucket(self),
        ))
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

impl ScryptoNativeInvocation for ResourceManagerBurnInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for ResourceManagerBurnInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::Burn(self),
        ))
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

impl ScryptoNativeInvocation for ResourceManagerUpdateVaultAuthInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for ResourceManagerUpdateVaultAuthInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::UpdateAuth(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerLockAuthInvocation {
    pub receiver: ResourceAddress,
    pub method: ResourceMethodAuthKey,
}

impl Invocation for ResourceManagerLockAuthInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ResourceManagerLockAuthInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for ResourceManagerLockAuthInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::LockAuth(self),
        ))
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

impl ScryptoNativeInvocation for ResourceManagerCreateVaultInvocation {
    type ScryptoOutput = Vault;
}

impl Into<NativeFnInvocation> for ResourceManagerCreateVaultInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::CreateVault(self),
        ))
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

impl ScryptoNativeInvocation for ResourceManagerCreateBucketInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<NativeFnInvocation> for ResourceManagerCreateBucketInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::CreateBucket(self),
        ))
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

impl ScryptoNativeInvocation for ResourceManagerMintInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<NativeFnInvocation> for ResourceManagerMintInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::Mint(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerGetMetadataInvocation {
    pub receiver: ResourceAddress,
}

impl Invocation for ResourceManagerGetMetadataInvocation {
    type Output = HashMap<String, String>;
}

impl ScryptoNativeInvocation for ResourceManagerGetMetadataInvocation {
    type ScryptoOutput = HashMap<String, String>;
}

impl Into<NativeFnInvocation> for ResourceManagerGetMetadataInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::GetMetadata(self),
        ))
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

impl ScryptoNativeInvocation for ResourceManagerGetResourceTypeInvocation {
    type ScryptoOutput = ResourceType;
}

impl Into<NativeFnInvocation> for ResourceManagerGetResourceTypeInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::GetResourceType(self),
        ))
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

impl ScryptoNativeInvocation for ResourceManagerGetTotalSupplyInvocation {
    type ScryptoOutput = Decimal;
}

impl Into<NativeFnInvocation> for ResourceManagerGetTotalSupplyInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::GetTotalSupply(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerUpdateMetadataInvocation {
    pub receiver: ResourceAddress,
    pub metadata: HashMap<String, String>,
}

impl Invocation for ResourceManagerUpdateMetadataInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ResourceManagerUpdateMetadataInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for ResourceManagerUpdateMetadataInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::UpdateMetadata(self),
        ))
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

impl ScryptoNativeInvocation for ResourceManagerUpdateNonFungibleDataInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for ResourceManagerUpdateNonFungibleDataInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::UpdateNonFungibleData(self),
        ))
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

impl ScryptoNativeInvocation for ResourceManagerNonFungibleExistsInvocation {
    type ScryptoOutput = bool;
}

impl Into<NativeFnInvocation> for ResourceManagerNonFungibleExistsInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::NonFungibleExists(self),
        ))
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

impl ScryptoNativeInvocation for ResourceManagerGetNonFungibleInvocation {
    type ScryptoOutput = [Vec<u8>; 2];
}

impl Into<NativeFnInvocation> for ResourceManagerGetNonFungibleInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::GetNonFungible(self),
        ))
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
