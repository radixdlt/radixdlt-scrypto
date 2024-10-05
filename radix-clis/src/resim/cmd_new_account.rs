use clap::Parser;
use colored::*;
use radix_common::network::NetworkDefinition;
use radix_common::prelude::*;
use radix_engine_interface::prelude::*;
use radix_engine_interface::types::FromPublicKey;
use radix_engine_interface::{metadata, metadata_init, rule};
use radix_rust::ContextualDisplay;
use rand::Rng;

use crate::resim::Error::TransactionFailed;
use crate::resim::*;

/// Create an account
#[derive(Parser, Debug)]
pub struct NewAccount {
    /// The network to use when outputting manifest, [simulator | adapanet | nebunet | mainnet]
    #[clap(short, long)]
    pub network: Option<String>,

    /// Output a transaction manifest without execution
    #[clap(short, long)]
    pub manifest: Option<PathBuf>,

    /// Turn on tracing
    #[clap(short, long)]
    pub trace: bool,
}

#[derive(ScryptoSbor, ManifestSbor)]
struct EmptyStruct;

impl NewAccount {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), String> {
        let secret = rand::thread_rng().gen::<[u8; 32]>();
        let private_key = Secp256k1PrivateKey::from_bytes(&secret).unwrap();
        let public_key = private_key.public_key();
        let auth_global_id = NonFungibleGlobalId::from_public_key(&public_key);
        let withdraw_auth = rule!(require(auth_global_id));
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .new_account_advanced(OwnerRole::Fixed(withdraw_auth), None)
            .build();

        let receipt = handle_manifest(
            manifest.into(),
            &Some("".to_string()), // explicit empty signer public keys
            &self.network,
            &self.manifest,
            self.trace,
            false,
            out,
        )?;

        let address_bech32_encoder = AddressBech32Encoder::new(&NetworkDefinition::simulator());

        if let Some(ref receipt) = receipt {
            let commit_result = receipt.expect_commit(true);
            commit_result
                .outcome
                .success_or_else(|err| TransactionFailed(err.clone()))?;

            let account = commit_result.new_component_addresses()[0];
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .create_non_fungible_resource(
                    OwnerRole::None,
                    NonFungibleIdType::Integer,
                    false,
                    NonFungibleResourceRoles::default(),
                    metadata!(
                        init {
                            "name" => "Owner Badge".to_owned(), locked;
                        }
                    ),
                    Some(btreemap!(
                        NonFungibleLocalId::integer(1) => (),
                    )),
                )
                .try_deposit_entire_worktop_or_refund(account, None)
                .build();
            let receipt = handle_manifest(
                manifest.into(),
                &Some("".to_string()), // explicit empty signer public keys
                &self.network,
                &None,
                self.trace,
                false,
                out,
            )?
            .unwrap();
            let resource_address = receipt.expect_commit(true).new_resource_addresses()[0];
            let owner_badge =
                NonFungibleGlobalId::new(resource_address, NonFungibleLocalId::integer(1));

            writeln!(out, "A new account has been created!").map_err(Error::IOError)?;
            writeln!(
                out,
                "Account component address: {}",
                account.display(&address_bech32_encoder).to_string().green()
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
            writeln!(
                out,
                "Owner badge: {}",
                owner_badge
                    .to_canonical_string(&AddressBech32Encoder::for_simulator())
                    .green()
            )
            .map_err(Error::IOError)?;

            let mut configs = get_configs()?;
            if configs.default_account.is_none()
                || configs.default_private_key.is_none()
                || configs.default_owner_badge.is_none()
            {
                configs.default_account = Some(account);
                configs.default_private_key = Some(private_key.to_hex());
                configs.default_owner_badge = Some(owner_badge);
                set_configs(&configs)?;

                writeln!(
                    out,
                    "Set up as default account since you had none, you can change it using `set-default-account`."
                )
                .map_err(Error::IOError)?;
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
