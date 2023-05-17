use clap::Parser;
use colored::*;
use radix_engine_interface::blueprints::clock::*;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::time::Instant;
use radix_engine_interface::time::UtcDateTime;
use radix_engine_stores::rocks_db::RocksdbSubstateStore;
use transaction::model::Instruction;

use crate::resim::*;

/// Show entries in the ledger state
#[derive(Parser, Debug)]
pub struct ShowLedger {}

impl ShowLedger {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let scrypto_interpreter = ScryptoVm::<DefaultWasmEngine>::default();
        let mut substate_db = RocksdbSubstateStore::standard(get_data_dir()?);
        Bootstrapper::new(&mut substate_db, &scrypto_interpreter, false).bootstrap_test_default();

        // Close the database
        drop(substate_db);

        let current_epoch = Self::get_current_epoch(out)?;
        writeln!(out, "{}: {}", "Current Epoch".green().bold(), current_epoch)
            .map_err(Error::IOError)?;

        let instant = Self::get_current_time(out, TimePrecision::Minute)?;
        let date_time = UtcDateTime::from_instant(&instant).unwrap();
        writeln!(
            out,
            "{}: {}",
            "Current Time".green().bold(),
            date_time.to_string()
        )
        .map_err(Error::IOError)?;

        writeln!(out, "").map_err(Error::IOError)?;

        Ok(())
    }

    pub fn get_current_epoch<O: std::io::Write>(out: &mut O) -> Result<u64, Error> {
        let instructions = vec![Instruction::CallMethod { 
            address: EPOCH_MANAGER.into(),
            method_name: EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT.to_string(),
            args: to_manifest_value(&EpochManagerGetCurrentEpochInput),
        }];
        let blobs = vec![];
        let initial_proofs = btreeset![];
        let receipt =
            handle_system_transaction(instructions, blobs, initial_proofs, false, false, out)?;
        Ok(receipt.expect_commit(true).output(0))
    }

    pub fn get_current_time<O: std::io::Write>(
        out: &mut O,
        precision: TimePrecision,
    ) -> Result<Instant, Error> {
        let instructions = vec![Instruction::CallMethod { 
            address: CLOCK.into(),
            method_name: CLOCK_GET_CURRENT_TIME_IDENT.to_string(),
            args: to_manifest_value(&ClockGetCurrentTimeInput { precision }),
        }];
        let blobs = vec![];
        let initial_proofs = btreeset![];
        let receipt =
            handle_system_transaction(instructions, blobs, initial_proofs, false, false, out)?;
        Ok(receipt.expect_commit(true).output(0))
    }
}
