use clap::Parser;
use radix_engine::transaction::*;
use scrypto::engine::types::*;

use crate::resim::*;

/// Call a function
#[derive(Parser, Debug)]
pub struct CallFunction {
    /// The package which the function belongs to
    package_id: PackageId,

    /// The name of the blueprint which the function belongs to
    blueprint_name: String,

    /// The function name
    function_name: String,

    /// The call arguments, e.g. \"5\", \"hello\", \"amount,resource_def_id\" for Bucket, or \"#id1,#id2,..,resource_def_id\" for non-fungible Bucket
    arguments: Vec<String>,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl CallFunction {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let default_account = get_default_account()?;
        let (default_pks, default_sks) = get_default_signers()?;

        let transaction = TransactionBuilder::new(&executor)
            .parse_args_and_call_function(
                self.package_id,
                &self.blueprint_name,
                &self.function_name,
                self.arguments.clone(),
                Some(default_account),
            )
            .call_method_with_all_resources(default_account, "deposit_batch")
            .build(default_pks)
            .map_err(Error::TransactionConstructionError)?
            .sign(&default_sks);
        process_transaction(transaction, &mut executor, &self.manifest)
    }
}
