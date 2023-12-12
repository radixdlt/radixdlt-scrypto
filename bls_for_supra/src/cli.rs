use crate::gateway::*;
use crate::utils::*;
use clap::{Parser, Subcommand};
use scrypto::blueprints::package::PackageDefinition;
use std::fs;
use std::{thread, time};
use transaction::prelude::*;

// Enkinet network data
const NETWORK_ID: u8 = 0x21;
const NETWORK_NAME: &str = "enkinet";
const NETWORK_HRP_SUFFIX: &str = "tdx_21_";
const GATEWAY_URL: &str = "https://enkinet-gateway.radixdlt.com";

const CRYPTO_SCRYPTO_BLUEPRINT_NAME: &str = "CryptoScrypto";

// This is the package address of the published CryptoScrypto blueprint.
// If you publish it by yourself you can use the new adress as well.
const CRYPTO_SCRYPTO_PACKAGE_ADDRESS: &str =
    "package_tdx_21_1pkt7zdllsneytdc9g60xn9jjhhwx7jaqxmeh58l4dwyx7rt5z9428f";

const TEST_MSG: &str = "Hello World!";
const _TEST_MSG_HASH: &str = "3ea2f1d0abf3fc66cf29eebb70cbd4e7fe762ef8a09bcc06c8edf641230afec0";
// Below key is derived from secret key: 5B00CC8C7153F39EF2E6E2FADB1BB95A1F4BF21F43CC5B28EFA9E526FB788C08
const TEST_PUB_KEY: &str = "8a38419cb83c15a92d11243384bea0acd15cbacc24b385b9c577b17272d6ad68bb53c52dbbf79324005528d2d73c2643";
const TEST_SIGNATURE: &str = "82131f69b6699755f830e29d6ed41cbf759591a2ab598aa4e9686113341118d1db900d190436048601791121b5757c341045d4d0c94a95ec31a9ba6205f9b7504de85dadff52874375c58eec6cec28397279de87d5595101e398d31646d345bb";

const CRYPTO_SCRYPTO_CODE_PATH: &str = "crypto_scrypto/crypto_scrypto.wasm";
const CRYPTO_SCRYPTO_RPD_PATH: &str = "crypto_scrypto/crypto_scrypto.rpd";
const CRYPTO_SCRYPTO_METADATA: &str = "CryptoScrypto package for Supra";

#[derive(Parser)]
#[command(author, version, about, long_about, verbatim_doc_comment)]
#[command(propagate_version = true)]
/// Simple CLI tool to demonstrate how to work with Scrypto blueprints using Rust language.
///
/// It communicates with the network via Gateway using HTTP REST API.
/// It performs:
/// - building transaction manifest for given command
/// - signing the transaction
/// - submitting the transaction to the network
/// - getting the transaction output
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get gateway status. This is sanity check, whether gateway is working fine.
    GatewayStatus,
    /// Calculate Keccak256 hash over given message
    KeccakHash(KeccakHash),
    /// Perform BLS verification over given message, public key and signature
    BlsVerify(BlsVerify),
    /// Publish given WASM and RPD files as a package
    PublishPackage(PublishPackage),
}

#[derive(Debug, Parser)]
struct KeccakHash {
    #[arg(long, short = 'a', default_value_t = CRYPTO_SCRYPTO_PACKAGE_ADDRESS.to_string())]
    /// Package address of the CryptoScrypto blueprint
    package_address: String,
    #[arg(long, short, default_value_t = TEST_MSG.to_string())]
    /// Message to hash
    msg: String,
}

#[derive(Debug, Parser)]
struct BlsVerify {
    #[arg(long, short = 'a', default_value_t = CRYPTO_SCRYPTO_PACKAGE_ADDRESS.to_string())]
    /// Package address of the CryptoScrypto blueprint
    package_address: String,
    #[arg(long, short, default_value_t = TEST_MSG.to_string())]
    /// Message to verify signature with (it will be hashed before with Keccak256)
    msg: String,
    #[arg(long, short, default_value_t = TEST_PUB_KEY.to_string())]
    /// BLS public key to perform verification (hex-encoded string)
    public_key: String,
    /// BLS signature to verify (hex-encoded string)
    #[arg(long, short, default_value_t = TEST_SIGNATURE.to_string())]
    signature: String,
}

