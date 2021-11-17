/// Unwrap a result and abort if it's a failure. Different from the normal
/// unwrap, this function does not dump the error details (for better performance).
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
        let payload = info.payload().downcast_ref::<&str>().unwrap_or(&"");
        let location = if let Some(l) = info.location() {
            format!("{}:{}:{}", l.file(), l.line(), l.column())
        } else {
            "Unknown location".to_owned()
        };

        crate::core::Logger::error(crate::rust::format!(
            "Panicked at '{}', {}",
            payload,
            location
        ));
    }));
}
