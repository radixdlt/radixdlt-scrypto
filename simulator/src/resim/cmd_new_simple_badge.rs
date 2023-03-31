use clap::Parser;
use colored::Colorize;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::{
    NonFungibleDataSchema, NonFungibleResourceManagerCreateWithInitialSupplyManifestInput,
    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
};
use radix_engine_interface::blueprints::resource::{
    ResourceMethodAuthKey, NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
};
use radix_engine_interface::rule;
use transaction::builder::ManifestBuilder;
use transaction::model::Instruction;

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
    pub url: Option<String>,

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
    ) -> Result<Option<NonFungibleGlobalId>, Error> {
        let network_definition = NetworkDefinition::simulator();
        let default_account = get_default_account()?;
        let mut metadata = BTreeMap::new();
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

        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100.into())
            .add_instruction(Instruction::CallFunction {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value(&NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                    id_type: NonFungibleIdType::Integer,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                    metadata,
                    access_rules: btreemap!(
                        ResourceMethodAuthKey::Withdraw => (rule!(allow_all), rule!(deny_all))
                    ),
                    entries: btreemap!(
                        NonFungibleLocalId::integer(1) => (to_manifest_value(&EmptyStruct {}) ,),
                    ),
                }),
            })
            .0
            .call_method(
                default_account,
                "deposit_batch",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = handle_manifest(
            manifest,
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

            let bech32_encoder = Bech32Encoder::new(&network_definition);
            writeln!(
                out,
                "NonFungibleGlobalId: {}",
                NonFungibleGlobalId::new(resource_address, NonFungibleLocalId::integer(1))
                    // This should be the opposite of parse_args in the manifest builder
                    .to_canonical_string(&bech32_encoder)
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
