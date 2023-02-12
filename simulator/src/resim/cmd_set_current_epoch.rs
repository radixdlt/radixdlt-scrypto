use clap::Parser;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::epoch_manager::{EPOCH_MANAGER_SET_EPOCH_IDENT, EpochManagerSetEpochInput};
use transaction::model::BasicInstruction;

use crate::resim::*;

/// Set the current epoch
#[derive(Parser, Debug)]
pub struct SetCurrentEpoch {
    /// The new epoch number
    pub epoch: u64,

    /// Turn on tracing
    #[clap(short, long)]
    pub trace: bool,
}

impl SetCurrentEpoch {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let instructions = vec![Instruction::Basic(BasicInstruction::CallMethod {
            component_address: EPOCH_MANAGER,
            method_name: EPOCH_MANAGER_SET_EPOCH_IDENT.to_string(),
            args: scrypto_encode(&EpochManagerSetEpochInput {
                epoch: self.epoch,
            }).unwrap(),
        })];

        let blobs = vec![];
        let initial_proofs = vec![AuthAddresses::system_role()];
        handle_system_transaction(instructions, blobs, initial_proofs, self.trace, true, out)
            .map(|_| ())
    }
}
