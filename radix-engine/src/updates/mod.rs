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

pub trait UpdateSettings: Sized {
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

#[derive(Clone)]
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

#[derive(Clone, Copy, Debug, Default)]
pub struct NoSettings;

impl UpdateSettingMarker for NoSettings {}

impl<T: DefaultForNetwork + UpdateSettingMarker> UpdateSetting<T> {
    pub fn enabled_as_default_for_network(network_definition: &NetworkDefinition) -> Self {
        Self::Enabled(T::default_for_network(network_definition))
    }
}

// TODO AFTER MERGE WITH NODE: Merge with UpdateBatchGenerator
/// This must be stateless, to allow the update to be resumed.
pub trait ProtocolUpdateBatchGenerator: ProtocolUpdateBatchGeneratorDynClone {
    fn status_tracking_enabled(&self) -> bool {
        true
    }

    /// Generate a batch of transactions to be committed atomically with a proof.
    /// *Panics* if the given batch index is outside the range (see [`Self::batch_count()`]).
    ///
    /// It should be assumed that the [`SubstateDatabase`] has *committed all previous batches*.
    /// This ensures that the update is deterministically continuable if the Node shuts down
    /// mid-update.
    ///
    /// TODO(potential API improvement): This is the interface currently needed by the Node, to
    /// allow the update to be resumed; it is not great, and we could improve this in future.
    fn generate_batch(
        &self,
        store: &dyn SubstateDatabase,
        batch_group_index: usize,
        batch_index: usize,
    ) -> ProtocolUpdateBatch;

    /// Returns the number of contained batch groups.
    /// Each batch group is a logical grouping of batches.
    /// For example, at genesis, there are three batch groups:
    /// * Bootstrap (Flash + Bootstrap Txn)
    /// * Chunk Execution
    /// * Wrap up
    ///
    /// The [`Self::generate_batch()`] expects the `batch_group_index`
    /// to be in the range `[0, self.batch_group_descriptors().len() - 1]`.
    fn batch_group_descriptors(&self) -> Vec<String>;

    /// Returns the number of contained batches.
    /// For a fixed batch group, [`Self::generate_batch()`] expects `batch_index`
    /// to be in the range `[0, self.batch_count() - 1]`.
    fn batch_count(&self, batch_group_index: usize) -> usize;
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

    fn batch_count(&self, _batch_group_index: usize) -> usize {
        0
    }
}
