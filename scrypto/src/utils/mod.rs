mod sha;

pub use sha::*;

/// Unwrap a result and panic (with no debug info) on error.
pub fn unwrap_or_panic<T, E>(res: Result<T, E>) -> T {
    match res {
        Ok(v) => v,
        _ => panic!(),
    }
}
