use super::module::*;
use crate::wasm_engines::traits::*;
use radix_common::prelude::*;
use radix_engine_interface::blueprints::package::CodeHash;
use std::sync::*;

#[derive(Clone, Debug, Default)]
pub struct WasmerNoCache;

impl ModuleCache<WasmerModule> for WasmerNoCache {
    #[inline]
    fn new(_: CacheSize) -> Self {
        Self
    }

    #[inline]
    fn store(&self, _: CodeHash, _: WasmerModule) {}

    #[inline]
    fn load<F, O>(&self, _: &CodeHash, _: F) -> Option<O>
    where
        F: FnOnce(&WasmerModule) -> O,
    {
        None
    }
}

pub struct WasmerLruModuleCache(RefCell<lru::LruCache<CodeHash, Arc<WasmerModule>>>);

impl ModuleCache<WasmerModule> for WasmerLruModuleCache {
    fn new(CacheSize::Entries(cache_size): CacheSize) -> Self {
        Self(RefCell::new(lru::LruCache::new(cache_size)))
    }

    fn store(&self, key: CodeHash, module: WasmerModule) {
        self.0.borrow_mut().put(key, Arc::new(module));
    }

    fn load<F, O>(&self, key: &CodeHash, callback: F) -> Option<O>
    where
        F: FnOnce(&WasmerModule) -> O,
    {
        self.0.borrow_mut().get(key).map(|item| callback(item))
    }
}

pub struct WasmerMokaModuleCache(moka::sync::Cache<CodeHash, Arc<WasmerModule>>);

impl ModuleCache<WasmerModule> for WasmerMokaModuleCache {
    fn new(CacheSize::Entries(cache_size): CacheSize) -> Self {
        Self(moka::sync::Cache::new(cache_size.get() as u64))
    }

    fn store(&self, key: CodeHash, module: WasmerModule) {
        self.0.insert(key, Arc::new(module));
    }

    fn load<F, O>(&self, key: &CodeHash, callback: F) -> Option<O>
    where
        F: FnOnce(&WasmerModule) -> O,
    {
        self.0.get(key).map(|item| callback(&item))
    }
}
