use clap::Parser;
use radix_common::data::manifest::manifest_decode;
use radix_common::prelude::*;
use radix_engine::utils::*;
use radix_transactions::manifest::*;
use radix_transactions::prelude::*;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

/// Radix transaction manifest decompiler
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, name = "rtmd")]
pub struct Args {
    /// Path to the output file
    #[clap(short, long)]
    output: PathBuf,

    /// Network to Use [Simulator | Alphanet | Mainnet]
    #[clap(short, long)]
    network: Option<String>,

    /// Whether to export blobs
    #[clap(short, long, action)]
    export_blobs: bool,

    /// Input file
    #[clap(required = true)]
    input: PathBuf,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    DecodeError(sbor::DecodeError),
    DecompileError(DecompileError),
    ParseNetworkError(ParseNetworkError),
    InstructionSchemaValidationError(radix_engine::utils::LocatedInstructionSchemaValidationError),
}

impl fmt::Display for Error {
    // TODO Implement pretty error printing
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Error> for String {
    fn from(err: Error) -> String {
        err.to_string()
    }
}

pub fn run() -> Result<(), String> {
    let args = Args::parse();

    let content = std::fs::read(&args.input).map_err(Error::IoError)?;
    let network = match args.network {
        Some(n) => NetworkDefinition::from_str(&n).map_err(Error::ParseNetworkError)?,
        None => NetworkDefinition::simulator(),
    };

    let manifest = match manifest_decode::<TransactionManifestV1>(&content)
        .map_err(Error::DecodeError)
    {
        Ok(manifest) => AnyTransactionManifest::V1(manifest),
        Err(e) => {
            // try to decode versioned transaction
            match manifest_decode::<VersionedTransactionPayload>(&content) {
                Ok(manifest) => match manifest {
                    VersionedTransactionPayload::TransactionIntentV1(intent) => {
                        TransactionManifestV1::from_intent(&intent).into()
                    }
                    VersionedTransactionPayload::SignedTransactionIntentV1(signed_intent) => {
                        TransactionManifestV1::from_intent(&signed_intent.intent).into()
                    }
                    VersionedTransactionPayload::NotarizedTransactionV1(notarized) => {
                        TransactionManifestV1::from_intent(&notarized.signed_intent.intent).into()
                    }
                    VersionedTransactionPayload::SystemTransactionV1(system_transaction) => {
                        SystemTransactionManifestV1::from_transaction(&system_transaction).into()
                    }
                    VersionedTransactionPayload::TransactionIntentV2(intent) => {
                        TransactionManifestV2::from_intent_core(&intent.root_intent_core).into()
                    }
                    VersionedTransactionPayload::SignedTransactionIntentV2(signed_intent) => {
                        TransactionManifestV2::from_intent_core(
                            &signed_intent.root_intent.root_intent_core,
                        )
                        .into()
                    }
                    VersionedTransactionPayload::NotarizedTransactionV2(notarized) => {
                        TransactionManifestV2::from_intent_core(
                            &notarized.signed_intent.root_intent.root_intent_core,
                        )
                        .into()
                    }
                    VersionedTransactionPayload::SubintentV2(subintent) => {
                        TransactionManifestV2::from_intent_core(&subintent.intent_core).into()
                    }
                    other_type => {
                        return Err(format!(
                            "Transaction type with discriminator {} not currently supported",
                            other_type.get_discriminator()
                        ))
                    }
                },
                Err(_) => {
                    // return original error
                    return Err(e.into());
                }
            }
        }
    };

    validate_call_arguments_to_native_components_any(&manifest)
        .map_err(Error::InstructionSchemaValidationError)?;

    let decompiled = decompile_any(&manifest, &network).map_err(Error::DecompileError)?;
    std::fs::write(&args.output, &decompiled).map_err(Error::IoError)?;

    if args.export_blobs {
        let directory = args.output.parent().unwrap();
        for (blob_hash, content) in manifest.get_blobs() {
            std::fs::write(directory.join(format!("{}.blob", blob_hash)), &content)
                .map_err(Error::IoError)?;
        }
    }

    Ok(())
}
