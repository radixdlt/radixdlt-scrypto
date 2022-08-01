use clap::Parser;
use sbor::rust::collections::*;
use scrypto::core::Network;
use scrypto::engine::types::*;
use transaction::builder::ManifestBuilder;

use crate::resim::*;

/// Create a badge with mutable supply
#[derive(Parser, Debug)]
pub struct NewBadgeMutable {
    /// The minter resource address
    minter_resource_address: ResourceAddress,

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

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl NewBadgeMutable {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
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

        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .lock_fee(10.into(), SYSTEM_COMPONENT)
            .new_badge_mutable(metadata, self.minter_resource_address)
            .build();
        handle_manifest(
            manifest,
            &self.signing_keys,
            &self.manifest,
            false,
            self.trace,
            true,
            out,
        )
        .map(|_| ())
    }
}
