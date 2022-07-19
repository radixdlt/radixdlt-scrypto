use sbor::rust::marker::PhantomData;
use sbor::rust::rc::Rc;
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
pub struct TransactionExecutor<'w, S, W, I>
where
    S: ReadableSubstateStore + WriteableSubstateStore + 'static,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    substate_store: Option<S>,
    wasm_engine: &'w mut W,
    wasm_instrumenter: &'w mut WasmInstrumenter,
    trace: bool,
    phantom: PhantomData<I>,
}

impl<'w, S, W, I> TransactionExecutor<'w, S, W, I>
where
    S: ReadableSubstateStore + WriteableSubstateStore + 'static,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn new(
        substate_store: S,
        wasm_engine: &'w mut W,
        wasm_instrumenter: &'w mut WasmInstrumenter,
        trace: bool,
    ) -> TransactionExecutor<'w, S, W, I> {
        Self {
            substate_store: Some(substate_store),
            wasm_engine,
            wasm_instrumenter,
            trace,
            phantom: PhantomData,
        }
    }

    /// Returns an immutable reference to the ledger.
    pub fn substate_store(&self) -> &S {
        self.substate_store
            .as_ref()
            .expect("Missing substate store")
    }

    /// Returns a mutable reference to the ledger.
    pub fn substate_store_mut(&mut self) -> &mut S {
        self.substate_store
            .as_mut()
            .expect("Missing substate store")
    }

    pub fn execute<T: ExecutableTransaction>(&mut self, transaction: &T) -> Receipt {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();

        let transaction_hash = transaction.transaction_hash();
        let transaction_network = transaction.transaction_network();
        let signer_public_keys = transaction.signer_public_keys().to_vec();
        let instructions = transaction.instructions().to_vec();

        // 1. Start state track
        let substate_store_rc =
            Rc::new(self.substate_store.take().expect("Missing substate store"));
        let mut track = Track::new(
            substate_store_rc.clone(),
            transaction_hash,
            transaction_network.clone(),
        );

        // 2. Apply pre-execution costing
        let mut cost_unit_counter = CostUnitCounter::new(MAX_TRANSACTION_COST, SYSTEM_LOAN_AMOUNT);
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
            self.trace,
            transaction_hash,
            signer_public_keys,
            false,
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
        if counter.owed() != 0 {
            // TODO: If a transaction finished before the loan check point AND the
            // loan is not fully repaid, we should reject it.
        }
        if counter.balance() > 0 {
            // TODO: refund
        }
        for _i in 0..10 {
            // TODO: burn fee + reward validators
        }
        let total_consumed = counter.consumed();
        let _overpaid = counter.balance();

        // 5. Generate receipts and commit (TODO: split out commit phase)
        let track_receipt = track.to_receipt();
        self.substate_store = match Rc::try_unwrap(substate_store_rc) {
            Ok(store) => Some(store),
            Err(_) => panic!("There should be no other strong refs that prevent unwrapping"),
        };
        let commit_receipt = track_receipt
            .state_changes
            .commit(self.substate_store_mut());

        #[cfg(feature = "alloc")]
        let execution_time = None;
        #[cfg(not(feature = "alloc"))]
        let execution_time = Some(now.elapsed().as_millis());

        #[cfg(not(feature = "alloc"))]
        if self.trace {
            println!("+{}+", "-".repeat(30));
            for (k, v) in cost_unit_counter.analysis {
                println!("|{:>20}: {:>8}|", k, v);
            }
            println!("+{}+", "-".repeat(30));
        }

        // 6. Produce the final transaction receipt
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
        let logs = track_receipt.logs;
        Receipt {
            transaction_network,
            instructions,
            result,
            logs,
            new_package_addresses,
            new_component_addresses,
            new_resource_addresses,
            execution_time,
            cost_units_consumed: total_consumed,
            commit_receipt,
        }

        // TODO: reject transactions not paying enough fees
    }

    pub fn destroy(self) -> S {
        self.substate_store.expect("Missing substate store")
    }
}
