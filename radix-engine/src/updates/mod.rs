use crate::internal_prelude::*;
mod anemone;
mod babylon;
mod bottlenose;
mod cuttlefish;
mod protocol_builder;
mod protocol_updates;

pub use anemone::*;
pub use babylon::*;
pub use bottlenose::*;
pub use cuttlefish::*;
pub use protocol_builder::*;
pub use protocol_updates::*;
use radix_transactions::model::{FlashTransactionV1, SystemTransactionV1};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolUpdateTransaction {
    FlashTransactionV1(FlashTransactionV1),
    SystemTransactionV1(ProtocolSystemTransactionV1),
}

impl From<FlashTransactionV1> for ProtocolUpdateTransaction {
    fn from(value: FlashTransactionV1) -> Self {
        Self::FlashTransactionV1(value)
    }
}

impl From<ProtocolSystemTransactionV1> for ProtocolUpdateTransaction {
    fn from(value: ProtocolSystemTransactionV1) -> Self {
        Self::SystemTransactionV1(value)
    }
}

impl ProtocolUpdateTransaction {
    pub fn flash(name: &str, state_updates: StateUpdates) -> Self {
        Self::FlashTransactionV1(FlashTransactionV1 {
            name: name.to_string(),
            state_updates,
        })
    }

    pub fn genesis_transaction(name: &str, transaction: SystemTransactionV1) -> Self {
        Self::SystemTransactionV1(ProtocolSystemTransactionV1 {
            name: name.to_string(),
            disable_auth: true,
            transaction,
        })
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            ProtocolUpdateTransaction::FlashTransactionV1(tx) => Some(tx.name.as_str()),
            ProtocolUpdateTransaction::SystemTransactionV1(tx) => Some(tx.name.as_str()),
        }
    }
}

/// At present, this isn't actually saved in the node - instead just the
/// SystemTransactionV1 is saved.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct ProtocolSystemTransactionV1 {
    pub name: String,
    pub disable_auth: bool,
    pub transaction: SystemTransactionV1,
}

/// A set of transactions which all get committed together with the same proof.
/// To avoid memory overflows, this should be kept small enough to comfortably fit into
/// memory (e.g. one transaction per batch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolUpdateBatch {
    pub transactions: Vec<ProtocolUpdateTransaction>,
}

impl ProtocolUpdateBatch {
    pub fn single(single_transaction: ProtocolUpdateTransaction) -> Self {
        Self {
            transactions: vec![single_transaction],
        }
    }
}

/// This requires [`ScryptoSbor`] so it can be used to override configuration in the node for tests.
pub trait UpdateSettings: Sized + ScryptoSbor {
    type BatchGenerator: ProtocolUpdateBatchGenerator;

    fn protocol_version() -> ProtocolVersion;

    fn all_enabled_as_default_for_network(network: &NetworkDefinition) -> Self;

    fn all_disabled() -> Self;

    fn create_batch_generator(&self) -> Self::BatchGenerator;

    fn enable(mut self, prop: impl FnOnce(&mut Self) -> &mut UpdateSetting<NoSettings>) -> Self {
        *prop(&mut self) = UpdateSetting::Enabled(NoSettings);
        self
    }

    fn enable_with<T: UpdateSettingMarker>(
        mut self,
        prop: impl FnOnce(&mut Self) -> &mut UpdateSetting<T>,
        setting: T,
    ) -> Self {
        *prop(&mut self) = UpdateSetting::Enabled(setting);
        self
    }

    fn disable<T: UpdateSettingMarker>(
        mut self,
        prop: impl FnOnce(&mut Self) -> &mut UpdateSetting<T>,
    ) -> Self {
        *prop(&mut self) = UpdateSetting::Disabled;
        self
    }

    fn set(mut self, updater: impl FnOnce(&mut Self)) -> Self {
        updater(&mut self);
        self
    }
}

pub trait DefaultForNetwork {
    fn default_for_network(network_definition: &NetworkDefinition) -> Self;
}

impl<T: Default> DefaultForNetwork for T {
    fn default_for_network(_: &NetworkDefinition) -> Self {
        Self::default()
    }
}

#[derive(Clone, Sbor)]
pub enum UpdateSetting<T: UpdateSettingMarker> {
    Enabled(T),
    Disabled,
}

impl UpdateSetting<NoSettings> {
    pub fn new(is_enabled: bool) -> Self {
        if is_enabled {
            Self::Enabled(NoSettings)
        } else {
            Self::Disabled
        }
    }
}

pub trait UpdateSettingMarker {}

#[derive(Clone, Copy, Debug, Default, Sbor)]
pub struct NoSettings;

