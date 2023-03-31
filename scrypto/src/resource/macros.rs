#[macro_export]
macro_rules! borrow_resource_manager {
    ($address:expr) => {
        $crate::resource::ResourceManager($address.clone())
    };
}
