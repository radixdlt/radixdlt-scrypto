use clap::Parser;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataValue, Url};
use radix_engine_interface::api::node_modules::ModuleConfig;
use transaction::builder::ManifestBuilder;

use crate::resim::*;

/// Create a fungible badge with mutable supply
#[derive(Parser, Debug)]
pub struct NewBadgeMutable {
    /// The minter resource address
    pub minter_badge: SimulatorResourceOrNonFungibleGlobalId,

    /// The symbol
    #[clap(long)]
    pub symbol: Option<String>,

    /// The name
    #[clap(long)]
    pub name: Option<String>,

    /// The description
    #[clap(long)]
    pub description: Option<String>,

    /// The website URL
    #[clap(long)]
    pub info_url: Option<String>,

    /// The ICON url
    #[clap(long)]
    pub icon_url: Option<String>,

    /// The network to use when outputting manifest, [simulator | adapanet | nebunet | mainnet]
    #[clap(short, long)]
    pub network: Option<String>,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    pub manifest: Option<PathBuf>,

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    pub signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    pub trace: bool,
}

impl NewBadgeMutable {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let mut metadata = BTreeMap::new();
        if let Some(symbol) = self.symbol.clone() {
            metadata.insert("symbol".to_string(), MetadataValue::String(symbol));
        }
        if let Some(name) = self.name.clone() {
            metadata.insert("name".to_string(), MetadataValue::String(name));
        }
        if let Some(description) = self.description.clone() {
            metadata.insert(
                "description".to_string(),
                MetadataValue::String(description),
            );
        }
        if let Some(info_url) = self.info_url.clone() {
            metadata.insert("info_url".to_string(), MetadataValue::Url(Url(info_url)));
        }
        if let Some(icon_url) = self.icon_url.clone() {
            metadata.insert("icon_url".to_string(), MetadataValue::Url(Url(icon_url)));
        };

        let metadata = ModuleConfig {
            init: metadata.into(),
            roles: RolesInit::default(),
        };

        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 500u32.into())
            .new_badge_mutable(OwnerRole::None, metadata, self.minter_badge.clone().into())
            .build();
        handle_manifest(
            manifest,
            &self.signing_keys,
            &self.network,
            &self.manifest,
            self.trace,
            true,
            out,
        )
        .map(|_| ())
    }
}
