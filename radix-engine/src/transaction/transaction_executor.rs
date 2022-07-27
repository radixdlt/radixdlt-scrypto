use sbor::rust::marker::PhantomData;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::buffer::*;
use scrypto::values::ScryptoValue;
use transaction::model::*;
use transaction::validation::{IdAllocator, IdSpace};

use crate::engine::Track;
use crate::engine::*;
use crate::fee::FeeTable;
use crate::fee::{CostUnitCounter, SystemLoanCostUnitCounter};
use crate::ledger::{ReadableSubstateStore, WriteableSubstateStore};
use crate::model::*;
use crate::transaction::*;
use crate::wasm::*;

pub struct TransactionExecutorConfig {
    pub trace: bool,
}

impl TransactionExecutorConfig {
    pub fn new(trace: bool) -> Self {
        TransactionExecutorConfig { trace }
    }
}

/// An executor that runs transactions.
pub struct TransactionExecutor<'s, 'w, S, W, I>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    substate_store: &'s mut S,
    wasm_engine: &'w mut W,
    wasm_instrumenter: &'w mut WasmInstrumenter,
    config: TransactionExecutorConfig,
    phantom1: PhantomData<I>,
}

impl<'s, 'w, S, W, I> TransactionExecutor<'s, 'w, S, W, I>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn new(
        substate_store: &'s mut S,
        wasm_engine: &'w mut W,
        wasm_instrumenter: &'w mut WasmInstrumenter,
        config: TransactionExecutorConfig,
    ) -> TransactionExecutor<'s, 'w, S, W, I> {
        Self {
            substate_store,
            wasm_engine,
            wasm_instrumenter,
            config,
            phantom1: PhantomData,
        }
    }

    pub fn execute_and_commit<T: ExecutableTransaction>(
        &mut self,
        transaction: &T,
    ) -> TransactionReceipt {
        let receipt = self.execute(transaction);
        receipt.state_updates.commit(self.substate_store);
        receipt
    }

    pub fn execute<T: ExecutableTransaction>(&mut self, transaction: &T) -> TransactionReceipt {
        self.execute_with_cost_unit_counter(transaction, SystemLoanCostUnitCounter::default())
    }

    pub fn execute_with_cost_unit_counter<T: ExecutableTransaction, C: CostUnitCounter>(
        &mut self,
        transaction: &T,
        mut cost_unit_counter: C,
    ) -> TransactionReceipt {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();

        let transaction_hash = transaction.transaction_hash();
        let transaction_network = transaction.transaction_network();
        let signer_public_keys = transaction.signer_public_keys().to_vec();
        let instructions = transaction.instructions().to_vec();
        #[cfg(not(feature = "alloc"))]
        if self.config.trace {
            println!("{:-^80}", "Transaction Metadata");
            println!("Transaction hash: {}", transaction_hash);
            println!("Transaction network: {:?}", transaction_network);
            println!("Transaction signers: {:?}", signer_public_keys);

            println!("{:-^80}", "Engine Execution Log");
        }

        // 1. Start state track
        let mut track = Track::new(self.substate_store);
        let mut id_allocator = IdAllocator::new(IdSpace::Application);

        // 2. Apply pre-execution costing
        let fee_table = FeeTable::new();
        cost_unit_counter
            .consume(
                fee_table.tx_decoding_per_byte() * transaction.transaction_payload_size() as u32,
                "tx_decoding",
            )
            .expect("System loan should cover this");
        cost_unit_counter
            .consume(
                fee_table.tx_verification_per_byte()
                    * transaction.transaction_payload_size() as u32,
                "tx_verification",
            )
            .expect("System loan should cover this");
        cost_unit_counter
            .consume(
                fee_table.tx_signature_validation_per_sig()
                    * transaction.signer_public_keys().len() as u32,
                "signature_validation",
            )
            .expect("System loan should cover this");

        // 3. Start a call frame and run the transaction
        let mut root_frame = CallFrame::new_root(
            self.config.trace,
            transaction_hash,
            signer_public_keys,
            false,
            &mut id_allocator,
            &mut track,
            self.wasm_engine,
            self.wasm_instrumenter,
            &mut cost_unit_counter,
            &fee_table,
        );
        let result = root_frame
            .invoke_snode(
                scrypto::core::SNodeRef::TransactionProcessor,
                "run".to_string(),
                ScryptoValue::from_typed(&TransactionProcessorRunInput {
                    instructions: instructions.clone(),
                }),
            )
            .map(|o| scrypto_decode::<Vec<Vec<u8>>>(&o.raw).unwrap());

        // 4. Settle transaction fee
        let counter = root_frame.cost_unit_counter();
        // TODO: burn fee in a FILO order
        // TODO: reward validators
        // TODO: refund overpaid fees
        let system_loan_fully_repaid = counter.owed() == 0;
        let max_cost_units = counter.limit();
        let cost_units_consumed = counter.consumed();
        let cost_units_price = 0u32.into();
        let burned = 0u32.into();
        let tipped = 0u32.into();
        #[cfg(not(feature = "alloc"))]
        if self.config.trace {
            println!("{:-^80}", "Cost Analysis");
            for (k, v) in cost_unit_counter.analysis() {
                println!("{:<20}: {:>8}", k, v);
            }
        }

        // 5. Produce the final transaction receipt
        let track_receipt = track.to_receipt(result.is_ok());
        let mut new_component_addresses = Vec::new();
        let mut new_resource_addresses = Vec::new();
        let mut new_package_addresses = Vec::new();
        for address in track_receipt.new_addresses {
            match address {
                Address::GlobalComponent(component_address) => {
                    new_component_addresses.push(component_address)
                }
                Address::ResourceManager(resource_address) => {
                    new_resource_addresses.push(resource_address)
                }
                Address::Package(package_address) => new_package_addresses.push(package_address),
                _ => {}
            }
        }
        #[cfg(feature = "alloc")]
        let execution_time = None;
        #[cfg(not(feature = "alloc"))]
        let execution_time = Some(now.elapsed().as_millis());
        let receipt = TransactionReceipt {
            transaction_network,
            transaction_fee: TransactionFeeSummary {
                system_loan_fully_repaid,
                max_cost_units,
                cost_units_consumed,
                cost_units_price,
                burned,
                tipped,
            },
            instructions,
            result,
            application_logs: track_receipt.application_logs,
            new_package_addresses,
            new_component_addresses,
            new_resource_addresses,
            execution_time,
            state_updates: track_receipt.state_updates,
        };

        #[cfg(not(feature = "alloc"))]
        if self.config.trace {
            println!("{:-^80}", "Transaction Receipt");
            println!("{:?}", receipt);
            println!("{:-^80}", "");
        }

        receipt
    }
}
