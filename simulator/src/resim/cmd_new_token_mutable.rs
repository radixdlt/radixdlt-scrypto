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

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// The transaction signers
    #[clap(short, long)]
    signers: Option<Vec<PublicKey>>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl NewTokenMutable {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let default_signers = get_default_signers()?;
        let mut metadata = HashMap::new();
        if let Some(symbol) = self.symbol.clone() {
            metadata.insert("symbol".to_string(), symbol);
        }
        if let Some(name) = self.name.clone() {
            metadata.insert("name".to_string(), name);
        }
        if let Some(description) = self.description.clone() {
            metadata.insert("description".to_string(), description);
        }
        if let Some(url) = self.url.clone() {
            metadata.insert("url".to_string(), url);
        }
        if let Some(icon_url) = self.icon_url.clone() {
            metadata.insert("icon_url".to_string(), icon_url);
        };
        let signatures = self.signers.clone().map(|v| v.into_iter().map(|k| k.0).collect())
            .unwrap_or(default_signers);
        let transaction = TransactionBuilder::new(&executor)
            .new_token_mutable(metadata, self.badge_address)
            .build(signatures)
            .map_err(Error::TransactionConstructionError)?;
        process_transaction(transaction, &mut executor, &self.manifest)
    }
}
