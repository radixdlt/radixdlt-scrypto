use crate::{internal_prelude::*, track::StateUpdates};
mod anemone;
mod bottlenose;
mod protocol_builder;
mod protocol_updates;

pub use anemone::*;
pub use bottlenose::*;
pub use protocol_builder::*;
pub use protocol_updates::*;

// TODO AFTER MERGE WITH NODE: Replace with node's UpdateTransaction
pub enum ProtocolUpdateTransactionDetails {
    FlashV1Transaction(FlashProtocolUpdateTransactionDetails),
}

impl ProtocolUpdateTransactionDetails {
    pub fn flash(name: &str, state_updates: StateUpdates) -> Self {
        Self::FlashV1Transaction(FlashProtocolUpdateTransactionDetails {
            name: name.to_string(),
            state_updates,
        })
    }
}

// TODO AFTER MERGE WITH NODE: Merge replace with node's FlashTransactionV1
pub struct FlashProtocolUpdateTransactionDetails {
    pub name: String,
    pub state_updates: StateUpdates,
}

/// A set of transactions which all get committed together with the same proof.
/// To avoid memory overflows, this should be kept small (e.g. one transaction each).
pub struct ProtocolUpdateBatch {
    pub transactions: Vec<ProtocolUpdateTransactionDetails>,
}

pub trait UpdateSettings: Sized {
    type BatchGenerator: ProtocolUpdateBatchGenerator;

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
    /// Generate a batch of transactions to be committed atomically with a proof.
    /// *Panics* if the given batch index is outside the range (see [`Self::batch_count()`]).
    ///
    /// It should be assumed that the [`SubstateDatabase`] has *committed all previous batches*.
    /// This ensures that the update is deterministically continuable if the Node shuts down
    /// mid-update.
    ///
    /// TODO(potential API improvement): This is the interface currently needed by the Node, to
    /// allow the update to be resumed; it is not great, and we could improve this in future.
    fn generate_batch(&self, store: &dyn SubstateDatabase, batch_index: u32)
        -> ProtocolUpdateBatch;

    /// Returns the number of contained batches.
    /// The [`Self::generate_batch()`] expects indices in range `[0, self.batch_count() - 1]`.
    fn batch_count(&self) -> u32;
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
