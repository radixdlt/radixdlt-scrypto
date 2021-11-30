/// Unwrap a result and abort if it's a failure.
///
/// Different from the normal `Result::unwrap()`, this function does not dump
/// the error details.
pub fn scrypto_unwrap<T, E>(res: Result<T, E>) -> T {
    match res {
        Ok(v) => v,
        _ => panic!("Result is an error"),
    }
}

/// Set up panic hook.
pub fn scrypto_setup_panic_hook() {
    #[cfg(not(feature = "alloc"))]
    std::panic::set_hook(Box::new(|info| {
        // parse message
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(ToString::to_string)
            .or(info
                .payload()
                .downcast_ref::<String>()
                .map(ToString::to_string))
            .unwrap_or(String::new());

        // parse location
        let location = if let Some(l) = info.location() {
            format!("{}:{}:{}", l.file(), l.line(), l.column())
        } else {
            "<unknown>".to_owned()
        };

        crate::core::Logger::error(crate::rust::format!(
            "Panicked at '{}', {}",
            payload,
            location
        ));
    }));
}