#[derive(Debug, Parser)]
struct PublishPackage {
    #[arg(long, short, default_value_t = CRYPTO_SCRYPTO_CODE_PATH.to_string())]
    /// Scrypto blueprint WASM file, output of 'scrypto build' command
    code_path: String,
    #[arg(long, short, default_value_t = CRYPTO_SCRYPTO_RPD_PATH.to_string())]
    /// Scrypto blueprint package definition file, output of 'scrypto build' command
    rpd_path: String,
    #[arg(long, short, default_value_t = CRYPTO_SCRYPTO_METADATA.to_string())]
    /// Package metadata to set for 'Description' key
    metadata: String,
}

struct CliCtx {
    gateway: GatewayApiClient,
    network_definition: NetworkDefinition,
    address_decoder: AddressBech32Decoder,
    address_encoder: AddressBech32Encoder,
    hash_encoder: TransactionHashBech32Encoder,
    private_key: Secp256k1PrivateKey,
}

impl CliCtx {
    fn new() -> Self {
        let gateway = GatewayApiClient::new(GATEWAY_URL);
        let network_definition = NetworkDefinition {
            id: NETWORK_ID,
            logical_name: String::from(NETWORK_NAME),
            hrp_suffix: String::from(NETWORK_HRP_SUFFIX),
        };
        let address_decoder = AddressBech32Decoder::new(&network_definition);
        let address_encoder = AddressBech32Encoder::new(&network_definition);
        let hash_encoder = TransactionHashBech32Encoder::new(&network_definition);

        // Key must be generated randomly.
        // For the sake of the simplicity we derive it from hardcoded integer.
        let private_key = Secp256k1PrivateKey::from_u64(3).unwrap();
        Self {
            gateway,
            network_definition,
            address_decoder,
            address_encoder,
            hash_encoder,
            private_key,
        }
    }

    fn cmd_gateway_status(&self) {
        let status = self.gateway.gateway_status();
        println!("gw status = {:?}", status);
    }

    fn execute_transaction(&self, manifest: TransactionManifestV1) -> TransactionDetails {
        let current_epoch = self.gateway.current_epoch();

        let (notarized_transaction, intent_hash) = create_notarized_transaction(
            &self.network_definition,
            current_epoch,
            &self.private_key,
            manifest,
        );

        let _ = self.gateway.transaction_submit(notarized_transaction);

        // Intent hash (unique identifier), which is often used
        // to query it's status in the gateway or in the dashboard.
        // It must be converted to Bech32 format before.
        // Eg.
        //   txid_tdx_21_14a9mm2e3fxyyh02wrz4xsalyxszez6kpqfh0a488hp8wjdvv55cq3wfzv0
        let intent_hash = self.hash_encoder.encode(&intent_hash).unwrap();
        println!("intent_hash : {}", intent_hash);

        // Wait for transaction finish
        loop {
            let status = self.gateway.transaction_status(&intent_hash);
            if !status.status.eq("Pending") {
                break;
            }
            thread::sleep(time::Duration::from_millis(1000));
        }
        self.gateway.transaction_details(&intent_hash)
    }

    // Call CryptoScrypto package "keccak256_hash" method to retrieve the digest of the message.
    fn cmd_keccak_hash(&self, cmd: &KeccakHash) {
        // Convert address from the human-readable bech32 format
        let package_address =
            PackageAddress::try_from_bech32(&self.address_decoder, &cmd.package_address).unwrap();
        let data = cmd.msg.as_bytes().to_vec();

        println!("Package address : {}", cmd.package_address);
        println!("Message         : {}", cmd.msg);

        // Build manifest
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                CRYPTO_SCRYPTO_BLUEPRINT_NAME,
                "keccak256_hash",
                manifest_args!(&data),
            )
            .build();

        let details = self.execute_transaction(manifest);
        // Gateway returns the output of the called method in the second item of
        // "transaction.receipt.output"
        // more details: https://radix-babylon-gateway-api.redoc.ly/#operation/TransactionCommittedDetails
        let output = details.get_output(1);

