use clap::Parser;
use std::fs::remove_dir_all;

use crate::resim::*;

/// Reset this simulator
#[derive(Parser, Debug)]
pub struct Reset {}

impl Reset {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let dir = get_data_dir()?;
        remove_dir_all(dir).map_err(Error::IOError)?;
        writeln!(out, "Data directory cleared.").map_err(Error::IOError)?;
        Ok(())
    }
}
