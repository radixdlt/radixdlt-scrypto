use clap::Parser;
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::resim::*;

/// Call a function
#[derive(Parser, Debug)]
pub struct CallFunction {
    /// The address of the package which the function belongs to
    package_address: Address,

    /// The name of the blueprint which the function belongs to
    blueprint_name: String,

    /// The function name
    function_name: String,

    /// The call arguments, e.g. \"5\", \"hello\", \"amount,resource_address\" for Bucket, or \"#id1,#id2,..,resource_address\" for NFT Bucket
    arguments: Vec<String>,

    /// The transaction signers
    #[clap(short, long)]
    signers: Vec<Address>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl CallFunction {
    pub fn run(&self) -> Result<(), Error> {
        let mut configs = get_configs()?;
        let account = configs.default_account.ok_or(Error::NoDefaultAccount)?;
        let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(
            &mut ledger,
            configs.current_epoch,
            configs.nonce,
            self.trace,
        );
        let transaction = TransactionBuilder::new(&executor)
            .call_function(
                self.package_address,
                &self.blueprint_name,
                &self.function_name,
                self.arguments.clone(),
                Some(account.0),
            )
            .call_method_with_all_resources(account.0, "deposit_batch")
            .build(self.signers.clone())
            .map_err(Error::TransactionConstructionError)?;
        let receipt = executor
            .run(transaction)
            .map_err(Error::TransactionValidationError)?;

        println!("{:?}", receipt);
        if receipt.result.is_ok() {
            configs.nonce = executor.nonce();
            set_configs(configs)?;
        }

        receipt.result.map_err(Error::TransactionExecutionError)
    }
}
