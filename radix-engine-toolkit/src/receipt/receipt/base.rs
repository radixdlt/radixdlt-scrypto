use core::hash::Hash;
use radix_common::prelude::{IndexMap, IndexSet};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// The transaction receipt DTO which is used in the communication between the CoreAPI and the Radix
/// Engine Toolkit.
///
/// A core strategy followed by this model and this work stream in general is that we want there
/// to be as little interpretation as possible done by the toolkit and for the core api or whatever
/// service that provides the receipt to provide the toolkit with exactly with the information that
/// it requires.
///
/// This is nice since it means that the data the toolkit needs can be provided through any means
/// the provider sees fit. As an example, the worktop changes are currently derived from the
/// execution trace. There are two options for how the toolkit could get this data:
///
/// 1. It could be given the execution trace and it can derive that data itself from the trace.
/// 2. It can be given that data without needing to worry about its source or how it was derived or
///    obtained.
///
/// The first approach would make it a problem for us to switch to other ways of getting this data
/// while the second approach does not. If we were to switch to events for that then it's a simple
/// change to the core-api to get this data in a different way and it would have no effect on the
/// toolkit.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    bound = "
        T::Usize: Serialize + DeserializeOwned,
        T::Bytes: Serialize + DeserializeOwned,
        T::NodeId: Serialize + DeserializeOwned,
        T::Decimal: Serialize + DeserializeOwned,
        T::MetadataValue: Serialize + DeserializeOwned,
        T::WorktopChange: Serialize + DeserializeOwned,
        T::NonFungibleGlobalId: Serialize + DeserializeOwned,
    "
)]
#[allow(clippy::large_enum_variant)]
pub enum ToolkitTransactionReceipt<T: TypeSelector + PartialEq + Eq> {
    /// The transaction would've been committed successfully if it were submitted to the ledger.
    CommitSuccess {
        /// The state updates summary from the transaction.
        state_updates_summary: StateUpdatesSummary<T>,
        /// The instruction-by-instruction worktop updates that took place in the transaction.
        worktop_changes: IndexMap<T::Usize, Vec<T::WorktopChange>>,
        /// The summary of the fees paid in the transaction.
        fee_summary: FeeSummary<T>,
        /// Information about the fees locked by the transaction.
        locked_fees: LockedFees<T>,
    },
    /// The transaction would've been committed failure if it were submitted
    /// to the ledger.
    CommitFailure {
        /// A [`String`] of the Rust debug formatted runtime error that caused the transaction to
        /// fail. This is not a JSON string or object since we do not currently offer any guarantees
        /// on the structure of the returned string.
        reason: String,
    },
    /// The transaction would've been rejected if it were submitted to the
    /// ledger.
    Reject {
        /// A [`String`] of the Rust debug formatted runtime error that caused the transaction to
        /// be rejected. This is not a JSON string or object since we do not currently offer any
        /// guarantees on the structure of the returned string.
        reason: String,
    },
    /// The transaction would've been aborted if it were submitted to the
    /// ledger.
    Abort {
        /// A [`String`] of the Rust debug formatted runtime error that caused the transaction to
        /// be aborted. This is not a JSON string or object since we do not currently offer any
        /// guarantees on the structure of the returned string.
        reason: String,
    },
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound = "T::Decimal: Serialize + DeserializeOwned")]
pub struct FeeSummary<T: TypeSelector> {
    pub execution_fees_in_xrd: T::Decimal,
    pub finalization_fees_in_xrd: T::Decimal,
    pub storage_fees_in_xrd: T::Decimal,
    pub royalty_fees_in_xrd: T::Decimal,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound = "T::Decimal: Serialize + DeserializeOwned")]
pub struct LockedFees<T: TypeSelector> {
    pub contingent: T::Decimal,
    pub non_contingent: T::Decimal,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound = "T::MetadataValue: Serialize + DeserializeOwned")]
pub enum MetadataUpdate<T: TypeSelector> {
    Set(T::MetadataValue),
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound = "
        T::Bytes: Serialize + DeserializeOwned,
        T::NodeId: Serialize + DeserializeOwned,
        T::Decimal: Serialize + DeserializeOwned,
        T::MetadataValue: Serialize + DeserializeOwned,
        T::NonFungibleGlobalId: Serialize + DeserializeOwned,
    ")]
pub struct StateUpdatesSummary<T: TypeSelector> {
    /// The set of new entities created in this transaction.
    pub new_entities: IndexSet<T::NodeId>,
    /// The metadata updates that were made in this transaction.
    pub metadata_updates: IndexMap<T::NodeId, IndexMap<String, MetadataUpdate<T>>>,
    /// The non-fungible data updates that were made in this transaction. Could be newly minted
    /// or updated non-fungible data.
    pub non_fungible_data_updates: IndexMap<T::NonFungibleGlobalId, T::Bytes>,
    /// The ids of the newly minted non-fungibles.
    pub newly_minted_non_fungibles: IndexSet<T::NonFungibleGlobalId>,
}

/// A trait that is used to allow for the [`ToolkitTransactionReceipt`] to use different types for
/// different fields. This allows the type to be multiplexed and used for both serialization as well
/// as runtime.
pub trait TypeSelector {
    /// The usize type to use.
    type Usize: core::fmt::Debug + PartialEq + Eq + Clone + Hash;
    /// The bytes type to use.
    type Bytes: core::fmt::Debug + PartialEq + Eq + Clone;
    /// The decimal type to use.
    type Decimal: core::fmt::Debug + PartialEq + Eq + Clone;

    /// The node id type to use.
    type NodeId: core::fmt::Debug + PartialEq + Eq + Clone + Hash;
    /// The non-fungible global id to use.
    type NonFungibleGlobalId: core::fmt::Debug + PartialEq + Eq + Clone + Hash;

    /// The metadata value to use.
    type MetadataValue: core::fmt::Debug + PartialEq + Eq + core::fmt::Debug + Clone;

    /// The type used for worktop updates.
    type WorktopChange: core::fmt::Debug + PartialEq + Eq + Clone;
}
