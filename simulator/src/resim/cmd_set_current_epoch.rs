use clap::Parser;
use radix_engine::types::*;

use crate::resim::*;

/// Set the current epoch
#[derive(Parser, Debug)]
pub struct SetCurrentEpoch {
    /// The new epoch number
    pub epoch_number: u64,
}

impl SetCurrentEpoch {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        db_upsert_epoch(Epoch::of(self.epoch_number))?;
        writeln!(out, "Epoch set successfully").map_err(Error::IOError)?;
        Ok(())
    }
}
