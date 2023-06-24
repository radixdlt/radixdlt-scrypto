use crate::utils::*;
use clap::Parser;
use colored::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::time::Instant;
use radix_engine_interface::time::UtcDateTime;
use radix_engine_store_interface::{
    db_key_mapper::{DatabaseKeyMapper, SpreadPrefixKeyMapper},
    interface::ListableSubstateDatabase,
};
use radix_engine_stores::rocks_db::RocksdbSubstateStore;
use transaction::model::InstructionV1;

use crate::resim::*;

/// Show entries in the ledger state
#[derive(Parser, Debug)]
pub struct ShowLedger {}

impl ShowLedger {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let scrypto_interpreter = ScryptoVm::<DefaultWasmEngine>::default();
        let mut substate_db = RocksdbSubstateStore::standard(get_data_dir()?);
        Bootstrapper::new(&mut substate_db, &scrypto_interpreter, false).bootstrap_test_default();

        Self::list_entries(out, &substate_db)?;

        // Close the database
        drop(substate_db);

        let current_epoch = Self::get_current_epoch(out)?;
        writeln!(
            out,
            "{}: {}",
            "Current Epoch".green().bold(),
            current_epoch.number()
        )
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

    pub fn list_entries<O: std::io::Write>(
        out: &mut O,
        substate_db: &RocksdbSubstateStore,
    ) -> Result<(), Error> {
        let address_bech32_encoder = AddressBech32Encoder::new(&NetworkDefinition::simulator());
        let mut packages: Vec<PackageAddress> = vec![];
        let mut components: Vec<ComponentAddress> = vec![];
        let mut resources: Vec<ResourceAddress> = vec![];

        for key in substate_db.list_partition_keys() {
            let (node_id, _) = SpreadPrefixKeyMapper::from_db_partition_key(&key);
            if let Ok(address) = PackageAddress::try_from(node_id.as_ref()) {
                if !packages.contains(&address) {
                    packages.push(address);
                }
            } else if let Ok(address) = ComponentAddress::try_from(node_id.as_ref()) {
                if !components.contains(&address) {
                    components.push(address);
                }
            } else if let Ok(address) = ResourceAddress::try_from(node_id.as_ref()) {
                if !resources.contains(&address) {
                    resources.push(address);
                }
            }
        }
        writeln!(out, "{}:", "Packages".green().bold()).map_err(Error::IOError)?;
        for (last, address) in packages.iter().identify_last() {
            writeln!(
                out,
                "{} {}",
                list_item_prefix(last),
                address.display(&address_bech32_encoder),
            )
            .map_err(Error::IOError)?;
        }
        writeln!(out, "{}:", "Components".green().bold()).map_err(Error::IOError)?;
        for (last, address) in components.iter().identify_last() {
            writeln!(
                out,
                "{} {}",
                list_item_prefix(last),
                address.display(&address_bech32_encoder),
            )
            .map_err(Error::IOError)?;
        }
        writeln!(out, "{}:", "Resource Managers".green().bold()).map_err(Error::IOError)?;
        for (last, address) in resources.iter().identify_last() {
            writeln!(
                out,
                "{} {}",
                list_item_prefix(last),
                address.display(&address_bech32_encoder),
            )
            .map_err(Error::IOError)?;
        }

        Ok(())
    }

    pub fn get_current_epoch<O: std::io::Write>(out: &mut O) -> Result<Epoch, Error> {
        let instructions = vec![InstructionV1::CallMethod {
            address: CONSENSUS_MANAGER.into(),
            method_name: CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ConsensusManagerGetCurrentEpochInput),
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
        let instructions = vec![InstructionV1::CallMethod {
            address: CONSENSUS_MANAGER.into(),
            method_name: CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ConsensusManagerGetCurrentTimeInput { precision }),
        }];
        let blobs = vec![];
        let initial_proofs = btreeset![];
        let receipt =
            handle_system_transaction(instructions, blobs, initial_proofs, false, false, out)?;
        Ok(receipt.expect_commit(true).output(0))
    }
}
