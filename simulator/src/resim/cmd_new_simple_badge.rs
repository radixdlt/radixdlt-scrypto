use clap::Parser;
use colored::Colorize;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataInit, MetadataValue, Url};
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::blueprints::resource::{
    NonFungibleDataSchema, NonFungibleResourceManagerCreateWithInitialSupplyManifestInput,
    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
};
use radix_engine_interface::blueprints::resource::{
    ResourceAction, NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
};
use radix_engine_interface::rule;
use transaction::builder::ManifestBuilder;
use transaction::model::InstructionV1;

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
    ) -> Result<Option<NonFungibleGlobalId>, Error> {
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
            metadata.set_and_lock("info_url", MetadataValue::Url(Url(info_url)));
        }
        if let Some(icon_url) = self.icon_url.clone() {
            metadata.set_and_lock("icon_url", MetadataValue::Url(Url(icon_url)));
        };

        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 5000u32.into())
            .add_instruction(InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(&NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                    owner_role: OwnerRole::None,
                    id_type: NonFungibleIdType::Integer,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                    metadata: ModuleConfig {
                        init: metadata,
                        roles: RolesInit::default(),
                    },
                    access_rules: btreemap!(
                        ResourceAction::Withdraw => (rule!(allow_all), rule!(deny_all))
                    ),
                    entries: btreemap!(
                        NonFungibleLocalId::integer(1) => (to_manifest_value_and_unwrap!(&EmptyStruct {}) ,),
                    ),
                    address_reservation: None,
                }),
            })
            .0
            .call_method(
                default_account,
                "try_deposit_batch_or_refund",
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
