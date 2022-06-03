use clap::Parser;
use scrypto::engine::types::*;
use transaction::builder::ManifestBuilder;

use crate::resim::*;

/// Transfer resource to another account
#[derive(Parser, Debug)]
pub struct Transfer {
    /// The amount to transfer.
    amount: Decimal,

    /// The resource address.
    resource_address: ResourceAddress,

    /// The recipient component address.
    recipient: ComponentAddress,

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

impl Transfer {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account_by_amount(
                self.amount,
                self.resource_address,
                get_default_account()?,
            )
            .call_method_with_all_resources(self.recipient, "deposit_batch")
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
