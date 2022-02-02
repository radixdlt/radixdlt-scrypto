use clap::Parser;

use crate::resim::*;

/// Set the current epoch
#[derive(Parser, Debug)]
pub struct SetCurrentEpoch {
    /// The new epoch number
    epoch: u64,
}

impl SetCurrentEpoch {
    pub fn run(&self) -> Result<(), Error> {
        let mut configs = get_configs()?;
        configs.current_epoch = self.epoch;
        set_configs(configs)?;

        println!("Current epoch set!");
        Ok(())
    }
}
