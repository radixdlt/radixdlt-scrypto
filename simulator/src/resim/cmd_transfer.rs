use clap::Parser;
use radix_engine::types::*;

use crate::resim::*;
use crate::utils::*;

/// Transfer resource to another account
#[derive(Parser, Debug)]
pub struct Transfer {
    /// The amount to transfer.
    pub amount: Decimal,

    /// The resource address.
    pub resource_address: SimulatorResourceAddress,

    /// The recipient component address.
    pub recipient: SimulatorComponentAddress,

    /// The proofs to add to the auth zone, in form of "<resource_address>:<amount>" or "<resource_address>:<nf_local_id1>,<nf_local_id2>"
    #[clap(short, long, multiple = true)]
    pub proofs: Option<Vec<String>>,

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

impl Transfer {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let address_bech32_decoder = AddressBech32Decoder::for_simulator();

        let default_account = get_default_account()?;
        let proofs = self.proofs.clone().unwrap_or_default();

        let mut builder = ManifestBuilder::new().lock_fee_from_faucet();
        for resource_specifier in proofs {
            builder = create_proof_from_account(
                builder,
                &address_bech32_decoder,
                default_account,
                resource_specifier,
            )
            .map_err(Error::FailedToBuildArguments)?
        }
        let manifest = builder
            .withdraw_from_account(default_account, self.resource_address.0, self.amount)
            .try_deposit_batch_or_refund(self.recipient.0)
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
