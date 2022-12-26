use clap::Parser;
use radix_engine::types::*;
use radix_engine_interface::data::*;
use transaction::builder::ManifestBuilder;

use crate::resim::*;
use crate::utils::*;

/// Transfer resource to another account
#[derive(Parser, Debug)]
pub struct Transfer {
    /// The amount to transfer.
    amount: Decimal,

    /// The resource address.
    resource_address: SimulatorResourceAddress,

    /// The recipient component address.
    recipient: SimulatorComponentAddress,

    /// The proofs to add to the auth zone
    #[clap(short, long, multiple = true)]
    proofs: Option<Vec<String>>,

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

impl Transfer {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let bech32_decoder = Bech32Decoder::for_simulator();

        let default_account = get_default_account()?;
        let proofs = self.proofs.clone().unwrap_or_default();

        let mut manifest_builder = &mut ManifestBuilder::new();
        for resource_specifier in proofs {
            manifest_builder = manifest_builder.borrow_mut(|builder| {
                add_create_proof_instruction_from_account_with_resource_specifier(
                    builder,
                    &bech32_decoder,
                    default_account,
                    resource_specifier,
                )
                .map_err(Error::FailedToBuildArgs)?;
                Ok(builder)
            })?;
        }

        let manifest = manifest_builder
            .lock_fee(FAUCET_COMPONENT, 100.into())
            .withdraw_from_account_by_amount(default_account, self.amount, self.resource_address.0)
            .call_method(
                self.recipient.0,
                "deposit_batch",
                args!(Expression::entire_worktop()),
            )
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
