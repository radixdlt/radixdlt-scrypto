#![allow(unused_must_use)]

use clap::Parser;
use scrypto::core::Network;
use scrypto::engine::types::*;
use transaction::builder::ManifestBuilder;

use crate::resim::*;

/// Call a method
#[derive(Parser, Debug)]
pub struct CallMethod {
    /// The component that the method belongs to
    component_address: ComponentAddress,

    /// The method name
    method_name: String,

    /// The call arguments
    arguments: Vec<String>,

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

impl CallMethod {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let default_account = get_default_account()?;

        let manifest = ManifestBuilder::new(Network::LocalSimulator)
            .call_method_with_abi(
                self.component_address,
                &self.method_name,
                self.arguments.clone(),
                Some(default_account),
                &export_abi_by_component(self.component_address)?,
            )
            .map_err(Error::TransactionConstructionError)?
            .call_method_with_all_resources(default_account, "deposit_batch")
            .build();
        handle_manifest(
            manifest,
            &self.signing_keys,
            &self.manifest,
            self.trace,
            true,
            out,
        )
        .map(|_| ())
    }
}
