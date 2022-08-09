use sbor::rust::marker::PhantomData;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::buffer::*;
use scrypto::engine::types::SubstateId;
use scrypto::math::Decimal;
use scrypto::prelude::TypeName;
use scrypto::prelude::RADIX_TOKEN;
use scrypto::resource::ResourceType;
use scrypto::values::ScryptoValue;
use transaction::model::*;
use transaction::validation::{IdAllocator, IdSpace};

use crate::constants::{DEFAULT_COST_UNIT_PRICE, DEFAULT_MAX_CALL_DEPTH, DEFAULT_SYSTEM_LOAN};
use crate::engine::Track;
use crate::engine::*;
use crate::fee::{FeeReserve, FeeTable, SystemLoanFeeReserve};
use crate::ledger::{ReadableSubstateStore, WriteableSubstateStore};
use crate::model::*;
use crate::transaction::*;
use crate::wasm::*;

pub struct ExecutionConfig {
    pub cost_unit_price: Decimal,
    pub max_call_depth: usize,
    pub system_loan: u32,
    pub is_system: bool,
    pub trace: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            cost_unit_price: DEFAULT_COST_UNIT_PRICE.parse().unwrap(),
            max_call_depth: DEFAULT_MAX_CALL_DEPTH,
            system_loan: DEFAULT_SYSTEM_LOAN,
            is_system: false,
            trace: false,
        }
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
    ) -> Self {
        Self {
            substate_store,
            wasm_engine,
            wasm_instrumenter,
            phantom: PhantomData,
        }
    }

    pub fn execute_and_commit<T: ExecutableTransaction>(
        &mut self,
        transaction: &T,
        params: &ExecutionConfig,
    ) -> TransactionReceipt {
        let receipt = self.execute(transaction, params);
        receipt.state_updates.commit(self.substate_store);
        receipt
    }

    pub fn execute<T: ExecutableTransaction>(
        &mut self,
        transaction: &T,
        params: &ExecutionConfig,
    ) -> TransactionReceipt {
        let fee_reserve = SystemLoanFeeReserve::new(
            transaction.cost_unit_limit(),
            transaction.tip_percentage(),
            params.cost_unit_price,
            params.system_loan,
        );

        self.execute_with_fee_reserve(transaction, params, fee_reserve)
    }

    pub fn execute_with_fee_reserve<T: ExecutableTransaction, C: FeeReserve>(
        &mut self,
        transaction: &T,
        params: &ExecutionConfig,
        mut fee_reserve: C,
    ) -> TransactionReceipt {
        #[cfg(not(feature = "alloc"))]
        let now = std::time::Instant::now();

        let transaction_hash = transaction.transaction_hash();
        let transaction_network = transaction.transaction_network();
        let signer_public_keys = transaction.signer_public_keys().to_vec();
        let instructions = transaction.instructions().to_vec();
        #[cfg(not(feature = "alloc"))]
        if params.trace {
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
        fee_reserve
            .consume(
                fee_table.tx_decoding_per_byte() * transaction.transaction_payload_size() as u32,
                "tx_decoding",
            )
            .expect("System loan should cover this");
        fee_reserve
            .consume(
                fee_table.tx_manifest_verification_per_byte()
                    * transaction.transaction_payload_size() as u32,
                "tx_manifest_verification",
            )
            .expect("System loan should cover this");
        fee_reserve
            .consume(
                fee_table.tx_signature_verification_per_sig()
                    * transaction.signer_public_keys().len() as u32,
                "tx_signature_verification",
            )
            .expect("System loan should cover this");

        // 3. Start a call frame and run the transaction
        let mut root_frame = CallFrame::new_root(
            params.trace,
            transaction_hash,
            signer_public_keys,
            params.is_system,
            params.max_call_depth,
            &mut id_allocator,
            &mut track,
            self.wasm_engine,
            self.wasm_instrumenter,
            &mut fee_reserve,
            &fee_table,
        );
        let result = root_frame
            .invoke_function(
                TypeName::TransactionProcessor,
                "run".to_string(),
                ScryptoValue::from_typed(&TransactionProcessorRunInput {
                    instructions: instructions.clone(),
                }),
            )
            .map(|o| scrypto_decode::<Vec<Vec<u8>>>(&o.raw).unwrap());

        // 4. Settle transaction fee
        let fee_summary = fee_reserve.finalize();
        #[cfg(not(feature = "alloc"))]
        if params.trace {
            println!("{:-^80}", "Cost Analysis");
            for (k, v) in &fee_summary.cost_breakdown {
                println!("{:<30}: {:>8}", k, v);
            }
        }

        let status = if fee_summary.loan_fully_repaid {
            match result {
                Ok(output) => TransactionStatus::Succeeded(output),
                Err(error) => TransactionStatus::Failed(error),
            }
        } else {
            TransactionStatus::Rejected
        };

        if status.is_success() {
            track.commit();
        } else {
            track.rollback();
        }

        let mut required = fee_summary.burned + fee_summary.tipped;
        let mut collector =
            ResourceContainer::new_empty(RADIX_TOKEN, ResourceType::Fungible { divisibility: 18 });
        for (vault_id, mut locked, contingent) in fee_summary.payments.iter().cloned().rev() {
            let amount = if contingent {
                if status.is_success() {
                    Decimal::min(locked.liquid_amount(), required)
                } else {
                    Decimal::zero()
                }
            } else {
                Decimal::min(locked.liquid_amount(), required)
            };

            // Deduct fee required
            required = required - amount;

            // Collect fees into collector
            collector
                .put(locked.take_by_amount(amount).unwrap())
                .unwrap();

            // Refund overpayment
            let substate_id = SubstateId::Vault(vault_id);
            track.acquire_lock(substate_id.clone(), true, true).unwrap();
            let mut substate = track.take_substate(substate_id.clone());
            substate.vault_mut().put(Bucket::new(locked)).unwrap();
            track.write_substate(substate_id.clone(), substate);
            track.release_lock(substate_id, true);
        }
        // TODO: update XRD supply or disable it
        // TODO: pay tips to the lead validator

        // 5. Produce the final transaction receipt
        let track_receipt = track.to_receipt();

        let mut new_component_addresses = Vec::new();
        let mut new_resource_addresses = Vec::new();
        let mut new_package_addresses = Vec::new();
        for address in track_receipt.new_addresses {
            match address {
                SubstateId::ComponentInfo(component_address) => {
                    new_component_addresses.push(component_address)
                }
                SubstateId::ResourceManager(resource_address) => {
                    new_resource_addresses.push(resource_address)
                }
                SubstateId::Package(package_address) => new_package_addresses.push(package_address),
                _ => {}
            }
        }

        #[cfg(feature = "alloc")]
        let execution_time = None;
        #[cfg(not(feature = "alloc"))]
        let execution_time = Some(now.elapsed().as_millis());

        let receipt = TransactionReceipt {
            status,
            transaction_network,
            fee_summary: fee_summary,
            instructions,
            application_logs: track_receipt.application_logs,
            new_package_addresses,
            new_component_addresses,
            new_resource_addresses,
            execution_time,
            state_updates: track_receipt.state_updates,
        };
        #[cfg(not(feature = "alloc"))]
        if params.trace {
            println!("{:-^80}", "Transaction Receipt");
            println!("{:?}", receipt);
            println!("{:-^80}", "");
        }

        receipt
    }
}
