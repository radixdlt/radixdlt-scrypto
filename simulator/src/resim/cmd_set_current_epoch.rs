use clap::Parser;
use radix_engine::types::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;

use crate::resim::*;

/// Set the current epoch
#[derive(Parser, Debug)]
pub struct SetCurrentEpoch {
    /// The new epoch number
    epoch: u64,

    /// The network to use when outputting manifest, [simulator | adapanet | nebunet | mainnet]
    #[clap(short, long)]
    network: Option<String>,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl SetCurrentEpoch {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(FAUCET_COMPONENT, 100.into())
            .call_native_method(
                RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)),
                "set_epoch",
                args!(self.epoch),
            )
            .build();
        handle_manifest(
            manifest,
            &self.signing_keys,
            &self.network,
            &self.manifest,
            self.trace,
            true,
            true,
            out,
        )
        .map(|_| ())
    }
}
