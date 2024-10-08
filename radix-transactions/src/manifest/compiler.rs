use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    LexerError(lexer::LexerError),
    ParserError(parser::ParserError),
    GeneratorError(generator::GeneratorError),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CompileErrorDiagnosticsStyle {
    PlainText,
    TextTerminalColors,
}

pub fn compile_error_diagnostics(
    s: &str,
    err: CompileError,
    style: CompileErrorDiagnosticsStyle,
) -> String {
    match err {
        CompileError::LexerError(err) => lexer::lexer_error_diagnostics(s, err, style),
        CompileError::ParserError(err) => parser::parser_error_diagnostics(s, err, style),
        CompileError::GeneratorError(err) => generator::generator_error_diagnostics(s, err, style),
    }
}

// Kept for backwards compatibility of downstream clients / integrators
pub use compile_manifest_v1 as compile;

pub fn compile_manifest_v1(
    manifest_string: &str,
    network: &NetworkDefinition,
    blobs: impl IsBlobProvider,
) -> Result<TransactionManifestV1, CompileError> {
    compile_manifest(manifest_string, network, blobs)
}

pub fn compile_any_manifest_with_pretty_error(
    manifest_string: &str,
    manifest_kind: ManifestKind,
    network: &NetworkDefinition,
    blobs: impl IsBlobProvider,
    error_style: CompileErrorDiagnosticsStyle,
) -> Result<AnyManifest, String> {
    compile_any_manifest(manifest_string, manifest_kind, network, blobs)
        .map_err(|err| compile_error_diagnostics(manifest_string, err, error_style))
}

pub fn compile_manifest_with_pretty_error<M: BuildableManifest>(
    manifest_string: &str,
    network: &NetworkDefinition,
    blobs: impl IsBlobProvider,
    error_style: CompileErrorDiagnosticsStyle,
) -> Result<M, String> {
    compile_manifest(manifest_string, network, blobs)
        .map_err(|err| compile_error_diagnostics(manifest_string, err, error_style))
}

pub fn compile_any_manifest(
    manifest_string: &str,
    manifest_kind: ManifestKind,
    network: &NetworkDefinition,
    blobs: impl IsBlobProvider,
) -> Result<AnyManifest, CompileError> {
    let manifest = match manifest_kind {
        ManifestKind::V1 => {
            compile_manifest::<TransactionManifestV1>(manifest_string, network, blobs)?.into()
        }
        ManifestKind::SystemV1 => {
            compile_manifest::<SystemTransactionManifestV1>(manifest_string, network, blobs)?.into()
        }
        ManifestKind::V2 => {
            compile_manifest::<TransactionManifestV2>(manifest_string, network, blobs)?.into()
        }
        ManifestKind::SubintentV2 => {
            compile_manifest::<SubintentManifestV2>(manifest_string, network, blobs)?.into()
        }
    };

    Ok(manifest)
}

pub fn compile_manifest<M: BuildableManifest>(
    s: &str,
    network: &NetworkDefinition,
    blobs: impl IsBlobProvider,
) -> Result<M, CompileError> {
    let address_bech32_decoder = AddressBech32Decoder::new(network);
    let transaction_bech32_decoder = TransactionHashBech32Decoder::new(network);

    let tokens = lexer::tokenize(s).map_err(CompileError::LexerError)?;
    let instructions = parser::Parser::new(tokens, parser::PARSER_MAX_DEPTH)
        .map_err(CompileError::ParserError)?
        .parse_manifest()
        .map_err(CompileError::ParserError)?;
    generator::generate_manifest(
        &instructions,
        &address_bech32_decoder,
        &transaction_bech32_decoder,
        blobs,
    )
    .map_err(CompileError::GeneratorError)
}
