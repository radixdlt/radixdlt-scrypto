use clap::Parser;
use radix_engine::transaction::*;
use scrypto::types::*;

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
    signers: Option<Vec<Address>>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl CallMethod {
    pub fn run(&self) -> Result<(), Error> {
        let mut runner = TransactionRunner::new()?;
        let default_account = runner.default_account()?;
        let default_signers = runner.default_signers()?;
        let transaction = TransactionBuilder::new(&runner.executor(self.trace))
            .call_method(
                self.component_address,
                &self.method_name,
                self.arguments.clone(),
                Some(default_account),
            )
            .call_method_with_all_resources(default_account, "deposit_batch")
            .build(self.signers.clone().unwrap_or(default_signers))
            .map_err(Error::TransactionConstructionError)?;
        runner.run_transaction(transaction, self.trace, |receipt| println!("{:?}", receipt))
    }
}
