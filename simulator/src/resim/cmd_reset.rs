use clap::Parser;

use crate::resim::*;

/// Reset this simulator
#[derive(Parser, Debug)]
pub struct Reset {}

impl Reset {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        SimulatorEnvironment::new().and_then(|mut env| env.reset())?;
        writeln!(out, "Data directory cleared.").map_err(Error::IOError)?;
        Ok(())
    }
}
