use super::module::*;
use crate::wasm_engines::traits::*;
use radix_common::prelude::*;
use radix_engine_interface::blueprints::package::CodeHash;
use std::sync::*;

#[derive(Clone, Debug, Default)]
pub struct NoCache<Module>(PhantomData<Module>);

impl<Module> ModuleCache<Module> for NoCache<Module> {
    #[inline]
    fn new(_: CacheSize) -> Self {
        Self(Default::default())
    }

    #[inline]
    fn store(&self, _: CodeHash, _: Module) {}

    #[inline]
    fn load<F, O>(&self, _: &CodeHash, _: F) -> Option<O>
    where
        F: FnOnce(&Module) -> O,
    {
        None
    }
}

pub struct LruModuleCache<Module>(RefCell<lru::LruCache<CodeHash, Arc<Module>>>);

impl<Module> ModuleCache<Module> for LruModuleCache<Module> {
    fn new(CacheSize::Entries(cache_size): CacheSize) -> Self {
        Self(RefCell::new(lru::LruCache::new(cache_size)))
    }

    fn store(&self, key: CodeHash, module: Module) {
        self.0.borrow_mut().put(key, Arc::new(module));
    }

    fn load<F, O>(&self, key: &CodeHash, callback: F) -> Option<O>
    where
        F: FnOnce(&Module) -> O,
    {
        self.0.borrow_mut().get(key).map(|item| callback(item))
    }
}

pub struct MokaModuleCache<Module>(moka::sync::Cache<CodeHash, Arc<Module>>);

impl<Module> ModuleCache<Module> for MokaModuleCache<Module>
where
    Module: Send + Sync + 'static,
{
    fn new(CacheSize::Entries(cache_size): CacheSize) -> Self {
        Self(moka::sync::Cache::new(cache_size.get() as u64))
    }

    fn store(&self, key: CodeHash, module: Module) {
        self.0.insert(key, Arc::new(module));
    }

    fn load<F, O>(&self, key: &CodeHash, callback: F) -> Option<O>
    where
        F: FnOnce(&Module) -> O,
    {
        self.0.get(key).map(|item| callback(&item))
    }
}
