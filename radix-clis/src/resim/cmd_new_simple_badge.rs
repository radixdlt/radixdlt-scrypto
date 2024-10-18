use clap::Parser;
use colored::Colorize;
use radix_common::prelude::*;
use radix_engine_interface::object_modules::metadata::{MetadataInit, MetadataValue, UncheckedUrl};
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::prelude::*;

use crate::resim::*;

#[derive(ManifestSbor, ScryptoSbor)]
struct EmptyStruct;

/// Create a non-fungible badge with fixed supply
#[derive(Parser, Debug)]
pub struct NewSimpleBadge {
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

impl NewSimpleBadge {
    pub fn run<O: std::io::Write>(
        &self,
        out: &mut O,
    ) -> Result<Option<NonFungibleGlobalId>, String> {
        let network_definition = NetworkDefinition::simulator();
        let default_account = get_default_account()?;
        let mut metadata = MetadataInit::new();
        if let Some(symbol) = self.symbol.clone() {
            metadata.set_and_lock("symbol", MetadataValue::String(symbol));
        }
        if let Some(name) = self.name.clone() {
            metadata.set_and_lock("name", MetadataValue::String(name));
        }
        if let Some(description) = self.description.clone() {
            metadata.set_and_lock("description", MetadataValue::String(description));
        }
        if let Some(info_url) = self.info_url.clone() {
            metadata.set_and_lock("info_url", MetadataValue::Url(UncheckedUrl::of(info_url)));
        }
        if let Some(icon_url) = self.icon_url.clone() {
            metadata.set_and_lock("icon_url", MetadataValue::Url(UncheckedUrl::of(icon_url)));
        };

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                false,
                NonFungibleResourceRoles::default(),
                ModuleConfig {
                    init: metadata,
                    roles: RoleAssignmentInit::default(),
                },
                Some(btreemap!(
                    NonFungibleLocalId::integer(1) => (),
                )),
            )
            .try_deposit_entire_worktop_or_refund(default_account, None)
            .build();
        let receipt = handle_manifest(
            manifest.into(),
            &self.signing_keys,
            &self.network,
            &self.manifest,
            self.trace,
            false,
            out,
        )
        .unwrap();

        if let Some(receipt) = receipt {
            let resource_address = receipt.expect_commit(true).new_resource_addresses()[0];

            let address_bech32_encoder = AddressBech32Encoder::new(&network_definition);
            writeln!(
                out,
                "NonFungibleGlobalId: {}",
                NonFungibleGlobalId::new(resource_address, NonFungibleLocalId::integer(1))
                    // This should be the opposite of parse_args in the manifest builder
                    .to_canonical_string(&address_bech32_encoder)
                    .green()
            )
            .map_err(Error::IOError)?;

            Ok(Some(NonFungibleGlobalId::new(
                resource_address,
                NonFungibleLocalId::integer(1),
            )))
        } else {
            Ok(None)
        }
    }
}
