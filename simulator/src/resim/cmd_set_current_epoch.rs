use clap::Parser;

use crate::resim::*;

/// Set the current epoch
#[derive(Parser, Debug)]
pub struct SetCurrentEpoch {
    /// The new epoch number
    epoch: u64,
}

impl SetCurrentEpoch {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let mut substate_store = RadixEngineDB::new(get_data_dir()?);
        substate_store.set_epoch(self.epoch);

        writeln!(out, "Current epoch set!").map_err(Error::IOError)?;
        Ok(())
    }
}
