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
        let mut runner = TransactionRunner::new()?;
        let public_key = runner.executor(self.trace).new_public_key();
        let account = runner.executor(self.trace).new_account(public_key);

        println!("{}", "=".repeat(80));
        println!("A new account has been created!");
        println!("Public key: {}", public_key.to_string().green());
        println!("Account address: {}", account.to_string().green());

        let mut configs = get_configs()?;
        if configs.default_account.is_none() {
            println!("As this is the first account, it has been set as your default account.");
            configs.default_account = Some((account, public_key));
        }
        println!("{}", "=".repeat(80));
        set_configs(configs)?;

        Ok(())
    }
}
