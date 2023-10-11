use crate::replay::Error;
use clap::Parser;
use flate2::write::GzEncoder;
use flate2::Compression;
use rocksdb::Direction;
use rocksdb::IteratorMode;
use rocksdb::Options;
use rocksdb::DB;
use std::fs::File;
use std::path::PathBuf;
use tar::Header;

/// Prepare transactions from a fully synced database
#[derive(Parser, Debug)]
pub struct Prepare {
    /// Path to the `state_manager` database
    pub database_dir: PathBuf,
    /// Path to the output transaction file, in `.tar.gz` format, with entries sorted
    pub transaction_file: PathBuf,
    /// The max number of transactions to export
    pub limit: Option<u32>,
}

impl Prepare {
    pub fn run(&self) -> Result<(), Error> {
        let temp_dir = tempfile::tempdir().map_err(Error::IOError)?;
        let db = DB::open_cf_as_secondary(
            &Options::default(),
            self.database_dir.as_path(),
            temp_dir.path(),
            vec!["raw_ledger_transactions"],
        )
        .map_err(Error::DatabaseError)?;
        db.try_catch_up_with_primary()
            .expect("DB catch up with primary failed");

        let tar_gz = File::create(&self.transaction_file).map_err(Error::IOError)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);

        let mut version = 1u64;
        let mut txn_iter = db.iterator_cf(
            &db.cf_handle("raw_ledger_transactions").unwrap(),
            IteratorMode::From(&version.to_be_bytes(), Direction::Forward),
        );
        let mut count = 0;
        while let Some(next_txn) = txn_iter.next() {
            let data = next_txn.unwrap().1.to_vec();

            // write to tar file
            let mut header = Header::new_gnu();
            header.set_path(format!("{:0>9}", version)).unwrap();
            header.set_size(data.len().try_into().unwrap());
            header.set_cksum();
            tar.append(&header, data.as_slice()).unwrap();

            // check limit
            count += 1;
            if count % 100 == 0 {
                println!("{}", count);
            }
            if count >= self.limit.unwrap_or(u32::MAX) {
                break;
            }

            version += 1;
        }

        std::fs::remove_dir_all(temp_dir.path()).map_err(Error::IOError)?;

        Ok(())
    }
}
