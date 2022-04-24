
use clap::Parser;
use colored::*;
use rand::Rng;
use scrypto::prelude::*;

use crate::resim::*;

/// Create an account
#[derive(Parser, Debug)]
pub struct NewAccount {
    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl NewAccount {

    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let mut ledger = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut executor = TransactionExecutor::new(&mut ledger, self.trace);

        if let Some(path) = &self.manifest {
            let secret = rand::thread_rng().gen::<[u8; 32]>();
            let private_key = EcdsaPrivateKey::from_bytes(&secret).unwrap();
            let public_key = private_key.public_key();
            let auth_address = NonFungibleAddress::new(
                ECDSA_TOKEN,
                NonFungibleId::from_bytes(public_key.to_vec()),
            );
            let withdraw_auth = auth!(require(auth_address));
            let transaction = TransactionBuilder::new()
                .call_method(SYSTEM_COMPONENT, "free_xrd", vec![])
                .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                    builder.new_account_with_resource(&withdraw_auth, bucket_id)
                })
                .build_with_no_nonce();
            let manifest = decompile(&transaction).map_err(Error::DecompileError)?;
            writeln!(out, "A manifest has been produced for the following key pair. To complete account creation, you will need to run the manifest!").map_err(Error::IOError)?;
            writeln!(out, "Public key: {}", public_key.to_string().green()).map_err(Error::IOError)?;
            writeln!(out, 
                "Private key: {}",
                hex::encode(private_key.to_bytes()).green()
            ).map_err(Error::IOError)?;
            return fs::write(path, manifest).map_err(Error::IOError);
        }

        let (public_key, private_key, account) = executor.new_account();
        writeln!(out, "A new account has been created!").map_err(Error::IOError)?;
        writeln!(out, "Account component address: {}", account.to_string().green()).map_err(Error::IOError)?;
        writeln!(out, "Public key: {}", public_key.to_string().green()).map_err(Error::IOError)?;
        writeln!(out, 
            "Private key: {}",
            hex::encode(private_key.to_bytes()).green()
        ).map_err(Error::IOError)?;
        if get_configs()?.is_none() {
            writeln!(out, "No configuration found on system. will use the above account as default.").map_err(Error::IOError)?;
            set_configs(&Configs {
                default_account: account,
                default_public_key: public_key,
                default_private_key: private_key.to_bytes(),
            })?;
        }

        Ok(())
    }
}
