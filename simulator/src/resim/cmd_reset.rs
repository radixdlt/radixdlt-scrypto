use clap::Parser;
use std::fs::remove_dir_all;

use crate::resim::*;

/// Reset this simulator
#[derive(Parser, Debug)]
pub struct Reset {}

impl Reset {
    pub fn run(&self) -> Result<(), Error> {
        let dir = get_data_dir()?;
        remove_dir_all(dir).map_err(Error::IOError)?;
        println!("Data directory cleared.");
        Ok(())
    }
}
