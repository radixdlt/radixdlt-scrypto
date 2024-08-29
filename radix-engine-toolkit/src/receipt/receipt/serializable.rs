use super::base::*;
use super::runtime::*;
use crate::receipt::{AsHex, AsStr, ToolkitReceiptError};
use radix_common::prelude::{
    AddressBech32Decoder, AddressBech32Encoder, Decimal as RuntimeDecimal,
    Ed25519PublicKey as RuntimeEd25519PublicKey,
    Ed25519PublicKeyHash as RuntimeEd25519PublicKeyHash, GlobalAddress as RuntimeGlobalAddress,
    Instant as RuntimeInstant, NonFungibleGlobalId as RuntimeNonFungibleGlobalId,
    NonFungibleLocalId as RuntimeNonFungibleLocalId, PublicKey as RuntimePublicKey,
    PublicKeyHash as RuntimePublicKeyHash, ResourceAddress as RuntimeResourceAddress,
    Secp256k1PublicKey as RuntimeSecp256k1PublicKey,
    Secp256k1PublicKeyHash as RuntimeSecp256k1PublicKeyHash,
};
use radix_common::prelude::{ContextualTryFrom, ContextualTryInto};
use radix_engine::system::system_modules::execution_trace::{
    ResourceSpecifier as RuntimeResourceSpecifier, WorktopChange as RuntimeWorktopChange,
};
use radix_engine_interface::prelude::{
    MetadataValue as RuntimeMetadataValue, NodeId as RuntimeNodeId,
    UncheckedOrigin as RuntimeOrigin, UncheckedUrl as RuntimeUrl,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializableTypeSelector;
impl TypeSelector for SerializableTypeSelector {
    type Usize = Usize;
    type Bytes = Bytes;
    type Decimal = Decimal;

    type NodeId = NodeId;
    type NonFungibleGlobalId = NonFungibleGlobalId;

    type MetadataValue = MetadataValue;

    type WorktopChange = WorktopChange;
}

/// The serializable toolkit transaction receipt.
pub type SerializableToolkitTransactionReceipt =
    ToolkitTransactionReceipt<SerializableTypeSelector>;

impl SerializableToolkitTransactionReceipt {
    pub fn into_runtime_receipt(
        self,
        address_encoder: &AddressBech32Decoder,
    ) -> Result<RuntimeToolkitTransactionReceipt, ToolkitReceiptError> {
        self.contextual_try_into(address_encoder)
    }
}

// The types used in this module - these are all serializable types through serde.
type NodeId = String;
type Decimal = AsStr<RuntimeDecimal>;
type NonFungibleLocalId = AsStr<RuntimeNonFungibleLocalId>;
type NonFungibleGlobalId = String;
type Usize = AsStr<usize>;
type U8 = AsStr<u8>;
type U32 = AsStr<u32>;
type U64 = AsStr<u64>;
type I32 = AsStr<i32>;
type I64 = AsStr<i64>;
type Bytes = AsHex<Vec<u8>>;
type FixedSizeBytes<const N: usize> = AsHex<[u8; N]>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value")]
pub enum WorktopChange {
    Take(ResourceSpecifier),
    Put(ResourceSpecifier),
}

impl ContextualTryFrom<WorktopChange> for RuntimeWorktopChange {
    type Context = AddressBech32Decoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: WorktopChange,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        match value {
            WorktopChange::Take(value) => value.contextual_try_into(context).map(Self::Take),
            WorktopChange::Put(value) => value.contextual_try_into(context).map(Self::Put),
        }
    }
}

