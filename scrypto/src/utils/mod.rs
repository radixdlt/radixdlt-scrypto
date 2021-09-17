mod sha;

pub use sha::*;

/// A lightweight `unwrap()` which does not format the error.
pub fn unwrap_light<T, E>(res: Result<T, E>) -> T {
    match res {
        Ok(v) => v,
        _ => panic!(),
    }
}
