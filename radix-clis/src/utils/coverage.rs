#[derive(Debug)]
pub enum CoverageError {
    MissingWasm32Target,
    IncorrectRustVersion,
    MissingLLVM,
    MissingRustLld,
    NoProfrawFiles,
    ProfdataMergeFailed,
    ClangFailed,
    RustLldFailed,
    LlvmCovFailed,
}
