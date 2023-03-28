use clap::Parser;
use colored::*;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::{
    require, AccessRulesConfig, FromPublicKey, NonFungibleDataSchema,
    NonFungibleResourceManagerCreateWithInitialSupplyManifestInput, ResourceMethodAuthKey,
    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
};
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::rule;
use rand::Rng;
use utils::ContextualDisplay;

use crate::resim::Error::TransactionFailed;
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

#[derive(ScryptoSbor, ManifestSbor)]
struct EmptyStruct;

impl NewAccount {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let secret = rand::thread_rng().gen::<[u8; 32]>();
        let private_key = EcdsaSecp256k1PrivateKey::from_bytes(&secret).unwrap();
        let public_key = private_key.public_key();
        let auth_global_id = NonFungibleGlobalId::from_public_key(&public_key);
        let withdraw_auth = rule!(require(auth_global_id));
        let config = AccessRulesConfig::new().default(withdraw_auth.clone(), withdraw_auth);
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100.into())
            .new_account_advanced(config)
            .build();

        let receipt = handle_manifest(
            manifest,
            &Some("".to_string()), // explicit empty signer public keys
            &self.network,
            &self.manifest,
            self.trace,
            false,
            out,
        )?;

        let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::simulator());

        if let Some(ref receipt) = receipt {
            let commit_result = receipt.expect_commit(true);
            commit_result
                .outcome
                .success_or_else(|err| TransactionFailed(err.clone()))?;

            let account = commit_result.new_component_addresses()[0];
            let manifest = ManifestBuilder::new()
                .lock_fee(FAUCET_COMPONENT, 100.into())
                .call_method(FAUCET_COMPONENT, "free", manifest_args!())
                .add_instruction(Instruction::CallFunction {
                    package_address: RESOURCE_MANAGER_PACKAGE,
                    blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                    function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                        .to_string(),
                    args: to_manifest_value(&NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        id_type: NonFungibleIdType::Integer,
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                        metadata: btreemap!(
                            "name".to_owned() => "Owner Badge".to_owned()
                        ),
                        access_rules: btreemap!(
                            ResourceMethodAuthKey::Withdraw => (rule!(allow_all), rule!(deny_all))
                        ),
                        entries: btreemap!(
                            NonFungibleLocalId::integer(1) => (to_manifest_value(&EmptyStruct {}) ,),
                        ),
                    }),
                })
                .0
                .call_method(
                    account,
                    "deposit_batch",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .build();
            let receipt = handle_manifest(
                manifest,
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
                account.display(&bech32_encoder).to_string().green()
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
                    .to_canonical_string(&Bech32Encoder::for_simulator())
                    .green()
            )
            .map_err(Error::IOError)?;

            let mut configs = get_configs()?;
            if configs.default_account.is_none()
                || configs.default_private_key.is_none()
                || configs.default_owner_badge.is_none()
            {
                configs.default_account = Some(account);
                configs.default_private_key = Some(hex::encode(private_key.to_bytes()));
                configs.default_owner_badge = Some(owner_badge);
                set_configs(&configs)?;

                writeln!(
                    out,
                    "Account configuration in complete. Will use the above account as default."
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