impl ContextualTryFrom<RuntimeWorktopChange> for WorktopChange {
    type Context = AddressBech32Encoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: RuntimeWorktopChange,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        match value {
            RuntimeWorktopChange::Take(value) => value.contextual_try_into(context).map(Self::Take),
            RuntimeWorktopChange::Put(value) => value.contextual_try_into(context).map(Self::Put),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum ResourceSpecifier {
    Amount {
        resource_address: NodeId,
        amount: Decimal,
    },
    Ids {
        resource_address: NodeId,
        ids: Vec<NonFungibleLocalId>,
    },
}

impl ContextualTryFrom<ResourceSpecifier> for RuntimeResourceSpecifier {
    type Context = AddressBech32Decoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: ResourceSpecifier,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        match value {
            ResourceSpecifier::Amount {
                resource_address,
                amount,
            } => Ok(RuntimeResourceSpecifier::Amount(
                RuntimeResourceAddress::try_from_bech32(&context, &resource_address)
                    .ok_or(ToolkitReceiptError::InvalidResourceAddress)?,
                amount.into_inner(),
            )),
            ResourceSpecifier::Ids {
                resource_address,
                ids,
            } => Ok(RuntimeResourceSpecifier::Ids(
                RuntimeResourceAddress::try_from_bech32(&context, &resource_address)
                    .ok_or(ToolkitReceiptError::InvalidResourceAddress)?,
                ids.into_iter().map(|value| value.into_inner()).collect(),
            )),
        }
    }
}

impl ContextualTryFrom<RuntimeResourceSpecifier> for ResourceSpecifier {
    type Context = AddressBech32Encoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: RuntimeResourceSpecifier,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        match value {
            RuntimeResourceSpecifier::Amount(resource_address, amount) => Ok(Self::Amount {
                resource_address: context.encode(resource_address.as_bytes())?,
                amount: amount.into(),
            }),
            RuntimeResourceSpecifier::Ids(resource_address, ids) => Ok(Self::Ids {
                resource_address: context.encode(resource_address.as_bytes())?,
                ids: ids.into_iter().map(|value| value.into()).collect(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value")]
pub enum MetadataValue {
    String(String),
    Bool(bool),
    U8(U8),
    U32(U32),
    U64(U64),
    I32(I32),
    I64(I64),
    Decimal(Decimal),
    GlobalAddress(NodeId),
    PublicKey(PublicKey),
    NonFungibleGlobalId(NonFungibleGlobalId),
    NonFungibleLocalId(NonFungibleLocalId),
    Instant(I64),
    Url(String),
    Origin(String),
    PublicKeyHash(PublicKeyHash),

    StringArray(Vec<String>),
    BoolArray(Vec<bool>),
    U8Array(Vec<U8>),
    U32Array(Vec<U32>),
    U64Array(Vec<U64>),
    I32Array(Vec<I32>),
    I64Array(Vec<I64>),
    DecimalArray(Vec<Decimal>),
    GlobalAddressArray(Vec<NodeId>),
    PublicKeyArray(Vec<PublicKey>),
    NonFungibleGlobalIdArray(Vec<NonFungibleGlobalId>),
    NonFungibleLocalIdArray(Vec<NonFungibleLocalId>),
    InstantArray(Vec<I64>),
    UrlArray(Vec<String>),
    OriginArray(Vec<String>),
    PublicKeyHashArray(Vec<PublicKeyHash>),
}

impl ContextualTryFrom<MetadataValue> for RuntimeMetadataValue {
    type Context = AddressBech32Decoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: MetadataValue,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            MetadataValue::String(value) => Self::String(value),
            MetadataValue::Bool(value) => Self::Bool(value),
            MetadataValue::U8(value) => Self::U8(value.into_inner()),
            MetadataValue::U32(value) => Self::U32(value.into_inner()),
            MetadataValue::U64(value) => Self::U64(value.into_inner()),
            MetadataValue::I32(value) => Self::I32(value.into_inner()),
            MetadataValue::I64(value) => Self::I64(value.into_inner()),
            MetadataValue::Decimal(value) => Self::Decimal(value.into_inner()),
            MetadataValue::GlobalAddress(value) => {
                RuntimeGlobalAddress::try_from_bech32(&context, &value)
                    .map(Self::GlobalAddress)
                    .ok_or(ToolkitReceiptError::InvalidGlobalAddress)?
            }
            MetadataValue::PublicKey(value) => Self::PublicKey(value.into()),
            MetadataValue::NonFungibleGlobalId(value) => {
                RuntimeNonFungibleGlobalId::try_from_canonical_string(&context, &value)
                    .map(Self::NonFungibleGlobalId)
                    .map_err(|_| ToolkitReceiptError::InvalidNonFungibleGlobalId)?
            }
            MetadataValue::NonFungibleLocalId(value) => {
                Self::NonFungibleLocalId(value.into_inner())
            }
            MetadataValue::Instant(value) => Self::Instant(RuntimeInstant::new(value.into_inner())),
            MetadataValue::Url(value) => Self::Url(RuntimeUrl::of(value)),
            MetadataValue::Origin(value) => Self::Origin(RuntimeOrigin::of(value)),
            MetadataValue::PublicKeyHash(value) => Self::PublicKeyHash(value.into()),
            MetadataValue::StringArray(value) => Self::StringArray(value),
            MetadataValue::BoolArray(value) => Self::BoolArray(value),
            MetadataValue::U8Array(value) => {
                Self::U8Array(value.into_iter().map(|value| value.into_inner()).collect())
            }
            MetadataValue::U32Array(value) => {
                Self::U32Array(value.into_iter().map(|value| value.into_inner()).collect())
            }
            MetadataValue::U64Array(value) => {
                Self::U64Array(value.into_iter().map(|value| value.into_inner()).collect())
            }
            MetadataValue::I32Array(value) => {
                Self::I32Array(value.into_iter().map(|value| value.into_inner()).collect())
            }
            MetadataValue::I64Array(value) => {
                Self::I64Array(value.into_iter().map(|value| value.into_inner()).collect())
            }
            MetadataValue::DecimalArray(value) => {
                Self::DecimalArray(value.into_iter().map(|value| value.into_inner()).collect())
            }
            MetadataValue::GlobalAddressArray(value) => value
                .into_iter()
                .map(|value| {
                    RuntimeGlobalAddress::try_from_bech32(&context, &value)
                        .ok_or(ToolkitReceiptError::InvalidGlobalAddress)
                })
                .collect::<Result<_, _>>()
                .map(Self::GlobalAddressArray)?,
            MetadataValue::PublicKeyArray(value) => {
                Self::PublicKeyArray(value.into_iter().map(|value| value.into()).collect())
            }
            MetadataValue::NonFungibleGlobalIdArray(value) => value
                .into_iter()
                .map(|value| {
                    RuntimeNonFungibleGlobalId::try_from_canonical_string(&context, &value)
                        .map_err(|_| ToolkitReceiptError::InvalidNonFungibleGlobalId)
                })
                .collect::<Result<_, _>>()
                .map(Self::NonFungibleGlobalIdArray)?,
            MetadataValue::NonFungibleLocalIdArray(value) => Self::NonFungibleLocalIdArray(
                value.into_iter().map(|value| value.into_inner()).collect(),
            ),
            MetadataValue::InstantArray(value) => Self::InstantArray(
                value
                    .into_iter()
                    .map(|value| RuntimeInstant::new(value.into_inner()))
                    .collect(),
            ),
            MetadataValue::UrlArray(value) => {
                Self::UrlArray(value.into_iter().map(RuntimeUrl::of).collect())
            }
            MetadataValue::OriginArray(value) => {
                Self::OriginArray(value.into_iter().map(RuntimeOrigin::of).collect())
            }
            MetadataValue::PublicKeyHashArray(value) => {
                Self::PublicKeyHashArray(value.into_iter().map(|value| value.into()).collect())
            }
        })
    }
}

impl ContextualTryFrom<RuntimeMetadataValue> for MetadataValue {
    type Context = AddressBech32Encoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: RuntimeMetadataValue,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            RuntimeMetadataValue::String(value) => Self::String(value),
            RuntimeMetadataValue::Bool(value) => Self::Bool(value),
            RuntimeMetadataValue::U8(value) => Self::U8(value.into()),
            RuntimeMetadataValue::U32(value) => Self::U32(value.into()),
            RuntimeMetadataValue::U64(value) => Self::U64(value.into()),
            RuntimeMetadataValue::I32(value) => Self::I32(value.into()),
            RuntimeMetadataValue::I64(value) => Self::I64(value.into()),
            RuntimeMetadataValue::Decimal(value) => Self::Decimal(value.into()),
            RuntimeMetadataValue::GlobalAddress(value) => context
                .encode(value.as_node_id().as_bytes())
                .map(Self::GlobalAddress)?,
            RuntimeMetadataValue::PublicKey(value) => Self::PublicKey(value.into()),
            RuntimeMetadataValue::NonFungibleGlobalId(value) => {
                Self::NonFungibleGlobalId(value.to_canonical_string(&context))
            }
            RuntimeMetadataValue::NonFungibleLocalId(value) => {
                Self::NonFungibleLocalId(value.into())
            }
            RuntimeMetadataValue::Instant(value) => {
                Self::Instant(value.seconds_since_unix_epoch.into())
            }
            RuntimeMetadataValue::Url(value) => Self::Url(value.0),
            RuntimeMetadataValue::Origin(value) => Self::Origin(value.0),
            RuntimeMetadataValue::PublicKeyHash(value) => Self::PublicKeyHash(value.into()),
            RuntimeMetadataValue::StringArray(value) => Self::StringArray(value),
            RuntimeMetadataValue::BoolArray(value) => Self::BoolArray(value),
            RuntimeMetadataValue::U8Array(value) => {
                Self::U8Array(value.into_iter().map(|value| value.into()).collect())
            }
            RuntimeMetadataValue::U32Array(value) => {
                Self::U32Array(value.into_iter().map(|value| value.into()).collect())
            }
            RuntimeMetadataValue::U64Array(value) => {
                Self::U64Array(value.into_iter().map(|value| value.into()).collect())
            }
            RuntimeMetadataValue::I32Array(value) => {
                Self::I32Array(value.into_iter().map(|value| value.into()).collect())
            }
            RuntimeMetadataValue::I64Array(value) => {
                Self::I64Array(value.into_iter().map(|value| value.into()).collect())
            }
            RuntimeMetadataValue::DecimalArray(value) => {
                Self::DecimalArray(value.into_iter().map(|value| value.into()).collect())
            }
            RuntimeMetadataValue::GlobalAddressArray(value) => value
                .into_iter()
                .map(|value| context.encode(value.as_bytes()))
                .collect::<Result<_, _>>()
                .map(Self::GlobalAddressArray)?,
            RuntimeMetadataValue::PublicKeyArray(value) => {
                Self::PublicKeyArray(value.into_iter().map(|value| value.into()).collect())
            }
            RuntimeMetadataValue::NonFungibleGlobalIdArray(value) => {
                Self::NonFungibleGlobalIdArray(
                    value
                        .into_iter()
                        .map(|item| item.to_canonical_string(&context))
                        .collect(),
                )
            }
            RuntimeMetadataValue::NonFungibleLocalIdArray(value) => {
                Self::NonFungibleLocalIdArray(value.into_iter().map(|value| value.into()).collect())
            }
            RuntimeMetadataValue::InstantArray(value) => Self::InstantArray(
                value
                    .into_iter()
                    .map(|value| value.seconds_since_unix_epoch.into())
                    .collect(),
            ),
            RuntimeMetadataValue::UrlArray(value) => {
                Self::UrlArray(value.into_iter().map(|value| value.0).collect())
            }
            RuntimeMetadataValue::OriginArray(value) => {
                Self::OriginArray(value.into_iter().map(|value| value.0).collect())
            }
            RuntimeMetadataValue::PublicKeyHashArray(value) => {
                Self::PublicKeyHashArray(value.into_iter().map(|value| value.into()).collect())
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value")]
pub enum PublicKey {
    Secp256k1(FixedSizeBytes<{ radix_common::prelude::Secp256k1PublicKey::LENGTH }>),
    Ed25519(FixedSizeBytes<{ radix_common::prelude::Ed25519PublicKey::LENGTH }>),
}

impl From<PublicKey> for RuntimePublicKey {
    fn from(value: PublicKey) -> Self {
        match value {
            PublicKey::Secp256k1(value) => {
                Self::Secp256k1(RuntimeSecp256k1PublicKey(value.into_inner()))
            }
            PublicKey::Ed25519(value) => Self::Ed25519(RuntimeEd25519PublicKey(value.into_inner())),
        }
    }
}

impl From<RuntimePublicKey> for PublicKey {
    fn from(value: RuntimePublicKey) -> Self {
        match value {
            RuntimePublicKey::Secp256k1(value) => Self::Secp256k1(value.0.into()),
            RuntimePublicKey::Ed25519(value) => Self::Ed25519(value.0.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value")]
pub enum PublicKeyHash {
    Secp256k1(FixedSizeBytes<{ radix_common::prelude::Secp256k1PublicKeyHash::LENGTH }>),
    Ed25519(FixedSizeBytes<{ radix_common::prelude::Ed25519PublicKeyHash::LENGTH }>),
}

impl From<PublicKeyHash> for RuntimePublicKeyHash {
    fn from(value: PublicKeyHash) -> Self {
        match value {
            PublicKeyHash::Secp256k1(value) => {
                Self::Secp256k1(RuntimeSecp256k1PublicKeyHash(value.into_inner()))
            }
            PublicKeyHash::Ed25519(value) => {
                Self::Ed25519(RuntimeEd25519PublicKeyHash(value.into_inner()))
            }
        }
    }
}

impl From<RuntimePublicKeyHash> for PublicKeyHash {
    fn from(value: RuntimePublicKeyHash) -> Self {
        match value {
            RuntimePublicKeyHash::Secp256k1(value) => Self::Secp256k1(value.0.into()),
            RuntimePublicKeyHash::Ed25519(value) => Self::Ed25519(value.0.into()),
        }
    }
}

impl From<FeeSummary<SerializableTypeSelector>> for FeeSummary<RuntimeTypeSelector> {
    fn from(value: FeeSummary<SerializableTypeSelector>) -> Self {
        Self {
            execution_fees_in_xrd: value.execution_fees_in_xrd.into_inner(),
            finalization_fees_in_xrd: value.finalization_fees_in_xrd.into_inner(),
            storage_fees_in_xrd: value.storage_fees_in_xrd.into_inner(),
            royalty_fees_in_xrd: value.royalty_fees_in_xrd.into_inner(),
        }
    }
}

impl From<FeeSummary<RuntimeTypeSelector>> for FeeSummary<SerializableTypeSelector> {
    fn from(value: FeeSummary<RuntimeTypeSelector>) -> Self {
        Self {
            execution_fees_in_xrd: value.execution_fees_in_xrd.into(),
            finalization_fees_in_xrd: value.finalization_fees_in_xrd.into(),
            storage_fees_in_xrd: value.storage_fees_in_xrd.into(),
            royalty_fees_in_xrd: value.royalty_fees_in_xrd.into(),
        }
    }
}

impl From<LockedFees<SerializableTypeSelector>> for LockedFees<RuntimeTypeSelector> {
    fn from(value: LockedFees<SerializableTypeSelector>) -> Self {
        Self {
            contingent: value.contingent.into_inner(),
            non_contingent: value.non_contingent.into_inner(),
        }
    }
}

impl From<LockedFees<RuntimeTypeSelector>> for LockedFees<SerializableTypeSelector> {
    fn from(value: LockedFees<RuntimeTypeSelector>) -> Self {
        Self {
            contingent: value.contingent.into(),
            non_contingent: value.non_contingent.into(),
        }
    }
}

impl ContextualTryFrom<MetadataUpdate<SerializableTypeSelector>>
    for MetadataUpdate<RuntimeTypeSelector>
{
    type Context = AddressBech32Decoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: MetadataUpdate<SerializableTypeSelector>,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        match value {
            MetadataUpdate::Set(value) => value.contextual_try_into(context).map(Self::Set),
            MetadataUpdate::Delete => Ok(Self::Delete),
        }
    }
}

impl ContextualTryFrom<MetadataUpdate<RuntimeTypeSelector>>
    for MetadataUpdate<SerializableTypeSelector>
{
    type Context = AddressBech32Encoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: MetadataUpdate<RuntimeTypeSelector>,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        match value {
            MetadataUpdate::Set(value) => value.contextual_try_into(context).map(Self::Set),
            MetadataUpdate::Delete => Ok(Self::Delete),
        }
    }
}

impl ContextualTryFrom<StateUpdatesSummary<SerializableTypeSelector>>
    for StateUpdatesSummary<RuntimeTypeSelector>
{
    type Context = AddressBech32Decoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: StateUpdatesSummary<SerializableTypeSelector>,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            new_entities: value
                .new_entities
                .into_iter()
                .map(|value| {
                    context
                        .validate_and_decode(&value)
                        .map_err(|_| ToolkitReceiptError::InvalidNodeId)
                        .and_then(|(_, value)| {
                            value
                                .try_into()
                                .map(RuntimeNodeId)
                                .map_err(|_| ToolkitReceiptError::InvalidNodeId)
                        })
                })
                .collect::<Result<_, _>>()?,
            metadata_updates: value
                .metadata_updates
                .into_iter()
                .map(|(node_id, updates)| {
                    context
                        .validate_and_decode(&node_id)
                        .map_err(|_| ToolkitReceiptError::InvalidNodeId)
                        .and_then(|(_, value)| {
                            value
                                .try_into()
                                .map(RuntimeNodeId)
                                .map_err(|_| ToolkitReceiptError::InvalidNodeId)
                        })
                        .and_then(|node_id| {
                            updates
                                .into_iter()
                                .map(|(metadata_key, metadata_update)| {
                                    MetadataUpdate::<RuntimeTypeSelector>::contextual_try_from(
                                        metadata_update,
                                        context,
                                    )
                                    .map(|value| (metadata_key, value))
                                })
                                .collect::<Result<_, _>>()
                                .map(|value| (node_id, value))
                        })
                })
                .collect::<Result<_, _>>()?,
            non_fungible_data_updates: value
                .non_fungible_data_updates
                .into_iter()
                .map(|(global_id, bytes)| {
                    RuntimeNonFungibleGlobalId::try_from_canonical_string(
                        &AddressBech32Decoder {
                            hrp_set: context.hrp_set.clone(),
                        },
                        &global_id,
                    )
                    .map_err(|_| ToolkitReceiptError::InvalidNonFungibleGlobalId)
                    .map(|global_id| (global_id, bytes.into_inner()))
                })
                .collect::<Result<_, _>>()?,
            newly_minted_non_fungibles: value
                .newly_minted_non_fungibles
                .into_iter()
                .map(|value| {
                    RuntimeNonFungibleGlobalId::try_from_canonical_string(
                        &AddressBech32Decoder {
                            hrp_set: context.hrp_set.clone(),
                        },
                        &value,
                    )
                    .map_err(|_| ToolkitReceiptError::InvalidNonFungibleGlobalId)
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

impl ContextualTryFrom<StateUpdatesSummary<RuntimeTypeSelector>>
    for StateUpdatesSummary<SerializableTypeSelector>
{
    type Context = AddressBech32Encoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: StateUpdatesSummary<RuntimeTypeSelector>,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            new_entities: value
                .new_entities
                .into_iter()
                .map(|value| context.encode(value.as_bytes()))
                .collect::<Result<_, _>>()?,
            metadata_updates: value
                .metadata_updates
                .into_iter()
                .map(|(node_id, updates)| {
                    context
                        .encode(node_id.as_bytes())
                        .map_err(ToolkitReceiptError::from)
                        .and_then(|node_id| {
                            updates
                                .into_iter()
                                .map(|(metadata_key, metadata_value)| {
                                    MetadataUpdate::<SerializableTypeSelector>::contextual_try_from(
                                        metadata_value,
                                        context,
                                    )
                                    .map(|value| (metadata_key, value))
                                })
                                .collect::<Result<_, _>>()
                                .map(|value| (node_id, value))
                        })
                })
                .collect::<Result<_, _>>()?,
            non_fungible_data_updates: value
                .non_fungible_data_updates
                .into_iter()
                .map(|(key, value)| (key.to_canonical_string(&context), value.into()))
                .collect(),
            newly_minted_non_fungibles: value
                .newly_minted_non_fungibles
                .into_iter()
                .map(|value| value.to_canonical_string(&context))
                .collect(),
        })
    }
}

impl ContextualTryFrom<ToolkitTransactionReceipt<SerializableTypeSelector>>
    for ToolkitTransactionReceipt<RuntimeTypeSelector>
{
    type Context = AddressBech32Decoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: ToolkitTransactionReceipt<SerializableTypeSelector>,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            ToolkitTransactionReceipt::CommitSuccess {
                state_updates_summary,
                worktop_changes,
                fee_summary,
                locked_fees,
            } => ToolkitTransactionReceipt::CommitSuccess {
                state_updates_summary: state_updates_summary.contextual_try_into(context)?,
                worktop_changes: worktop_changes
                    .into_iter()
                    .map(|(key, value)| {
                        value
                            .into_iter()
                            .map(|value| value.contextual_try_into(context))
                            .collect::<Result<_, _>>()
                            .map(|value| (key.into_inner(), value))
                    })
                    .collect::<Result<_, _>>()?,
                fee_summary: fee_summary.into(),
                locked_fees: locked_fees.into(),
            },
            ToolkitTransactionReceipt::CommitFailure { reason } => {
                ToolkitTransactionReceipt::CommitFailure { reason }
            }
            ToolkitTransactionReceipt::Reject { reason } => {
                ToolkitTransactionReceipt::Reject { reason }
            }
            ToolkitTransactionReceipt::Abort { reason } => {
                ToolkitTransactionReceipt::Abort { reason }
            }
        })
    }
}

impl ContextualTryFrom<ToolkitTransactionReceipt<RuntimeTypeSelector>>
    for ToolkitTransactionReceipt<SerializableTypeSelector>
{
    type Context = AddressBech32Encoder;
    type Error = ToolkitReceiptError;

    fn contextual_try_from(
        value: ToolkitTransactionReceipt<RuntimeTypeSelector>,
        context: &Self::Context,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            ToolkitTransactionReceipt::CommitSuccess {
                state_updates_summary,
                worktop_changes,
                fee_summary,
                locked_fees,
            } => ToolkitTransactionReceipt::CommitSuccess {
                state_updates_summary: state_updates_summary.contextual_try_into(context)?,
                worktop_changes: worktop_changes
                    .into_iter()
                    .map(|(key, value)| {
                        value
                            .into_iter()
                            .map(|value| value.contextual_try_into(context))
                            .collect::<Result<_, _>>()
                            .map(|value| (key.into(), value))
                    })
                    .collect::<Result<_, _>>()?,
                fee_summary: fee_summary.into(),
                locked_fees: locked_fees.into(),
            },
            ToolkitTransactionReceipt::CommitFailure { reason } => {
                ToolkitTransactionReceipt::CommitFailure { reason }
            }
            ToolkitTransactionReceipt::Reject { reason } => {
                ToolkitTransactionReceipt::Reject { reason }
            }
            ToolkitTransactionReceipt::Abort { reason } => {
                ToolkitTransactionReceipt::Abort { reason }
            }
        })
    }
}
