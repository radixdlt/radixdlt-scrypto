pub mod ast;
pub mod decompiler;
pub mod generator;
pub mod lexer;
pub mod parser;

pub use decompiler::{decompile, DecompileError};

use radix_engine::model::Transaction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    LexerError(lexer::LexerError),
    ParserError(parser::ParserError),
    GeneratorError(generator::GeneratorError),
}

pub fn compile(s: &str) -> Result<Transaction, CompileError> {
    let tokens = lexer::tokenize(s).map_err(CompileError::LexerError)?;
    let ast = parser::Parser::new(tokens)
        .parse_transaction()
        .map_err(CompileError::ParserError)?;
    generator::generate_transaction(&ast).map_err(CompileError::GeneratorError)
}
