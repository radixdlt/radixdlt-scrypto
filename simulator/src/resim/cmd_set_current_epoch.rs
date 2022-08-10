use clap::Parser;
use radix_engine::constants::*;
use radix_engine::engine::Track;
use radix_engine::engine::{CallFrame, SystemApi};
use radix_engine::fee::{FeeTable, SystemLoanFeeReserve};
use scrypto::core::{
    FnIdentifier, NativeFnIdentifier, Receiver, SystemFnIdentifier, SystemSetEpochInput,
};
use scrypto::crypto::hash;
use scrypto::engine::types::RENodeId;
use scrypto::values::ScryptoValue;
use transaction::validation::{IdAllocator, IdSpace};

use crate::resim::*;

/// Set the current epoch
#[derive(Parser, Debug)]
pub struct SetCurrentEpoch {
    /// The new epoch number
    epoch: u64,
}

impl SetCurrentEpoch {
    pub fn run<O: std::io::Write>(&self, _out: &mut O) -> Result<(), Error> {
        // TODO: can we construct a proper transaction to do the following?

        let tx_hash = hash(get_nonce()?.to_string());
        let mut substate_store = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut wasm_engine = DefaultWasmEngine::new();
        let mut wasm_instrumenter = WasmInstrumenter::new();
        let mut id_allocator = IdAllocator::new(IdSpace::Application);
        let mut track = Track::new(&substate_store);
        let mut fee_reserve = SystemLoanFeeReserve::default();
        let fee_table = FeeTable::new();

        // Create root call frame.
        let mut root_frame = CallFrame::new_root(
            false,
            tx_hash,
            vec![],
            true,
            DEFAULT_MAX_CALL_DEPTH,
            &mut id_allocator,
            &mut track,
            &mut wasm_engine,
            &mut wasm_instrumenter,
            &mut fee_reserve,
            &fee_table,
        );

        // Invoke the system
        root_frame
            .invoke_method(
                Receiver::Ref(RENodeId::System),
                FnIdentifier::Native(NativeFnIdentifier::System(SystemFnIdentifier::SetEpoch)),
                ScryptoValue::from_typed(&SystemSetEpochInput { epoch: self.epoch }),
            )
            .map(|_| ())
            .map_err(Error::TransactionExecutionError)?;

        // Commit
        track.commit();
        let receipt = track.to_receipt();
        receipt.state_updates.commit(&mut substate_store);

        Ok(())
    }
}
