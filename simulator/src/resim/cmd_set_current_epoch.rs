use clap::Parser;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerSetEpochInput, CONSENSUS_MANAGER_SET_EPOCH_IDENT,
};
use transaction::model::InstructionV1;

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
        let instructions = vec![InstructionV1::CallMethod {
            address: CONSENSUS_MANAGER.into(),
            method_name: CONSENSUS_MANAGER_SET_EPOCH_IDENT.to_string(),
            args: to_manifest_value(&ConsensusManagerSetEpochInput { epoch: self.epoch }),
        }];

        let blobs = vec![];
        let initial_proofs = btreeset![AuthAddresses::system_role()];
        handle_system_transaction(instructions, blobs, initial_proofs, self.trace, true, out)
            .map(|_| ())
    }
}
