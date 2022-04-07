use clap::Parser;
use radix_engine::transaction::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::*;

use crate::resim::*;

/// Create a badge with fixed supply
#[derive(Parser, Debug)]
pub struct NewBadgeFixed {
    /// The total supply
    total_supply: Decimal,

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

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl NewBadgeFixed {
    pub fn run(&self) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);
        let default_account = get_default_account()?;
        let (default_pks, default_sks) = get_default_signers()?;
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

        let transaction = TransactionBuilder::new()
            .new_badge_fixed(metadata, self.total_supply)
            .call_method_with_all_resources(default_account, "deposit_batch")
            .build(executor.get_nonce(default_pks))
            .sign(&default_sks);
        process_transaction(transaction, &mut executor, &self.manifest)
    }
}
