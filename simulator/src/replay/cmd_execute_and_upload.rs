use crate::replay::ledger_transaction::LedgerTransaction;

use super::ledger_transaction::PreparedLedgerTransaction;
use super::ledger_transaction_execution::execute_prepared_ledger_transaction;
use super::ledger_transaction_execution::prepare_ledger_transaction;
use super::ledger_transaction_execution::LedgerTransactionReceipt;
use super::txn_reader::TxnReader;
use super::Error;
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_config::profile::ProfileFileRegionProvider;
use clap::Parser;
use flate2::read::GzDecoder;
use flume;
use futures::future::join_all;
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::ScryptoVm;
use radix_engine_interface::prelude::NetworkDefinition;
use radix_engine_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_engine_store_interface::interface::CommittableSubstateDatabase;
use radix_engine_stores::rocks_db_with_merkle_tree::RocksDBWithMerkleTreeSubstateStore;
use std::fs::File;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tar::Archive;

/// Run transactions in archive, using RocksDB
#[derive(Parser, Debug)]
pub struct TxnExecuteAndUpload {
    /// The transaction file, in `.tar.gz` format, with entries sorted
    pub source: PathBuf,
    /// Path to a folder for storing state
    pub database_dir: PathBuf,

    /// The network to use, [mainnet | stokenet]
    #[clap(short, long)]
    pub network: Option<String>,
    /// The max version to execute
    #[clap(short, long)]
    pub max_version: Option<u64>,

    /// Trace transaction execution
    #[clap(long)]
    pub trace: bool,
}

impl TxnExecuteAndUpload {
    pub fn run(&self) -> Result<(), Error> {
        let network = match &self.network {
            Some(n) => NetworkDefinition::from_str(n).map_err(Error::ParseNetworkError)?,
            None => NetworkDefinition::mainnet(),
        };

        let cur_version = {
            let database = RocksDBWithMerkleTreeSubstateStore::standard(self.database_dir.clone());
            let cur_version = database.get_current_version();
            if cur_version >= self.max_version.unwrap_or(u64::MAX) {
                return Ok(());
            }
            cur_version
        };
        let to_version = self.max_version.clone();

        let start = std::time::Instant::now();
        let (tx_sender, tx_receiver) = flume::bounded(10);
        let (receipt_sender, receipt_receiver) = flume::bounded(10);

        // txn reader
        let mut txn_reader = if self.source.is_file() {
            let tar_gz = File::open(&self.source).map_err(Error::IOError)?;
            let tar = GzDecoder::new(tar_gz);
            let archive = Archive::new(tar);
            TxnReader::TransactionFile(archive)
        } else if self.source.is_dir() {
            TxnReader::StateManagerDatabaseDir(self.source.clone())
        } else {
            return Err(Error::InvalidTransactionSource);
        };
        let txn_read_thread_handle = thread::spawn(move || {
            println!("Reader thread start!");
            txn_reader.read(cur_version, to_version, tx_sender).unwrap();
        });

        // txn executor
        let mut database = RocksDBWithMerkleTreeSubstateStore::standard(self.database_dir.clone());
        let trace = self.trace;
        let txn_write_thread_handle = thread::spawn(move || {
            println!("Executor thread start!");
            let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
            let iter = tx_receiver.iter();
            for tx_payload in iter {
                let tx_prepared = prepare_ledger_transaction(&tx_payload);
                let receipt = execute_prepared_ledger_transaction(
                    &database,
                    &scrypto_vm,
                    &network,
                    &tx_prepared,
                    trace,
                );
                // TODO: better handling of error to support breakpoint
                receipt_sender
                    .send((tx_payload, tx_prepared, receipt.clone()))
                    .unwrap();
                let state_updates = receipt.into_state_updates();
                let database_updates =
                    state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
                database.commit(&database_updates);

                let new_state_root_hash = database.get_current_root_hash();
                let new_version = database.get_current_version();

                if new_version < 1000 || new_version % 1000 == 0 {
                    print_progress(start.elapsed(), new_version, new_state_root_hash);
                }
            }

            let duration = start.elapsed();
            println!("Time elapsed: {:?}", duration);
            println!("State version: {}", database.get_current_version());
            println!("State root hash: {}", database.get_current_root_hash());
        });

        // receipt uploader
        let txn_upload_thread_handle = thread::spawn(move || {
            println!("Uploader thread start!");

            async fn set_up_s3_client() -> aws_sdk_s3::Client {
                let config = aws_config::from_env()
                    .region(
                        ProfileFileRegionProvider::builder()
                            .profile_name("sandbox-cli")
                            .build(),
                    )
                    .credentials_provider(
                        ProfileFileCredentialsProvider::builder()
                            .profile_name("sandbox-cli")
                            .build(),
                    )
                    .load()
                    .await;
                aws_sdk_s3::Client::new(&config)
            }

            async fn upload_transaction(
                client: &aws_sdk_s3::Client,
                payload: Vec<u8>,
                prepared: PreparedLedgerTransaction,
                receipt: LedgerTransactionReceipt,
            ) {
                use sbor::representations::*;

                let hash = prepared.summary.hash.to_string();

                // transaction
                let tx = LedgerTransaction::from_payload_bytes(&payload).unwrap();
                let tx_sbor = payload;
                let tx_json = serde_json::to_string(&tx).unwrap();

                // receipt
                let (type_id, schema) = generate_full_schema_from_single_type::<
                    LedgerTransactionReceipt,
                    ScryptoCustomSchema,
                >();
                let receipt_sbor = scrypto_encode(&receipt).unwrap();
                let receipt_slice =
                    ScryptoRawPayload::new_from_valid_slice_with_checks(&receipt_sbor).unwrap();
                let receipt_serializable =
                    receipt_slice.serializable(SerializationParameters::WithSchema {
                        mode: SerializationMode::Programmatic,
                        schema: schema.v1(),
                        custom_context: ScryptoValueDisplayContext::no_context(),
                        type_id,
                        depth_limit: SCRYPTO_SBOR_V1_MAX_DEPTH,
                    });
                let receipt_json = serde_json::to_string(&receipt_serializable).unwrap();

                join_all(vec![
                    client
                        .put_object()
                        .bucket("yulongtest")
                        .key(format!("transaction-sbor/{hash}"))
                        .body(tx_sbor.into())
                        .send(),
                    client
                        .put_object()
                        .bucket("yulongtest")
                        .key(format!("transaction-json/{hash}"))
                        .body(tx_json.into_bytes().into())
                        .send(),
                    client
                        .put_object()
                        .bucket("yulongtest")
                        .key(format!("receipt-sbor/{hash}"))
                        .body(receipt_sbor.into())
                        .send(),
                    client
                        .put_object()
                        .bucket("yulongtest")
                        .key(format!("receipt-json/{hash}"))
                        .body(receipt_json.into_bytes().into())
                        .send(),
                ])
                .await;
            }

            let client = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(set_up_s3_client());

            // TODO: multi-threading
            let iter = receipt_receiver.iter();
            for (tx_payload, tx_prepared, receipt) in iter {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(upload_transaction(
                        &client,
                        tx_payload,
                        tx_prepared,
                        receipt,
                    ));
            }
        });

        txn_read_thread_handle.join().unwrap();
        txn_write_thread_handle.join().unwrap();
        txn_upload_thread_handle.join().unwrap();

        Ok(())
    }
}

fn print_progress(duration: Duration, new_version: u64, new_root: Hash) {
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = (duration.as_secs() / 60) / 60;
    println!(
        "New version: {}, {}, {:0>2}:{:0>2}:{:0>2}",
        new_version, new_root, hours, minutes, seconds
    );
}
