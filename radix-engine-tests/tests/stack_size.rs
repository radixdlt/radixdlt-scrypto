use radix_engine::blueprints::access_controller::AccessControllerError;
use radix_engine::blueprints::resource::NonFungibleResourceManagerError;
use radix_engine::errors::*;
use radix_engine::system::kernel_modules::auth::AuthError;

macro_rules! check_size {
    ($type:ident, $size:expr) => {
        assert!(
            std::mem::size_of::<$type>() < $size,
            "Size of {} {} greater than {} bytes",
            stringify!($type),
            std::mem::size_of::<$type>(),
            $size
        )
    };
}
macro_rules! print_size {
    ($type:ident) => {
        println!(
            "{:35} : {:-3}",
            stringify!($type),
            std::mem::size_of::<$type>()
        );
    };
}

#[test]
fn test_error_enum_sizes() {
    // Large enums might consume stack pretty quick.
    println!("Popular error enums sizes:");
    print_size!(RuntimeError);
    print_size!(KernelError);
    print_size!(CallFrameError);
    print_size!(SystemError);
    print_size!(InterpreterError);
    print_size!(ModuleError);
    print_size!(ApplicationError);
    print_size!(AuthError);
    print_size!(AccessControllerError);
    print_size!(NonFungibleResourceManagerError);

    check_size!(RuntimeError, 100);
    check_size!(KernelError, 100);
    check_size!(CallFrameError, 100);
    check_size!(SystemError, 100);
    check_size!(InterpreterError, 100);
    check_size!(ModuleError, 100);
    check_size!(ApplicationError, 100);
    check_size!(AuthError, 100);
    check_size!(AccessControllerError, 100);
    check_size!(NonFungibleResourceManagerError, 100);
}
