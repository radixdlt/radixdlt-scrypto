use radix_engine::blueprints::access_controller::AccessControllerError;
use radix_engine::blueprints::resource::NonFungibleResourceManagerError;
use radix_engine::errors::*;
use radix_engine::system::kernel_modules::auth::AuthError;

// This file is supposed collect tests that help monitoring and debugging stack usage.

/* Large enums might consume stack pretty quick.
   Below macros and functions help to debug such issues.
   Our intention is to keep error enums no greater than 100 bytes.
   Our approach is to Box enum members, which make the enum so large.

   Example:
    - before - size of CallFrameError reaches almost 100
      pub enum CallFrameError {
        OffsetDoesNotExist(OffsetDoesNotExist),      <--- OffsetDoesNotExist size 64
        RENodeNotVisible(NodeId),
        RENodeNotOwned(NodeId),
        MovingLockedRENode(NodeId),
        FailedToMoveSubstateToTrack(TrackError),     <--- TrackError size 88
      }

    - after boxing largest members - size of CallFrameError reduced to 40

      pub enum CallFrameError {
        OffsetDoesNotExist(Box<OffsetDoesNotExist>),
        RENodeNotVisible(NodeId),
        RENodeNotOwned(NodeId),
        MovingLockedRENode(NodeId),
        FailedToMoveSubstateToTrack(Box<TrackError>),
      }
*/

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
