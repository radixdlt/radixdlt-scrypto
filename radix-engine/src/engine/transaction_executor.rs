use sbor::rust::marker::PhantomData;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::buffer::*;
use scrypto::values::ScryptoValue;
use std::ops::DerefMut;
use transaction::model::*;
use transaction::validation::{IdAllocator, IdSpace};

use crate::engine::*;
use crate::fee::*;
use crate::ledger::*;
use crate::model::*;
use crate::state_manager::*;
use crate::wasm::*;

pub enum TransactionCostCounterConfig {
    SystemLoanAndMaxCost {
        system_loan_amount: u32,
        max_transaction_cost: u32,
    },
    UnlimitedLoanAndMaxCost {
        max_transaction_cost: u32,
    },
}

pub struct TransactionExecutorConfig {
    pub trace: bool,
    pub cost_counter_config: TransactionCostCounterConfig,
}

impl TransactionExecutorConfig {
    pub fn new(trace: bool, cost_counter_config: TransactionCostCounterConfig) -> Self {
        TransactionExecutorConfig {
            trace,
            cost_counter_config,
        }
    }

    pub fn default(trace: bool) -> Self {
        Self::new(
            trace,
            TransactionCostCounterConfig::SystemLoanAndMaxCost {
                system_loan_amount: DEFAULT_SYSTEM_LOAN_AMOUNT,
                max_transaction_cost: DEFAULT_MAX_TRANSACTION_COST,
            },
        )
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
    cost_unit_counter: Box<dyn CostUnitCounter>,
    phantom: PhantomData<I>,
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
        // Metering
        let cost_unit_counter: Box<dyn CostUnitCounter> = match config.cost_counter_config {
            TransactionCostCounterConfig::SystemLoanAndMaxCost {
                system_loan_amount,
                max_transaction_cost,
            } => Box::new(SystemLoanCostUnitCounter::new(
                max_transaction_cost,
                system_loan_amount,
            )),
            TransactionCostCounterConfig::UnlimitedLoanAndMaxCost {
                max_transaction_cost,
            } => Box::new(UnlimitedLoanCostUnitCounter::new(max_transaction_cost)),
        };

        Self {
            substate_store,
            wasm_engine,
            wasm_instrumenter,
            config,
            cost_unit_counter,
            phantom: PhantomData,
        }
    }

    /// Returns an immutable reference to the ledger.
    pub fn substate_store(&self) -> &S {
        self.substate_store
    }

    /// Returns a mutable reference to the ledger.
    pub fn substate_store_mut(&mut self) -> &mut S {
        self.substate_store
    }

    pub fn execute<T: ExecutableTransaction>(&mut self, transaction: &T) -> Receipt {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();

        let transaction_hash = transaction.transaction_hash();
        let transaction_network = transaction.transaction_network();
        let signer_public_keys = transaction.signer_public_keys().to_vec();
        let instructions = transaction.instructions().to_vec();

        // Start state track
        let mut track = Track::new(self.substate_store);

        let mut id_allocator = IdAllocator::new(IdSpace::Application);

        // Metering
        let fee_table = FeeTable::new();
        let cost_unit_counter = self.cost_unit_counter.deref_mut();

        // Charge transaction decoding and stateless verification
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

        // Create root call frame.
        let mut root_frame = CallFrame::new_root(
            self.config.trace,
            transaction_hash,
            signer_public_keys,
            false,
            &mut id_allocator,
            &mut track,
            self.wasm_engine,
            self.wasm_instrumenter,
            cost_unit_counter,
            &fee_table,
        );

        // Invoke the transaction processor
        // TODO: may consider moving transaction parsing to `TransactionProcessor` as well.
        let result = root_frame.invoke_snode(
            scrypto::core::SNodeRef::TransactionProcessor,
            "run".to_string(),
            ScryptoValue::from_typed(&TransactionProcessorRunInput {
                instructions: instructions.clone(),
            }),
        );
        let cost_units_consumed = root_frame.cost_unit_counter().consumed();

        let (outputs, error) = match result {
            Ok(o) => (scrypto_decode::<Vec<Vec<u8>>>(&o.raw).unwrap(), None),
            Err(e) => (Vec::new(), Some(e)),
        };

        let track_receipt = track.to_receipt();

        // commit state updates
        let commit_receipt = if error.is_none() {
            if !track_receipt.borrowed_substates.is_empty() {
                panic!(
                    "Borrowed substates have not been returned {:?}",
                    track_receipt.borrowed_substates
                )
            }

            let commit_receipt = track_receipt.diff.commit(self.substate_store);
            Some(commit_receipt)
        } else {
            None
        };

        let mut new_component_addresses = Vec::new();
        let mut new_resource_addresses = Vec::new();
        let mut new_package_addresses = Vec::new();
        for address in track_receipt.new_addresses {
            match address {
                Address::GlobalComponent(component_address) => {
                    new_component_addresses.push(component_address)
                }
                Address::Resource(resource_address) => {
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

        #[cfg(not(feature = "alloc"))]
        if self.config.trace {
            println!("+{}+", "-".repeat(30));
            for (k, v) in cost_unit_counter.analysis() {
                println!("|{:>20}: {:>8}|", k, v);
            }
            println!("+{}+", "-".repeat(30));
        }

        Receipt {
            transaction_network,
            commit_receipt,
            instructions,
            result: match error {
                Some(error) => Err(error),
                None => Ok(()),
            },
            outputs,
            logs: track_receipt.logs,
            new_package_addresses,
            new_component_addresses,
            new_resource_addresses,
            execution_time,
            cost_units_consumed,
        }
    }
}
