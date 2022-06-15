use sbor::rust::marker::PhantomData;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::buffer::*;
use scrypto::values::ScryptoValue;
use transaction::model::*;

use crate::engine::*;
use crate::fee::CostUnitCounter;
use crate::fee::FeeTable;
use crate::fee::MAX_TRANSACTION_COST;
use crate::fee::SYSTEM_LOAN_AMOUNT;
use crate::ledger::*;
use crate::model::*;
use crate::wasm::*;

/// An executor that runs transactions.
pub struct TransactionExecutor<'s, 'w, S, W, I>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    substate_store: &'s mut S,
    wasm_engine: &'w mut W,
    trace: bool,
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
        trace: bool,
    ) -> TransactionExecutor<'s, 'w, S, W, I> {
        Self {
            substate_store,
            wasm_engine,
            trace,
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
        let signer_public_keys = transaction.signer_public_keys().to_vec();
        let instructions = transaction.instructions().to_vec();

        // Start state track
        let mut track = Track::new(self.substate_store, transaction_hash);

        // Metering
        let cost_unit_counter = CostUnitCounter::new(MAX_TRANSACTION_COST, SYSTEM_LOAN_AMOUNT);
        let fee_table = FeeTable::new();

        // Create root call frame.
        let mut root_frame = CallFrame::new_root(
            self.trace,
            transaction_hash,
            signer_public_keys,
            &mut track,
            self.wasm_engine,
            cost_unit_counter,
            fee_table,
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
            if !track_receipt.borrowed.is_empty() {
                panic!("There should be nothing borrowed by end of transaction.");
            }
            let commit_receipt = track_receipt.substates.commit(self.substate_store);
            Some(commit_receipt)
        } else {
            None
        };

        let mut new_component_addresses = Vec::new();
        let mut new_resource_addresses = Vec::new();
        let mut new_package_addresses = Vec::new();
        for address in track_receipt.new_addresses {
            match address {
                Address::Component(component_address) => {
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

        Receipt {
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
