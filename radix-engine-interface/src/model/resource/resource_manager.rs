use sbor::rust::collections::{BTreeMap, BTreeSet};
use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto_abi::*;
use utils::{copy_u8_array, ContextualDisplay};

use crate::address::*;
use crate::api::wasm::*;
use crate::api::*;
use crate::data::types::Own;
use crate::data::ScryptoCustomValueKind;
use crate::math::*;
use crate::model::*;
use crate::scrypto_type;

use crate::*;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
pub enum VaultMethodAuthKey {
    Withdraw,
    Deposit,
    Recall,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    LegacyDescribe,
)]
pub enum ResourceMethodAuthKey {
    Mint,
    Burn,
    UpdateNonFungibleData,
    UpdateMetadata,
    Withdraw,
    Deposit,
    Recall,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerCreateNonFungibleInvocation {
    pub resource_address: Option<[u8; 26]>, // TODO: Clean this up
    pub id_type: NonFungibleIdType,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

impl Invocation for ResourceManagerCreateNonFungibleInvocation {
    type Output = ResourceAddress;
}

impl SerializableInvocation for ResourceManagerCreateNonFungibleInvocation {
    type ScryptoOutput = ResourceAddress;
}

impl Into<CallTableInvocation> for ResourceManagerCreateNonFungibleInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::CreateNonFungible(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerCreateFungibleInvocation {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

impl Invocation for ResourceManagerCreateFungibleInvocation {
    type Output = ResourceAddress;
}

impl SerializableInvocation for ResourceManagerCreateFungibleInvocation {
    type ScryptoOutput = ResourceAddress;
}

impl Into<CallTableInvocation> for ResourceManagerCreateFungibleInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::CreateFungible(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerCreateNonFungibleWithInitialSupplyInvocation {
    pub id_type: NonFungibleIdType,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub entries: BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>,
}

impl Invocation for ResourceManagerCreateNonFungibleWithInitialSupplyInvocation {
    type Output = (ResourceAddress, Bucket);
}

impl SerializableInvocation for ResourceManagerCreateNonFungibleWithInitialSupplyInvocation {
    type ScryptoOutput = (ResourceAddress, Bucket);
}

impl Into<CallTableInvocation> for ResourceManagerCreateNonFungibleWithInitialSupplyInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::CreateNonFungibleWithInitialSupply(
            self,
        ))
        .into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerCreateUuidNonFungibleWithInitialSupplyInvocation {
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub entries: BTreeSet<(Vec<u8>, Vec<u8>)>,
}

impl Invocation for ResourceManagerCreateUuidNonFungibleWithInitialSupplyInvocation {
    type Output = (ResourceAddress, Bucket);
}

impl SerializableInvocation for ResourceManagerCreateUuidNonFungibleWithInitialSupplyInvocation {
    type ScryptoOutput = (ResourceAddress, Bucket);
}

impl Into<CallTableInvocation> for ResourceManagerCreateUuidNonFungibleWithInitialSupplyInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(
            ResourceInvocation::CreateUuidNonFungibleWithInitialSupply(self),
        )
        .into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerCreateFungibleWithInitialSupplyInvocation {
    pub resource_address: Option<[u8; 26]>, // TODO: Clean this up
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
}

impl Invocation for ResourceManagerCreateFungibleWithInitialSupplyInvocation {
    type Output = (ResourceAddress, Bucket);
}

impl SerializableInvocation for ResourceManagerCreateFungibleWithInitialSupplyInvocation {
    type ScryptoOutput = (ResourceAddress, Bucket);
}

impl Into<CallTableInvocation> for ResourceManagerCreateFungibleWithInitialSupplyInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::CreateFungibleWithInitialSupply(self))
            .into()
    }
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for ResourceManagerBucketBurnInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::BurnBucket(self)).into()
    }
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for ResourceManagerBurnInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::Burn(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for ResourceManagerUpdateVaultAuthInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::UpdateVaultAuth(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for ResourceManagerSetVaultAuthMutabilityInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::LockVaultAuth(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerCreateVaultInvocation {
    pub receiver: ResourceAddress,
}

impl Invocation for ResourceManagerCreateVaultInvocation {
    type Output = Own;
}

impl SerializableInvocation for ResourceManagerCreateVaultInvocation {
    type ScryptoOutput = Own;
}

impl Into<CallTableInvocation> for ResourceManagerCreateVaultInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::CreateVault(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerCreateBucketInvocation {
    pub receiver: ResourceAddress,
}

impl Invocation for ResourceManagerCreateBucketInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for ResourceManagerCreateBucketInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for ResourceManagerCreateBucketInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::CreateBucket(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerMintNonFungibleInvocation {
    pub receiver: ResourceAddress,
    pub entries: BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>,
}

impl Invocation for ResourceManagerMintNonFungibleInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for ResourceManagerMintNonFungibleInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for ResourceManagerMintNonFungibleInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::MintNonFungible(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerMintUuidNonFungibleInvocation {
    pub receiver: ResourceAddress,
    pub entries: Vec<(Vec<u8>, Vec<u8>)>,
}

impl Invocation for ResourceManagerMintUuidNonFungibleInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for ResourceManagerMintUuidNonFungibleInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for ResourceManagerMintUuidNonFungibleInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::MintUuidNonFungible(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerMintFungibleInvocation {
    pub receiver: ResourceAddress,
    pub amount: Decimal,
}

impl Invocation for ResourceManagerMintFungibleInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for ResourceManagerMintFungibleInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for ResourceManagerMintFungibleInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::MintFungible(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerGetResourceTypeInvocation {
    pub receiver: ResourceAddress,
}

impl Invocation for ResourceManagerGetResourceTypeInvocation {
    type Output = ResourceType;
}

impl SerializableInvocation for ResourceManagerGetResourceTypeInvocation {
    type ScryptoOutput = ResourceType;
}

impl Into<CallTableInvocation> for ResourceManagerGetResourceTypeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::GetResourceType(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerGetTotalSupplyInvocation {
    pub receiver: ResourceAddress,
}

impl Invocation for ResourceManagerGetTotalSupplyInvocation {
    type Output = Decimal;
}

impl SerializableInvocation for ResourceManagerGetTotalSupplyInvocation {
    type ScryptoOutput = Decimal;
}

impl Into<CallTableInvocation> for ResourceManagerGetTotalSupplyInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::GetTotalSupply(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerUpdateNonFungibleDataInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleLocalId,
    pub data: Vec<u8>,
}

impl Invocation for ResourceManagerUpdateNonFungibleDataInvocation {
    type Output = ();
}

impl SerializableInvocation for ResourceManagerUpdateNonFungibleDataInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for ResourceManagerUpdateNonFungibleDataInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::UpdateNonFungibleData(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerNonFungibleExistsInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleLocalId,
}

impl Invocation for ResourceManagerNonFungibleExistsInvocation {
    type Output = bool;
}

impl SerializableInvocation for ResourceManagerNonFungibleExistsInvocation {
    type ScryptoOutput = bool;
}

impl Into<CallTableInvocation> for ResourceManagerNonFungibleExistsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::ResourceManager(ResourceInvocation::NonFungibleExists(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerGetNonFungibleInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleLocalId,
}

impl Invocation for ResourceManagerGetNonFungibleInvocation {
    type Output = [Vec<u8>; 2];
}

impl SerializableInvocation for ResourceManagerGetNonFungibleInvocation {
    type ScryptoOutput = [Vec<u8>; 2];
}

impl Into<CallTableInvocation> for ResourceManagerGetNonFungibleInvocation {
    fn into(self) -> CallTableInvocation {
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
    pub fn raw(&self) -> [u8; 26] {
        match self {
            Self::Normal(v) => v.clone(),
        }
    }

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
    ScryptoCustomValueKind::ResourceAddress,
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
