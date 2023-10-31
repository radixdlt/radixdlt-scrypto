#[derive(Debug)]
pub enum CoverageError {
    IncorrectRustVersion,
    MissingLLVM,
    NoProfrawFiles,
    ProfdataMergeFailed,
    ClangFailed,
    LlvmCovFailed,
}
