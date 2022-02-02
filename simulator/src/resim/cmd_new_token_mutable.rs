use clap::Parser;
use radix_engine::transaction::*;
use scrypto::rust::collections::*;
use scrypto::types::*;

use crate::resim::*;

/// Create a token with mutable supply
#[derive(Parser, Debug)]
pub struct NewTokenMutable {
    /// The minter badge address
    badge_address: Address,

    /// The symbol
    #[clap(long)]
    symbol: Option<String>,

    /// The name
    #[clap(long)]
    name: Option<String>,

    /// The description
    #[clap(long)]
    description: Option<String>,

    /// The website URL
    #[clap(long)]
    url: Option<String>,

    /// The ICON url
    #[clap(long)]
    icon_url: Option<String>,

    /// The transaction signers
    #[clap(short, long)]
    signers: Option<Vec<Address>>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl NewTokenMutable {
    pub fn run(&self) -> Result<(), Error> {
        let mut runner = TransactionRunner::new()?;
        let default_signers = runner.default_signers()?;
        let mut metadata = HashMap::new();
        if let Some(symbol) = self.symbol.clone() {
            metadata.insert("symbol".to_string(), symbol);
        }
        if let Some(name) = self.symbol.clone() {
            metadata.insert("name".to_string(), name);
        }
        if let Some(description) = self.symbol.clone() {
            metadata.insert("description".to_string(), description);
        }
        if let Some(url) = self.symbol.clone() {
            metadata.insert("url".to_string(), url);
        }
        if let Some(icon_url) = self.symbol.clone() {
            metadata.insert("icon_url".to_string(), icon_url);
        };
        let transaction = TransactionBuilder::new(&runner.executor(self.trace))
            .new_token_mutable(metadata, self.badge_address)
            .build(self.signers.clone().unwrap_or(default_signers))
            .map_err(Error::TransactionConstructionError)?;
        runner.run_transaction(transaction, self.trace, |receipt| println!("{:?}", receipt))
    }
}
