use clap::Parser;
use radix_engine::ledger::Ledger;

use crate::resim::*;

/// Set the current epoch
#[derive(Parser, Debug)]
pub struct SetCurrentEpoch {
    /// The new epoch number
    epoch: u64,
}

impl SetCurrentEpoch {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
        ledger.set_epoch(self.epoch);

        println!("Current epoch set!");
        Ok(())
    }
}
