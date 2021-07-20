#[macro_export]
macro_rules! call_kernel {
    ($operation: expr, $input: expr) => {
        unsafe {
            // 1. serialize the input
            let input_bytes = crate::buffer::bincode_encode(&$input);

            // 2. make a kernel call
            let output_ptr =
                crate::kernel::radix_kernel($operation, input_bytes.as_ptr(), input_bytes.len());

            // 3. copy and release the buffer (allocated by kernel)
            let output_bytes = crate::kernel::radix_copy(output_ptr);
            crate::kernel::radix_free(output_ptr);

            // 4. deserialize the output
            let output = crate::buffer::bincode_decode(&output_bytes);
            output
        }
    };
}

#[macro_export]
macro_rules! call_blueprint {
    ($rtn_type: ty, $blueprint: expr, $component: expr, $method: expr) => {
        {
            extern crate alloc;
            let rtn = crate::constructs::Blueprint::call(&$blueprint, $component, $method, alloc::vec::Vec::new());
            crate::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };

    ($rtn_type: ty, $blueprint: expr, $component: expr, $method: expr, $($args: expr),+) => {
        {
            extern crate alloc;
            let mut args = alloc::vec::Vec::new();
            $(args.push(crate::buffer::radix_encode(&$args));)+
            let rtn = crate::constructs::Blueprint::call(&$blueprint, $component, $method, args);
            crate::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };
}

#[macro_export]
macro_rules! call_component {
    ($rtn_type: ty, $component: expr, $method: expr) => {
        {
            extern crate alloc;
            let rtn = crate::constructs::Component::call(&$component, $method, alloc::vec::Vec::new());
            crate::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };

    ($rtn_type: ty, $component: expr, $method: expr, $($args: expr),+) => {
        {
            extern crate alloc;
            let mut args = alloc::vec::Vec::new();
            $(args.push(crate::buffer::radix_encode(&$args));)+
            let rtn = crate::constructs::Component::call(&$component, $method, args);
            crate::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };
}

#[macro_export]
macro_rules! error {
    ($msg: expr) => {
        {
            extern crate alloc;
            crate::constructs::Logger::error(alloc::format!($msg));
        }
    };
    ($msg: expr, $($args: expr),+) => {
        {
            extern crate alloc;
            crate::constructs::Logger::error(alloc::format!($msg, $($args),+));
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($msg: expr) => {
        {
            extern crate alloc;
            crate::constructs::Logger::warn(alloc::format!($msg));
        }
    };
    ($msg: expr, $($args: expr),+) => {
        {
            extern crate alloc;
            crate::constructs::Logger::warn(alloc::format!($msg, $($args),+));
        }
    };
}

#[macro_export]
macro_rules! info {
    ($msg: expr) => {
        {
            extern crate alloc;
            crate::constructs::Logger::info(alloc::format!($msg));
        }
    };
    ($msg: expr, $($args: expr),+) => {
        {
            extern crate alloc;
            crate::constructs::Logger::info(alloc::format!($msg, $($args),+));
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($msg: expr) => {
        {
            extern crate alloc;
            crate::constructs::Logger::debug(alloc::format!($msg));
        }
    };
    ($msg: expr, $($args: expr),+) => {
        {
            extern crate alloc;
            crate::constructs::Logger::debug(alloc::format!($msg, $($args),+));
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($msg: expr) => {
        {
            extern crate alloc;
            crate::constructs::Logger::trace(alloc::format!($msg));
        }
    };
    ($msg: expr, $($args: expr),+) => {
        {
            extern crate alloc;
            crate::constructs::Logger::trace(alloc::format!($msg, $($args),+));
        }
    };
}
