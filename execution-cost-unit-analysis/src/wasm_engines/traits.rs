use crate::configuration::*;
use radix_engine::blueprints::package::*;
use std::num::NonZero;

pub trait ModuleCache<T>
where
    Self: IntoDescriptor<Descriptor = Cache>,
{
    fn new(cache_size: CacheSize) -> Self;
    fn store(&self, key: CodeHash, module: T);
    fn load<F, O>(&self, key: &CodeHash, callback: F) -> Option<O>
    where
        F: FnOnce(&T) -> O;
}

pub enum CacheSize {
    Entries(NonZero<usize>),
}
