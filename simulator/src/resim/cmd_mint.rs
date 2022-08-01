use clap::Parser;
use scrypto::core::Network;
use scrypto::engine::types::*;
use transaction::builder::ManifestBuilder;

use crate::resim::*;

/// Mint resource
#[derive(Parser, Debug)]
pub struct Mint {
    /// The amount of resource to mint
    amount: Decimal,

    /// The resource address
    resource_address: ResourceAddress,

    /// The minter resource address
    minter_resource_address: ResourceAddress,

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

impl Mint {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let default_account = get_default_account()?;

        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .lock_fee(10.into(), SYSTEM_COMPONENT)
            .create_proof_from_account(self.minter_resource_address, default_account)
            .mint(self.amount, self.resource_address)
            .call_method_with_all_resources(default_account, "deposit_batch")
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
