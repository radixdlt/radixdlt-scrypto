use crate::rust::vec::Vec;

/// Copies a slice to a fixed-sized array.
pub fn copy_u8_array<const N: usize>(slice: &[u8]) -> [u8; N] {
    if slice.len() == N {
        let mut bytes = [0u8; N];
        bytes.copy_from_slice(&slice[0..N]);
        bytes
    } else {
        panic!("Invalid length");
    }
}

/// Combines a u8 with a u8 slice.
pub fn combine(ty: u8, bytes: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(1 + bytes.len());
    v.push(ty);
    v.extend(bytes);
    v
}

/// Sets up panic hook.
pub fn set_up_panic_hook() {
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
