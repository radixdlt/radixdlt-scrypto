/// Call a method of a blueprint.
#[macro_export]
macro_rules! call_blueprint {
    ($rtn_type: ty, $blueprint: expr, $component: expr, $method: expr $(,)?) => {
        {
            extern crate alloc;
            let rtn = scrypto::constructs::Blueprint::call(&$blueprint, $component, $method, alloc::vec::Vec::new());
            scrypto::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };

    ($rtn_type: ty, $blueprint: expr, $component: expr, $method: expr, $($args: expr),+ $(,)?) => {
        {
            extern crate alloc;
            let mut args = alloc::vec::Vec::new();
            $(args.push(scrypto::buffer::radix_encode(&$args));)+
            let rtn = scrypto::constructs::Blueprint::call(&$blueprint, $component, $method, args);
            scrypto::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };
}

/// Call a method of a component.
#[macro_export]
macro_rules! call_component {
    ($rtn_type: ty, $component: expr, $method: expr $(,)?) => {
        {
            extern crate alloc;
            let rtn = scrypto::constructs::Component::call(&$component, $method, alloc::vec::Vec::new());
            scrypto::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };

    ($rtn_type: ty, $component: expr, $method: expr, $($args: expr),+ $(,)?) => {
        {
            extern crate alloc;
            let mut args = alloc::vec::Vec::new();
            $(args.push(scrypto::buffer::radix_encode(&$args));)+
            let rtn = scrypto::constructs::Component::call(&$component, $method, args);
            scrypto::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };
}

/// Log an `ERROR` message.
#[macro_export]
macro_rules! error {
    ($($args: expr),+) => {
        extern crate alloc;
        scrypto::constructs::Logger::log(scrypto::constructs::LogLevel::Error, alloc::format!($($args),+));
    };
}

/// Log a `WARN` message.
#[macro_export]
macro_rules! warn {
    ($($args: expr),+) => {
        extern crate alloc;
        scrypto::constructs::Logger::log(scrypto::constructs::LogLevel::Warn, alloc::format!($($args),+));
    };
}

/// Log an `INFO` message.
#[macro_export]
macro_rules! info {
    ($($args: expr),+) => {
        extern crate alloc;
        scrypto::constructs::Logger::log(scrypto::constructs::LogLevel::Info, alloc::format!($($args),+));
    };
}

/// Log a `DEBUG` message.
#[macro_export]
macro_rules! debug {
    ($($args: expr),+) => {
        extern crate alloc;
        scrypto::constructs::Logger::log(scrypto::constructs::LogLevel::Debug, alloc::format!($($args),+));
    };
}

/// Log a `TRACE` message.
#[macro_export]
macro_rules! trace {
    ($($args: expr),+) => {
        extern crate alloc;
        scrypto::constructs::Logger::log(scrypto::constructs::LogLevel::Trace, alloc::format!($($args),+));
    };
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec::Vec;

    use scrypto::abi::*;
    use scrypto::buffer::*;
    use scrypto::kernel::*;

    #[no_mangle]
    pub extern "C" fn radix_kernel(_op: u32, input_ptr: *const u8, input_len: usize) -> *mut u8 {
        let mut input_bytes = Vec::<u8>::with_capacity(input_len);
        unsafe {
            core::ptr::copy(input_ptr, input_bytes.as_mut_ptr(), input_len);
            input_bytes.set_len(input_len);
        }
        let input: EmitLogInput = bincode_decode(&input_bytes);

        assert_eq!("INFO", input.level);
        assert_eq!("Hello, World!", input.message);

        let output = EmitLogOutput {};
        let output_bytes = bincode_encode(&output);
        let output_ptr = radix_alloc(output_bytes.len());
        unsafe {
            core::ptr::copy(output_bytes.as_ptr(), output_ptr, output_bytes.len());
        }
        output_ptr
    }

    #[test]
    fn test_system_call() {
        info!("Hello, {}", "World!");
    }
}
