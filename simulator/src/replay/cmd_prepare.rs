use crate::replay::Error;
use clap::Parser;

/// Prepare transactions from a fully synced database
#[derive(Parser, Debug)]
pub struct Prepare {
    /// Path to the `state_manager` database
    pub transaction: String,
    /// The max number of transactions to export
    pub limit: Option<u32>,
}

impl Prepare {
    pub fn run(&self) -> Result<(), Error> {
        Ok(())
    }
}
