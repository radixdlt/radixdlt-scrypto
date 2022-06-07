use crate::manifest::*;

use crate::model::TransactionManifest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    LexerError(lexer::LexerError),
    ParserError(parser::ParserError),
    GeneratorError(generator::GeneratorError),
}

pub fn compile(s: &str) -> Result<TransactionManifest, CompileError> {
    let tokens = lexer::tokenize(s).map_err(CompileError::LexerError)?;
    let instructions = parser::Parser::new(tokens)
        .parse_manifest()
        .map_err(CompileError::ParserError)?;
    generator::generate_manifest(&instructions).map_err(CompileError::GeneratorError)
}
