use clap::Parser;
use radix_engine::transaction::*;
use scrypto::engine::types::*;

use crate::resim::*;

/// Call a method
#[derive(Parser, Debug)]
pub struct CallMethod {
    /// The component that the method belongs to
    component_address: ComponentAddress,

    /// The method name
    method_name: String,

    /// The call arguments
    arguments: Vec<String>,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl CallMethod {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let default_account = get_default_account()?;
        let (default_pks, default_sks) = get_default_signers()?;

        let transaction = TransactionBuilder::new()
            .call_method_with_abi(
                self.component_address,
                &self.method_name,
                self.arguments.clone(),
                Some(default_account),
                &executor
                    .export_abi_by_component(self.component_address)
                    .map_err(Error::AbiExportError)?,
            )
            .map_err(Error::TransactionConstructionError)?
            .call_method_with_all_resources(default_account, "deposit_batch")
            .build(executor.get_nonce(default_pks))
            .sign(&default_sks);
        process_transaction(transaction, &mut executor, &self.manifest)
    }
}
