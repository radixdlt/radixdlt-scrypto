use clap::Parser;
use colored::Colorize;
use radix_engine::types::*;
use radix_engine_interface::data::*;
use radix_engine_interface::model::NonFungibleAddress;
use radix_engine_interface::node::*;
use radix_engine_interface::rule;
use transaction::builder::ManifestBuilder;

use crate::resim::*;

#[scrypto(TypeId, Encode, Decode)]
struct EmptyStruct;

/// Create a badge with fixed supply
#[derive(Parser, Debug)]
pub struct NewSimpleBadge {
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

    /// The network to use when outputting manifest, [simulator | adapanet | nebunet | mainnet]
    #[clap(short, long)]
    network: Option<String>,

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

impl NewSimpleBadge {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
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

        let mut resource_auth = BTreeMap::new();
        resource_auth.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );

        let manifest = ManifestBuilder::new(&network_definition)
            .lock_fee(FAUCET_COMPONENT, 100.into())
            .create_resource(
                ResourceType::NonFungible {
                    id_type: NonFungibleIdType::U32,
                },
                metadata,
                resource_auth,
                Option::Some(MintParams::NonFungible {
                    entries: BTreeMap::from([(
                        NonFungibleId::U32(1),
                        (
                            scrypto_encode(&EmptyStruct).unwrap(),
                            scrypto_encode(&EmptyStruct).unwrap(),
                        ),
                    )]),
                }),
            )
            .call_method(
                default_account,
                "deposit_batch",
                args!(Expression::entire_worktop()),
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
        .unwrap()
        .unwrap();

        let resource_address = receipt
            .expect_commit()
            .entity_changes
            .new_resource_addresses[0];

        let bech32_encoder = Bech32Encoder::new(&network_definition);
        writeln!(
            out,
            "NFAddress: {}",
            NonFungibleAddress::new(resource_address, NonFungibleId::U32(1))
                // This should be the opposite of parse_args in the manifest builder
                .to_canonical_combined_string(&bech32_encoder)
                .green()
        )
        .map_err(Error::IOError)?;
        writeln!(
            out,
            "Resource: {}",
            resource_address.to_string(&bech32_encoder).green()
        )
        .map_err(Error::IOError)?;
        writeln!(
            out,
            "NFID: {}",
            NonFungibleId::U32(1).to_combined_simple_string()
        )
        .map_err(Error::IOError)?;
        Ok(())
    }
}
