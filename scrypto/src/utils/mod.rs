mod sha;

pub use sha::*;

/// A lightweight version of `unwrap()`, which does not format errors.
pub fn unwrap_light<T, E>(res: Result<T, E>) -> T {
    match res {
        Ok(v) => v,
        _ => panic!(),
    }
}
