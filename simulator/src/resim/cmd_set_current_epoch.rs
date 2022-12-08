use clap::Parser;
use radix_engine::types::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::constants::AuthAddresses;
use radix_engine_interface::data::*;
use transaction::model::SystemInstruction;

use crate::resim::*;

/// Set the current epoch
#[derive(Parser, Debug)]
pub struct SetCurrentEpoch {
    /// The new epoch number
    epoch: u64,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl SetCurrentEpoch {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let instructions = vec![SystemInstruction::CallNativeMethod {
            method_ident: NativeMethodIdent {
                receiver: RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)),
                method_name: "set_epoch".to_string(),
            },
            args: args!(EPOCH_MANAGER, self.epoch),
        }
        .into()];
        let blobs = vec![];
        let initial_proofs = vec![AuthAddresses::validator_role()];
        handle_system_transaction(instructions, blobs, initial_proofs, self.trace, true, out)
            .map(|_| ())
    }
}
