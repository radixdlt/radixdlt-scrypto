use clap::Parser;
use radix_engine::transaction::*;
use scrypto::engine::types::*;

use crate::resim::*;

/// Call a function
#[derive(Parser, Debug)]
pub struct CallFunction {
    /// The package which the function belongs to
    package_address: PackageAddress,

    /// The name of the blueprint which the function belongs to
    blueprint_name: String,

    /// The function name
    function_name: String,

    /// The call arguments, e.g. \"5\", \"hello\", \"amount,resource_address\" for Bucket, or \"#id1,#id2,..,resource_address\" for non-fungible Bucket
    arguments: Vec<String>,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl CallFunction {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::new(get_data_dir()?);
        let wasm_engine = default_wasm_engine();
        let mut executor = TransactionExecutor::new(&mut ledger, wasm_engine, self.trace);
        let default_account = get_default_account()?;

        let transaction = TransactionBuilder::new()
            .call_function_with_abi(
                self.package_address,
                &self.blueprint_name,
                &self.function_name,
                self.arguments.clone(),
                Some(default_account),
                &executor
                    .export_abi(self.package_address, &self.blueprint_name)
                    .map_err(Error::AbiExportError)?,
            )
            .map_err(Error::TransactionConstructionError)?
            .call_method_with_all_resources(default_account, "deposit_batch")
            .build_with_no_nonce();
        process_transaction(
            &mut executor,
            transaction,
            &self.signing_keys,
            &self.manifest,
            out,
        )
    }
}
