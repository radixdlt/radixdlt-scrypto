use clap::Parser;
use colored::*;
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::resim::*;

/// Create an account
#[derive(Parser, Debug)]
pub struct NewAccount {
    /// The transaction signers
    #[clap(short, long)]
    signers: Vec<Address>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl NewAccount {
    pub fn run(&self) -> Result<(), Error> {
        let mut configs = get_configs()?;
        let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(
            &mut ledger,
            configs.current_epoch,
            configs.nonce,
            self.trace,
        );
        let key = executor.new_public_key();
        let transaction = TransactionBuilder::new(&executor)
            .call_method(
                SYSTEM_COMPONENT,
                "free_xrd",
                vec!["1000000".to_owned()],
                None,
            )
            .new_account_with_resource(key, 1000000.into(), RADIX_TOKEN)
            .build(self.signers.clone())
            .map_err(Error::TransactionConstructionError)?;
        let receipt = executor.run(transaction).unwrap();
        println!("{:?}", receipt);

        if receipt.result.is_ok() {
            let account = receipt.component(0).unwrap();
            println!("{}", "=".repeat(80));
            println!("A new account has been created!");
            println!("Public key: {}", key.to_string().green());
            println!("Account address: {}", account.to_string().green());
            if configs.default_account.is_none() {
                println!("As this is the first account, it has been set as your default account.");
                configs.default_account = Some((receipt.component(0).unwrap(), key));
            }
            println!("{}", "=".repeat(80));

            configs.nonce = executor.nonce();
            set_configs(configs)?;
            Ok(())
        } else {
            receipt.result.map_err(Error::TransactionExecutionError)
        }
    }
}
