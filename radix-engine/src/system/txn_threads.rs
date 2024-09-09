use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_transactions::model::InstructionV2;
use crate::blueprints::transaction_processor::{MAX_TOTAL_BLOB_SIZE_PER_INVOCATION, TxnProcessor};
use crate::errors::RuntimeError;
use crate::system::system::SystemService;
use crate::system::system_callback::SystemBasedKernelApi;

pub struct TxnThreads {
    pub threads: Vec<TxnProcessor<InstructionV2>>,
}

impl TxnThreads {
    pub fn execute<Y: SystemBasedKernelApi>(&mut self, api: &mut Y) -> Result<(), RuntimeError> {
        api.kernel_switch_thread(0)?;

        let mut system_service = SystemService::new(api);
        let mut txn_processor = self.threads.get_mut(0).unwrap();

        txn_processor.execute(&mut system_service)?;

        Ok(())
    }
}