/// Converts a closure with no arguments to an [`FnOnce`].
#[inline]
pub const fn fn_once<F: FnOnce() -> O, O>(f: F) -> F {
    f
}

#[cfg(feature = "std")]
pub fn catch_unwind_system_panic_transformer<T>(
    args: Result<Result<T, crate::errors::RuntimeError>, Box<dyn std::any::Any + Send + 'static>>,
) -> Result<T, crate::errors::RuntimeError> {
    match args {
        Ok(rtn) => rtn,
        Err(cause) => {
            let message = if let Some(s) = cause.downcast_ref::<&'static str>() {
                (*s).to_string()
            } else if let Some(s) = cause.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic!".to_string()
            };
            Err(crate::errors::RuntimeError::SystemError(
                crate::errors::SystemError::SystemPanic(message),
            ))
        }
    }
}
