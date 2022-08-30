use scrypto::address::Bech32Decoder;
use scrypto::core::NetworkDefinition;

use crate::manifest::*;
use crate::model::TransactionManifest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    LexerError(lexer::LexerError),
    ParserError(parser::ParserError),
    GeneratorError(generator::GeneratorError),
}

pub fn compile<T: BlobLoader>(
    s: &str,
    network: &NetworkDefinition,
    blob_loader: &T,
) -> Result<TransactionManifest, CompileError> {
    let bech32_decoder = Bech32Decoder::new(network);

    let tokens = lexer::tokenize(s).map_err(CompileError::LexerError)?;
    let instructions = parser::Parser::new(tokens)
        .parse_manifest()
        .map_err(CompileError::ParserError)?;
    generator::generate_manifest(&instructions, &bech32_decoder, blob_loader)
        .map_err(CompileError::GeneratorError)
}
