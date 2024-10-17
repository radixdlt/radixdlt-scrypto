use crate::prelude::*;

/// Radix transaction manifest compiler
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, name = "rtmc")]
pub struct Args {
    /// Path to the output file
    #[clap(short, long)]
    output: PathBuf,

    /// Network to Use [Simulator | Alphanet | Mainnet]
    #[clap(short, long)]
    network: Option<String>,

    /// The paths to blobs
    #[clap(short, long, multiple = true)]
    blobs: Option<Vec<String>>,

    /// Input file
    #[clap(required = true)]
    input: PathBuf,

    /// The manifest type [V1 | SystemV1 | V2 | SubintentV2], defaults to V2
    #[clap(short, long)]
    kind: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    EncodeError(sbor::EncodeError),
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

    let content = std::fs::read_to_string(&args.input).map_err(Error::IoError)?;
    let network = match args.network {
        Some(n) => NetworkDefinition::from_str(&n).map_err(Error::ParseNetworkError)?,
        None => NetworkDefinition::simulator(),
    };
    let mut blobs = Vec::new();
    if let Some(paths) = args.blobs {
        for path in paths {
            blobs.push(std::fs::read(path).map_err(Error::IoError)?);
        }
    }

    let manifest_kind = ManifestKind::parse_or_latest(args.kind.as_ref().map(|x| x.as_str()))?;
    let manifest = compile_any_manifest_with_pretty_error(
        &content,
        manifest_kind,
        &network,
        BlobProvider::new_with_blobs(blobs),
        CompileErrorDiagnosticsStyle::TextTerminalColors,
    )?;

    manifest
        .validate(ValidationRuleset::all())
        .map_err(Error::ManifestValidationError)?;

    validate_call_arguments_to_native_components(&manifest)
        .map_err(Error::InstructionSchemaValidationError)?;

    write_ensuring_folder_exists(
        args.output,
        manifest_encode(&manifest).map_err(Error::EncodeError)?,
    )
    .map_err(Error::IoError)?;

    Ok(())
}
