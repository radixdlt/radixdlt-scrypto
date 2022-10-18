use clap::Parser;
use radix_engine::constants::*;
use radix_engine::engine::Track;
use radix_engine::engine::{Kernel, SystemApi};
use radix_engine::fee::{FeeTable, SystemLoanFeeReserve};
use radix_engine::types::*;
use radix_engine_stores::rocks_db::RadixEngineDB;
use scrypto::core::{FnIdent, MethodIdent, ReceiverMethodIdent};
use transaction::model::{AuthModule, AuthZoneParams};

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
        let blobs = HashMap::new();
        let mut substate_store = RadixEngineDB::with_bootstrap(get_data_dir()?);

        let mut scrypto_interpreter = ScryptoInterpreter {
            wasm_engine: DefaultWasmEngine::new(),
            wasm_instrumenter: WasmInstrumenter::new(),
            wasm_metering_params: WasmMeteringParams::new(
                InstructionCostRules::tiered(1, 5, 10, 5000),
                512,
            ),
            phantom: PhantomData,
        };
        let mut track = Track::new(
            &substate_store,
            SystemLoanFeeReserve::default(),
            FeeTable::new(),
        );

        let mut kernel = Kernel::new(
            tx_hash,
            AuthZoneParams {
                initial_proofs: vec![AuthModule::validator_role_nf_address()],
                virtualizable_proofs_resource_addresses: BTreeSet::new(),
            },
            &blobs,
            DEFAULT_MAX_CALL_DEPTH,
            &mut track,
            &mut scrypto_interpreter,
            Vec::new(),
        );

        // Invoke the system
        kernel
            .invoke(
                FnIdent::Method(ReceiverMethodIdent {
                    receiver: Receiver::Ref(RENodeId::Global(GlobalAddress::Component(
                        SYS_SYSTEM_COMPONENT,
                    ))),
                    method_ident: MethodIdent::Native(NativeMethod::System(SystemMethod::SetEpoch)),
                }),
                ScryptoValue::from_typed(&SystemSetEpochInput { epoch: self.epoch }),
            )
            .map(|_| ())
            .map_err(Error::TransactionExecutionError)?;

        // Commit
        let receipt = track.finalize(Ok(Vec::new()));
        if let TransactionResult::Commit(c) = receipt.result {
            c.state_updates.commit(&mut substate_store);
        }

        Ok(())
    }
}
