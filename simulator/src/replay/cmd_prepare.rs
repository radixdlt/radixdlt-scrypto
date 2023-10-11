use crate::replay::Error;
use clap::Parser;

/// Prepare transactions from a fully synced database
#[derive(Parser, Debug)]
pub struct Prepare {}

impl Prepare {
    pub fn run(&self) -> Result<(), Error> {
        Ok(())
    }
}
