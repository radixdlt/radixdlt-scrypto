use super::*;
use crate::internal_prelude::*;

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
/// the batch is fixed.
///
/// The use of lazy Generator traits is designed to allow the content of batch groups /
/// batches to be resolved lazily (e.g. with input from the database).
pub trait ProtocolUpdateGenerator: 'static {
    fn insert_status_tracking_flash_transactions(&self) -> bool {
        true
    }

    /// Return the list of batch groups for the protocol update.
    ///
    /// Each should be a fixed, conceptual step in the update process.
    fn batch_groups(&self) -> Vec<Box<dyn ProtocolUpdateBatchGroupGenerator + '_>>;
}

/// Each batch group is a logical grouping of batches.
///
/// For example, at genesis, there are three batch groups:
/// * `"bootstrap"` (flash + bootstrap transaction)
/// * `"chunks"`
/// * `"wrap-up"`
/// * The node also adds a `"scenarios"` batch group.
pub trait ProtocolUpdateBatchGroupGenerator<'a> {
    /// This is `&'static` because batch groups are intended to be fixed conceptual steps
    /// in the protocol update.
    ///
    /// The batch-group name should be kebab-case for consistency.
    fn batch_group_name(&self) -> &'static str;

    /// The content of these batches must be *fully reproducible* from the state of the store
    /// *before any updates were committed*. This is why we return an array of batch generators.
    ///
    /// If a protocol update needs to do some complicated/inline batch updates to substates, you may need to:
    /// * Have a first batch group where the planned work is saved batch-by-batch to some special partition
    /// * Have a second batch group where the planned work is performed, by reading from this special partition
    /// * Have a third batch group where the planned work is deleted
    fn generate_batches(
        self: Box<Self>,
        store: &dyn SubstateDatabase,
    ) -> Vec<Box<dyn ProtocolUpdateBatchGenerator + 'a>>;
}

/// Generate a batch of transactions to be committed atomically with a proof.
///
/// It should be assumed that the [`SubstateDatabase`] has *committed all previous batches*.
/// This ensures that the update is deterministically continuable if the node shuts down
/// mid-update.
pub trait ProtocolUpdateBatchGenerator {
    /// The batch name should be kebab-case for consistency
    fn batch_name(&self) -> &str;

    /// Generates the content of the batch
    fn generate_batch(self: Box<Self>, store: &dyn SubstateDatabase) -> ProtocolUpdateBatch;
}

pub(super) struct NoOpGenerator;

impl ProtocolUpdateGenerator for NoOpGenerator {
    fn batch_groups(&self) -> Vec<Box<dyn ProtocolUpdateBatchGroupGenerator>> {
        vec![]
    }
}

/// A simple batch group generator, which knows its batches in advance.
///
/// For some protocol updates, you might want to use a custom batch group generator,
/// which is more lazy, or sources its work from the database.
pub struct FixedBatchGroupGenerator<'a> {
    name: &'static str,
    batches: Vec<Box<dyn ProtocolUpdateBatchGenerator + 'a>>,
}

impl<'a> FixedBatchGroupGenerator<'a> {
    pub fn named(name: &'static str) -> Self {
        if name != name.to_ascii_lowercase().as_str() {
            panic!("Protocol update batch group names should be in kebab-case for consistency");
        }
        Self {
            name,
            batches: vec![],
        }
    }

    pub fn add_bespoke_batch(mut self, batch: impl ProtocolUpdateBatchGenerator + 'a) -> Self {
        self.batches.push(Box::new(batch));
        self
    }

    pub fn add_batch(
        self,
        name: impl Into<String>,
        generator: impl FnOnce(&dyn SubstateDatabase) -> ProtocolUpdateBatch + 'a,
    ) -> Self {
        self.add_bespoke_batch(BatchGenerator::new(name, generator))
    }

    pub fn build(self) -> Box<dyn ProtocolUpdateBatchGroupGenerator<'a> + 'a> {
        Box::new(self)
    }
}

impl<'a> ProtocolUpdateBatchGroupGenerator<'a> for FixedBatchGroupGenerator<'a> {
    fn batch_group_name(&self) -> &'static str {
        self.name
    }

    fn generate_batches(
        self: Box<Self>,
        _store: &dyn SubstateDatabase,
    ) -> Vec<Box<dyn ProtocolUpdateBatchGenerator + 'a>> {
        self.batches
    }
}

pub struct BatchGenerator<'a> {
    name: String,
    generator: Box<dyn FnOnce(&dyn SubstateDatabase) -> ProtocolUpdateBatch + 'a>,
}

impl<'a> BatchGenerator<'a> {
    pub fn new(
        name: impl Into<String>,
        generator: impl FnOnce(&dyn SubstateDatabase) -> ProtocolUpdateBatch + 'a,
    ) -> Self {
        let name = name.into();
        if name.to_ascii_lowercase() != name {
            panic!("Protocol update batch names should be in kebab-case for consistency");
        }
        Self {
            name,
            generator: Box::new(generator),
        }
    }

    pub fn build(self) -> Box<dyn ProtocolUpdateBatchGenerator + 'a> {
        Box::new(self)
    }
}

impl<'a> ProtocolUpdateBatchGenerator for BatchGenerator<'a> {
    fn batch_name(&self) -> &str {
        self.name.as_str()
    }

    fn generate_batch(self: Box<Self>, store: &dyn SubstateDatabase) -> ProtocolUpdateBatch {
        (self.generator)(store)
    }
}
