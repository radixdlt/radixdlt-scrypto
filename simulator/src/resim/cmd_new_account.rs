use clap::Parser;
use colored::*;
use scrypto::types::*;

use crate::resim::*;

/// Create an account
#[derive(Parser, Debug)]
pub struct NewAccount {
    /// The transaction signers
    #[clap(short, long)]
    signers: Option<Vec<Address>>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl NewAccount {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let public_key = executor.new_public_key();
        let account = executor.new_account(public_key);

        println!("A new account has been created!");
        println!("Account address: {}", account.to_string().green());
        println!("Public key: {}", public_key.to_string().green());
        if get_configs()?.is_none() {
            println!(
                "No configuration found on system. will use the above account and public key as default."
            );
            set_configs(&Configs {
                default_account: account,
                default_signers: vec![public_key],
            })?;
        }

        Ok(())
    }
}
