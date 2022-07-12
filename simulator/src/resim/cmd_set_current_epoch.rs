use clap::Parser;
use scrypto::prelude::SYSTEM_PACKAGE;
use scrypto::to_struct;

use crate::resim::*;

/// Set the current epoch
#[derive(Parser, Debug)]
pub struct SetCurrentEpoch {
    /// The new epoch number
    epoch: u64,
}

impl SetCurrentEpoch {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let manifest = ManifestBuilder::new()
            .call_function(
                SYSTEM_PACKAGE,
                "System",
                "set_epoch",
                to_struct!(self.epoch)
            )
            .build();
        handle_manifest(
            manifest,
            &Option::None,
            &Option::None,
            false,
            false,
            out,
        )
        .map(|_| ())
    }
}