impl UpdateSettingMarker for NoSettings {}

impl<T: DefaultForNetwork + UpdateSettingMarker> UpdateSetting<T> {
    pub fn enabled_as_default_for_network(network_definition: &NetworkDefinition) -> Self {
        Self::Enabled(T::default_for_network(network_definition))
    }
}

/// Generates batches for the protocol update. These are structured as:
/// * One or more named batch groups
/// * One or more batches under each batch group.
///   Each batch is committed separately in the node. Separating into batches allows the
///   node not to have to hold too much in memory at any given time.
///
/// The batch generation must be stateless (aside from the database), to allow the update
/// to be resumed in the node mid-way through after a restart.
///
/// Therefore any transient state required between batches must be stored in the database,
/// and we must ensure that whilst each batch group is executing, the content of
/// [`batch_count`][batch_count] and [`generate_batch`][generate_batch] is fixed. This
/// is explained further in the `generate_batch` docs.
///
/// [generate_batch]: ProtocolUpdateBatchGenerator::generate_batch
/// [batch_count]: ProtocolUpdateBatchGenerator::batch_count
pub trait ProtocolUpdateBatchGenerator: ProtocolUpdateBatchGeneratorDynClone {
    fn status_tracking_enabled(&self) -> bool {
        true
    }

    /// Generate a batch of transactions to be committed atomically with a proof.
    ///
    /// It should be assumed that the [`SubstateDatabase`] has *committed all previous batches*.
    /// This ensures that the update is deterministically continuable if the Node shuts down
    /// mid-update.
    ///
    /// If a protocol update needs to do some complicated/inline batch updates to substates, you may need to:
    /// * Have a first batch group where the planned work is saved batch-by-batch to some special partition
    /// * Have a second batch group where the planned work is performed, by reading from this special partition
    /// * Have a third batch group where the planned work is deleted
    ///
    /// ## Panics
    /// Should panic if:
    /// * Called with a `batch_group_index >= self.batch_group_descriptors().len()`
    /// * Called with a `batch_index >= self.batch_count(store, batch_group_index)`
    fn generate_batch(
        &self,
        store: &dyn SubstateDatabase,
        batch_group_index: usize,
        batch_index: usize,
    ) -> ProtocolUpdateBatch;

    /// Returns an "UpperCamelCase" descriptor for each batch group which forms part of the update.
    ///
    /// Each batch group is a logical grouping of batches.
    ///
    /// For example, at genesis, there are three batch groups:
    /// * "Bootstrap" (Flash + Bootstrap Txn)
    /// * "Chunks"
    /// * "WrapUp"
    /// * (And optionally "Scenarios", which is added by the node)
    ///
    /// The [`Self::generate_batch()`] expects the `batch_group_index`
    /// to be in the range `[0, self.batch_group_descriptors().len() - 1]`.
    fn batch_group_descriptors(&self) -> Vec<String>;

    /// Returns the number of contained batches in the given batch group.
    ///
    /// Whilst this takes a `store`, the count and content of batches MUST be *fixed*
    /// for the duration of the batch group's execution. This ensures the update can be
    /// resumed mid-way through. An example of how this might work is given in the
    /// `generate_batch` method.
    ///
    /// For a fixed batch group, [`generate_batch`][Self::generate_batch] expects `batch_index`
    /// to be in the range `[0, self.batch_count() - 1]`.
    fn batch_count(&self, store: &dyn SubstateDatabase, batch_group_index: usize) -> usize;
}

pub trait ProtocolUpdateBatchGeneratorDynClone {
    fn clone_box(&self) -> Box<dyn ProtocolUpdateBatchGenerator>;
}

impl<T> ProtocolUpdateBatchGeneratorDynClone for T
where
    T: 'static + ProtocolUpdateBatchGenerator + Clone,
{
    fn clone_box(&self) -> Box<dyn ProtocolUpdateBatchGenerator> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ProtocolUpdateBatchGenerator> {
    fn clone(&self) -> Box<dyn ProtocolUpdateBatchGenerator> {
        self.clone_box()
    }
}

#[derive(Clone)]
struct NoOpBatchGenerator;

impl ProtocolUpdateBatchGenerator for NoOpBatchGenerator {
    fn generate_batch(
        &self,
        _store: &dyn SubstateDatabase,
        _batch_group_index: usize,
        _batch_index: usize,
    ) -> ProtocolUpdateBatch {
        panic!("This should not be called because batch_group_descriptors is empty")
    }

    fn batch_group_descriptors(&self) -> Vec<String> {
        vec![]
    }

    fn batch_count(&self, _store: &dyn SubstateDatabase, _batch_group_index: usize) -> usize {
        0
    }
}
