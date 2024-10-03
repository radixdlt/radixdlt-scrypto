use crate::blueprints::transaction_processor::{IntentProcessor, ResumeResult};
use crate::errors::RuntimeError;
use crate::internal_prelude::{Sbor, ScryptoEncode, ScryptoSbor};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_common::crypto::Hash;
use radix_common::prelude::{GlobalAddressReservation, Reference};
use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_rust::prelude::*;
use radix_rust::prelude::{IndexMap, IndexSet};
use radix_transactions::model::InstructionV1;

#[cfg(not(feature = "coverage"))]
pub const MAX_TOTAL_BLOB_SIZE_PER_INVOCATION: usize = 1024 * 1024;
#[cfg(feature = "coverage")]
pub const MAX_TOTAL_BLOB_SIZE_PER_INVOCATION: usize = 64 * 1024 * 1024;

/// The minor version of the TransactionProcessor V1 package
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Sbor)]
pub enum TransactionProcessorV1MinorVersion {
    Zero,
    One,
}

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct TransactionProcessorRunInput {
    pub manifest_encoded_instructions: Vec<u8>,
    pub global_address_reservations: Vec<GlobalAddressReservation>,
    pub references: Vec<Reference>, // Required so that the kernel passes the references to the processor frame
    pub blobs: IndexMap<Hash, Vec<u8>>,
}

// This needs to match the above, but is easily encodable to avoid cloning from the transaction payload to encode
#[derive(Debug, Eq, PartialEq, ScryptoEncode)]
pub struct TransactionProcessorRunInputEfficientEncodable {
    pub manifest_encoded_instructions: Rc<Vec<u8>>,
    pub global_address_reservations: Vec<GlobalAddressReservation>,
    pub references: Rc<IndexSet<Reference>>,
    pub blobs: Rc<IndexMap<Hash, Vec<u8>>>,
}

pub struct TransactionProcessorBlueprint;

impl TransactionProcessorBlueprint {
    pub(crate) fn run<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<L>,
        L: Default,
    >(
        manifest_encoded_instructions: Vec<u8>,
        global_address_reservations: Vec<GlobalAddressReservation>,
        _references: Vec<Reference>, // Required so that the kernel passes the references to the processor frame
        blobs: IndexMap<Hash, Vec<u8>>,
        version: TransactionProcessorV1MinorVersion,
        api: &mut Y,
    ) -> Result<Vec<InstructionOutput>, RuntimeError> {
        let max_total_size_of_blobs = match version {
            TransactionProcessorV1MinorVersion::Zero => usize::MAX,
            TransactionProcessorV1MinorVersion::One => MAX_TOTAL_BLOB_SIZE_PER_INVOCATION,
        };
        let mut txn_processor_single_thread = IntentProcessor::<InstructionV1>::init(
            Rc::new(manifest_encoded_instructions),
            global_address_reservations,
            Rc::new(blobs),
            max_total_size_of_blobs,
            api,
        )?;
        let resume_result = txn_processor_single_thread.resume(None, api)?;
        if !matches!(resume_result, ResumeResult::RootIntentDone) {
            panic!("Unexpected yield occurred in v1 transaction processing");
        }
        Ok(txn_processor_single_thread.outputs)
    }
}
