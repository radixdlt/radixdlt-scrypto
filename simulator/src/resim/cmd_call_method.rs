use clap::Parser;
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::resim::*;

/// Call a method
#[derive(Parser, Debug)]
pub struct CallMethod {
    /// The address of the component that the method belongs to
    component_address: Address,

    /// The method name
    method_name: String,

    /// The call arguments
    arguments: Vec<String>,

    /// The transaction signers
    #[clap(short, long)]
    signers: Vec<Address>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl CallMethod {
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
            .call_method(
                self.component_address,
                &self.method_name,
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
