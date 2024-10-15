use clap::Parser;
use radix_common::prelude::*;

use crate::resim::*;
use crate::utils::*;

/// Transfer resource to another account
#[derive(Parser, Debug)]
pub struct Transfer {
    /// The resource specifier.
    pub resource_specifier: String,

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
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), String> {
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

        let resource_specifier =
            parse_resource_specifier(&self.resource_specifier, &address_bech32_decoder)
                .map_err(|_| Error::InvalidResourceSpecifier(self.resource_specifier.clone()))?;

        builder = match resource_specifier {
            crate::utils::ResourceSpecifier::Amount(amount, resource_address) => {
                builder.withdraw_from_account(default_account, resource_address, amount)
            }
            crate::utils::ResourceSpecifier::Ids(ids, resource_address) => {
                builder.withdraw_non_fungibles_from_account(default_account, resource_address, ids)
            }
        };
        let manifest = builder
            .try_deposit_entire_worktop_or_refund(self.recipient.0, None)
            .build();
        handle_manifest(
            manifest.into(),
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
