use crate::builder::TransactionManifestV1;
use crate::manifest::*;
use radix_engine_interface::address::AddressBech32Decoder;
use radix_engine_interface::network::NetworkDefinition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    LexerError(lexer::LexerError),
    ParserError(parser::ParserError),
    GeneratorError(generator::GeneratorError),
}

pub fn compile<B>(
    s: &str,
    network: &NetworkDefinition,
    blobs: B,
) -> Result<TransactionManifestV1, CompileError>
where
    B: IsBlobProvider,
{
    let address_bech32_decoder = AddressBech32Decoder::new(network);

    let tokens = lexer::tokenize(s).map_err(CompileError::LexerError)?;
    let instructions = parser::Parser::new(tokens, parser::PARSER_MAX_DEPTH)
        .parse_manifest()
        .map_err(CompileError::ParserError)?;
    generator::generate_manifest(&instructions, &address_bech32_decoder, blobs)
        .map_err(CompileError::GeneratorError)
}
