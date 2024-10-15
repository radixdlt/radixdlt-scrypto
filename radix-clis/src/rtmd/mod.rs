use crate::prelude::*;

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
    ManifestValidationError(ManifestValidationError),
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

    let manifest = AnyManifest::attempt_decode_from_arbitrary_payload(&content)?;

    manifest
        .validate(ValidationRuleset::all())
        .map_err(Error::ManifestValidationError)?;

    validate_call_arguments_to_native_components(&manifest)
        .map_err(Error::InstructionSchemaValidationError)?;

    let decompiled = decompile_any(&manifest, &network).map_err(Error::DecompileError)?;

    write_ensuring_folder_exists(&args.output, &decompiled).map_err(Error::IoError)?;

    if args.export_blobs {
        let directory = args.output.parent().unwrap();
        for (blob_hash, content) in manifest.get_blobs() {
            std::fs::write(directory.join(format!("{}.blob", blob_hash)), &content)
                .map_err(Error::IoError)?;
        }
    }

    Ok(())
}
