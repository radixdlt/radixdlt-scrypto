use super::Error;
use flate2::read::GzDecoder;
use flume;
use flume::Sender;
use radix_common::prelude::*;
use radix_transactions::model::RawLedgerTransaction;
use rocksdb::{Direction, IteratorMode, Options, DB};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tar::Archive;

pub enum TxnReader {
    TransactionFile(Archive<GzDecoder<File>>),
    StateManagerDatabaseDir(PathBuf),
}

impl TxnReader {
    pub fn read(
        &mut self,
        from_version: u64,
        to_version: Option<u64>,
        tx: Sender<RawLedgerTransaction>,
    ) -> Result<(), Error> {
        match self {
            TxnReader::TransactionFile(archive) => {
                for entry in archive.entries().map_err(Error::IOError)? {
                    // read the entry
                    let mut entry = entry.map_err(|e| Error::IOError(e))?;
                    let tx_version = entry
                        .header()
                        .path()
                        .ok()
                        .and_then(|path| path.to_str().map(ToOwned::to_owned))
                        .and_then(|s| u64::from_str(&s).ok())
                        .ok_or(Error::InvalidTransactionArchive)?;
                    let mut tx_payload = Vec::new();
                    entry
                        .read_to_end(&mut tx_payload)
                        .map_err(|e| Error::IOError(e))?;

                    if tx_version <= from_version {
                        continue;
                    }
                    if let Some(to_version) = to_version {
                        if tx_version > to_version {
                            break;
                        }
                    }

                    tx.send(RawLedgerTransaction::from_vec(tx_payload)).unwrap();
                }
            }
            TxnReader::StateManagerDatabaseDir(db_dir) => {
                let temp_dir = tempfile::tempdir().map_err(Error::IOError)?;

                let db = DB::open_cf_as_secondary(
                    &Options::default(),
                    db_dir.as_path(),
                    temp_dir.as_ref(),
                    vec![
                        "raw_ledger_transactions",
                        "committed_transaction_identifiers",
                    ],
                )
                .unwrap();

                let iter_start_state_version = from_version + 1;

                loop {
                    db.try_catch_up_with_primary()
                        .expect("DB catch up with primary failed");
                    let mut txn_iter = db.iterator_cf(
                        &db.cf_handle("raw_ledger_transactions").unwrap(),
                        IteratorMode::From(
                            &iter_start_state_version.to_be_bytes(),
                            Direction::Forward,
                        ),
                    );
                    while let Some(next_txn) = txn_iter.next() {
                        let next_txn = next_txn.unwrap();
                        tx.send(RawLedgerTransaction::from_vec(next_txn.1.to_vec()))
                            .unwrap();
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }

        Ok(())
    }
}
