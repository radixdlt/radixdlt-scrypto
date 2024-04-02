use super::Error;
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

/// Prepare transactions from a fully synced Node database
#[derive(Parser, Debug)]
pub struct TxnPrepare {
    /// Path to the `state_manager` database
    pub database_dir: PathBuf,
    /// Path to the output transaction file, in `.tar.gz` format, with entries sorted
    pub transaction_file: PathBuf,

    /// The max version to export
    #[clap(short, long)]
    pub max_version: Option<u64>,
}

const TRANSACTION_COLUMN: &str = "raw_ledger_transactions";

impl TxnPrepare {
    pub fn run(&self) -> Result<(), String> {
        let temp_dir = tempfile::tempdir().map_err(Error::IOError)?;
        let db = DB::open_cf_as_secondary(
            &Options::default(),
            self.database_dir.as_path(),
            temp_dir.path(),
            vec![TRANSACTION_COLUMN],
        )
        .map_err(Error::DatabaseError)?;
        db.try_catch_up_with_primary()
            .map_err(Error::DatabaseError)?;

        let tar_gz = File::create(&self.transaction_file).map_err(Error::IOError)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);

        let mut txn_iter = db.iterator_cf(
            &db.cf_handle(TRANSACTION_COLUMN).unwrap(),
            IteratorMode::From(&[], Direction::Forward),
        );
        let mut last_version = 0u64;
        while let Some(txn) = txn_iter.next() {
            // read transaction
            let txn = txn.map_err(Error::DatabaseError)?;
            let version = u64::from_be_bytes(txn.0.to_vec().try_into().unwrap());
            let payload = txn.1.to_vec();

            // check limit
            if version > self.max_version.unwrap_or(u64::MAX) {
                break;
            }

            // write to the tar file
            let mut header = Header::new_gnu();
            header.set_path(format!("{:0>9}", version)).unwrap();
            header.set_size(payload.len().try_into().unwrap());
            header.set_cksum();
            tar.append(&header, payload.as_slice()).unwrap();

            // print progress
            if version < 1000 || version % 1000 == 0 {
                println!("New version: {}", version);
            }
            last_version = version;
        }
        println!("Last version: {}", last_version);

        std::fs::remove_dir_all(temp_dir.path()).map_err(Error::IOError)?;
        Ok(())
    }
}
