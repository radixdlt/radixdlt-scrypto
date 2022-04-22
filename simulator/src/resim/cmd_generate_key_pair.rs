use clap::Parser;
use colored::*;
use rand::Rng;
use scrypto::prelude::*;

use crate::resim::*;

/// Generate a key pair
#[derive(Parser, Debug)]
pub struct GenerateKeyPair {}

impl GenerateKeyPair {
    pub fn run(&self) -> Result<(), Error> {
        let secret = rand::thread_rng().gen::<[u8; 32]>();
        let private_key = EcdsaPrivateKey::from_bytes(&secret).unwrap();
        let public_key = private_key.public_key();
        println!("Public key: {}", public_key.to_string().green());
        println!(
            "Private key: {}",
            hex::encode(private_key.to_bytes()).green()
        );
        Ok(())
    }
}
