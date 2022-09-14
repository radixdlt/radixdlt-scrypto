use clap::Parser;
use radix_engine::types::*;
use scrypto::prelude::Expression;
use transaction::builder::ManifestBuilder;

use crate::resim::*;

/// Call a function
#[derive(Parser, Debug)]
pub struct CallFunction {
    /// The package which the function belongs to
    package_address: PackageAddress,

    /// The name of the blueprint which the function belongs to
    blueprint_name: String,

    /// The function name
    function_name: String,

    /// The call arguments, e.g. \"5\", \"hello\", \"amount,resource_address\" for Bucket, or \"#id1,#id2,..,resource_address\" for non-fungible Bucket
    arguments: Vec<String>,

    /// The proofs to add to the auth zone
    #[clap(short, long, multiple = true)]
    proofs: Option<Vec<String>>,

    /// The network to use when outputting manifest
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

impl CallFunction {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let default_account = get_default_account()?;
        let proofs = self.proofs.clone().unwrap_or_default();

        let mut manifest_builder = &mut ManifestBuilder::new(&NetworkDefinition::simulator());
        for resource_specifier in proofs {
            manifest_builder = manifest_builder
                .create_proof_from_account_by_resource_specifier(
                    resource_specifier,
                    default_account,
                )
                .map_err(Error::FailedToBuildArgs)?;
        }

        let manifest = manifest_builder
            .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
            .call_function_with_abi(
                self.package_address,
                &self.blueprint_name,
                &self.function_name,
                self.arguments.clone(),
                Some(default_account),
                &export_abi(self.package_address, &self.blueprint_name)?,
            )
            .map_err(Error::TransactionConstructionError)?
            .call_method(
                default_account,
                "deposit_batch",
                args!(Expression::entire_worktop()),
            )
            .build();
        handle_manifest(
            manifest,
            &self.signing_keys,
            &self.network,
            &self.manifest,
            false,
            self.trace,
            true,
            out,
        )
        .map(|_| ())
    }
}
