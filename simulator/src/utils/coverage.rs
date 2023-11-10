#[derive(Debug)]
pub enum CoverageError {
    MissingWasm32Target,
    IncorrectRustVersion,
    MissingLLVM,
    NoProfrawFiles,
    ProfdataMergeFailed,
    ClangFailed,
    LlvmCovFailed,
}