        // The data is in an SBOR encode in hex string.
        // We need to decode it:
        // - first to raw SBOR (byte array)
        // - then decode SBOR to the expected type
        let sbor_data = hex::decode(output).unwrap();

        let hash: Hash = scrypto_decode(&sbor_data).unwrap();
        println!("Message hash    : {}", hash);
    }

    // Call CryptoScrypto package "bls12381_v1_verify" method to verify the signature
    fn cmd_bls_verify(&self, cmd: &BlsVerify) {
        // Convert address from the human-readable bech32 format
        let package_address =
            PackageAddress::try_from_bech32(&self.address_decoder, &cmd.package_address).unwrap();
        let msg_hash = keccak256_hash(cmd.msg.clone());

        println!("Package address : {}", cmd.package_address);
        println!("Message         : {}", cmd.msg);
        println!("Message hash    : {}", msg_hash);
        println!("Publick key     : {}", cmd.public_key);
        println!("Signature       : {}", cmd.signature);

        let pub_key = Bls12381G1PublicKey::from_str(&cmd.public_key).unwrap();
        let signature = Bls12381G2Signature::from_str(&cmd.signature).unwrap();

        // Build manifest
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                CRYPTO_SCRYPTO_BLUEPRINT_NAME,
                "bls12381_v1_verify",
                manifest_args!(msg_hash.to_vec(), pub_key, signature),
            )
            .build();

        let details = self.execute_transaction(manifest);
        // Gateway returns the output of the called method in the second item of
        // "transaction.receipt.output"
        // more details: https://radix-babylon-gateway-api.redoc.ly/#operation/TransactionCommittedDetails
        let output = details.get_output(1);

        // The data is in an SBOR encode in hex string.
        // We need to decode it:
        // - first to raw SBOR (byte array)
        // - then decode SBOR to the expected type
        let sbor_data = hex::decode(output).unwrap();

        let result: bool = scrypto_decode(&sbor_data).unwrap();
        println!("BLS verify  : {:?}", result);
    }

    // Publish package using given *.wasm and *.rpd files
    fn cmd_publish_package(&self, cmd: &PublishPackage) {
        println!("WASM file: {}", cmd.code_path);
        println!("RPD file : {}", cmd.rpd_path);
        println!("Metadata : {}", cmd.metadata);

        let mut metadata = BTreeMap::new();
        metadata.insert(
            "Description".to_string(),
            MetadataValue::String(cmd.metadata.to_string()),
        );
        let code = fs::read(cmd.code_path.clone()).unwrap();
        let rpd: PackageDefinition =
            manifest_decode(&fs::read(cmd.rpd_path.clone()).unwrap()).unwrap();

        // Build manifest
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .publish_package_advanced(None, code, rpd, metadata, OwnerRole::None)
            .build();

        let details = self.execute_transaction(manifest);
        // Gateway returns the output of the called method in the second item of
        // "transaction.receipt.output"
        // more details: https://radix-babylon-gateway-api.redoc.ly/#operation/TransactionCommittedDetails
        let output = details.get_output(1);

        // The data is in an SBOR encode in hex string.
        // We need to decode it:
        // - first to raw SBOR (byte array)
        // - then decode SBOR to the expected type
        let sbor_data = hex::decode(output).unwrap();

        let address: PackageAddress = scrypto_decode(&sbor_data).unwrap();

        // Encode the address into human-readabl bech32 format
        let address = self.address_encoder.encode(address.as_ref()).unwrap();
        println!("Published package address  : {}", address);
    }
}

pub fn run() {
    let cli = Cli::parse();

    let ctx = CliCtx::new();

    match &cli.command {
        Commands::GatewayStatus => {
            ctx.cmd_gateway_status();
        }
        Commands::KeccakHash(cmd) => {
            ctx.cmd_keccak_hash(cmd);
        }
        Commands::BlsVerify(cmd) => {
            ctx.cmd_bls_verify(cmd);
        }
        Commands::PublishPackage(cmd) => {
            ctx.cmd_publish_package(cmd);
        }
    }
}
