use crate::core::Logger;
use crate::rust::borrow::ToOwned;

/// Unwrap a result and abort if it's a failure. Different from the normal
/// unwrap, this function does not dump the error details (for better performance).
pub fn scrypto_unwrap<T, E>(res: Result<T, E>) -> T {
    match res {
        Ok(v) => v,
        _ => scrypto_abort("Result is a failure"),
    }
}

/// Dumps an error message and abort.
pub fn scrypto_abort<S: AsRef<str>>(msg: S) -> ! {
    Logger::error(msg.as_ref().to_owned());
    panic!();
}
