#[macro_export]
macro_rules! borrow_package {
    ($address:expr) => {
        $crate::component::BorrowedPackage($address.clone())
    };
}
