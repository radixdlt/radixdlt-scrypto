use clap::Parser;
use colored::*;
use radix_engine::types::*;
use rand::Rng;

use crate::resim::Error::TransactionExecutionError;
use crate::resim::*;

/// Create an account
#[derive(Parser, Debug)]
pub struct NewAccount {
    /// The network to use when outputting manifest, [simulator | adapanet | nebunet | mainnet]
    #[clap(short, long)]
    network: Option<String>,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl NewAccount {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let secret = rand::thread_rng().gen::<[u8; 32]>();
        let private_key = EcdsaPrivateKey::from_bytes(&secret).unwrap();
        let public_key = private_key.public_key();
        let auth_address = NonFungibleAddress::from_public_key(&public_key);
        let withdraw_auth = rule!(require(auth_address));
        let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
            .call_method(SYS_FAUCET_COMPONENT, "free", args!())
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.new_account_with_resource(&withdraw_auth, bucket_id)
            })
            .build();

        let receipt = handle_manifest(
            manifest,
            &Some("".to_string()), // explicit empty signer public keys
            &self.network,
            &self.manifest,
            ExecutionPrivilege::User,
            self.trace,
            false,
            out,
        )?;

        let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());

        if let Some(receipt) = receipt {
            let commit_result = receipt.result.expect_commit();
            commit_result
                .outcome
                .success_or_else(TransactionExecutionError)?;

            let account = commit_result.entity_changes.new_component_addresses[0];
            writeln!(out, "A new account has been created!").map_err(Error::IOError)?;
            writeln!(
                out,
                "Account component address: {}",
                bech32_encoder
                    .encode_component_address(&account)
                    .to_string()
                    .green()
            )
            .map_err(Error::IOError)?;
            writeln!(out, "Public key: {}", public_key.to_string().green())
                .map_err(Error::IOError)?;
            writeln!(
                out,
                "Private key: {}",
                hex::encode(private_key.to_bytes()).green()
            )
            .map_err(Error::IOError)?;

            let mut configs = get_configs()?;
            if configs.default_account.is_none() {
                writeln!(
                    out,
                    "No configuration found on system. will use the above account as default."
                )
                .map_err(Error::IOError)?;
                configs.default_account = Some((account, hex::encode(private_key.to_bytes())));
                set_configs(&configs)?;
            }
        } else {
            writeln!(out, "A manifest has been produced for the following key pair. To complete account creation, you will need to run the manifest!").map_err(Error::IOError)?;
            writeln!(out, "Public key: {}", public_key.to_string().green())
                .map_err(Error::IOError)?;
            writeln!(
                out,
                "Private key: {}",
                hex::encode(private_key.to_bytes()).green()
            )
            .map_err(Error::IOError)?;
        }
        Ok(())
    }
}
