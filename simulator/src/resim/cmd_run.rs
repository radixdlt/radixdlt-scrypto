use clap::Parser;
use radix_engine::model::*;
use scrypto::crypto::*;
use std::path::PathBuf;

use crate::resim::*;

/// Compiles, signs and runs a transaction manifest
#[derive(Parser, Debug)]
pub struct Run {
    /// The path to a transaction manifest file
    path: PathBuf,

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    signing_keys: Option<Vec<String>>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Run {
    pub fn run(&self) -> Result<(), Error> {
        let private_keys = if let Some(keys) = &self.signing_keys {
            keys.iter()
                .map(|key| {
                    hex::decode(key)
                        .map_err(|_| Error::InvalidPrivateKey)
                        .and_then(|bytes| {
                            EcdsaPrivateKey::from_bytes(&bytes)
                                .map_err(|_| Error::InvalidPrivateKey)
                        })
                })
                .collect::<Result<Vec<EcdsaPrivateKey>, Error>>()?
        } else {
            vec![get_default_signers()?.1]
        };

        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let manifest = std::fs::read_to_string(&self.path).map_err(Error::IOError)?;
        let mut unsigned = transaction_manifest::compile(&manifest).map_err(Error::CompileError)?;
        unsigned.instructions.push(Instruction::Nonce {
            nonce: executor.substate_store().get_nonce(),
        });
        let signed = unsigned.sign(private_keys.iter().collect::<Vec<&EcdsaPrivateKey>>());
        process_transaction(signed, &mut executor, &None)
    }
}
