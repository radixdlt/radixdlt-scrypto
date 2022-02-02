use clap::Parser;
use radix_engine::transaction::*;
use scrypto::rust::collections::*;
use scrypto::types::*;

use crate::ledger::*;
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
    signers: Vec<Address>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl NewTokenMutable {
    pub fn run(&self) -> Result<(), Error> {
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

        let mut configs = get_configs()?;
        let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(
            &mut ledger,
            configs.current_epoch,
            configs.nonce,
            self.trace,
        );
        let transaction = TransactionBuilder::new(&executor)
            .new_token_mutable(metadata, self.badge_address)
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
