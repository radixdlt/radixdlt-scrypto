use super::module::*;
use crate::wasm_engines::traits::*;
use radix_common::prelude::*;
use radix_engine_interface::blueprints::package::CodeHash;
use std::sync::*;

#[derive(Clone, Debug, Default)]
pub struct WasmiNoCache;

impl ModuleCache<WasmiModule> for WasmiNoCache {
    #[inline]
    fn new(_: CacheSize) -> Self {
        Self
    }

    #[inline]
    fn store(&self, _: CodeHash, _: WasmiModule) {}

    #[inline]
    fn load<F, O>(&self, _: &CodeHash, _: F) -> Option<O>
    where
        F: FnOnce(&WasmiModule) -> O,
    {
        None
    }
}

pub struct WasmiLruModuleCache(RefCell<lru::LruCache<CodeHash, Arc<WasmiModule>>>);

impl ModuleCache<WasmiModule> for WasmiLruModuleCache {
    fn new(CacheSize::Entries(cache_size): CacheSize) -> Self {
        Self(RefCell::new(lru::LruCache::new(cache_size)))
    }

    fn store(&self, key: CodeHash, module: WasmiModule) {
        self.0.borrow_mut().put(key, Arc::new(module));
    }

    fn load<F, O>(&self, key: &CodeHash, callback: F) -> Option<O>
    where
        F: FnOnce(&WasmiModule) -> O,
    {
        self.0.borrow_mut().get(key).map(|item| callback(item))
    }
}

pub struct WasmiMokaModuleCache(moka::sync::Cache<CodeHash, Arc<WasmiModule>>);

impl ModuleCache<WasmiModule> for WasmiMokaModuleCache {
    fn new(CacheSize::Entries(cache_size): CacheSize) -> Self {
        Self(moka::sync::Cache::new(cache_size.get() as u64))
    }

    fn store(&self, key: CodeHash, module: WasmiModule) {
        self.0.insert(key, Arc::new(module));
    }

    fn load<F, O>(&self, key: &CodeHash, callback: F) -> Option<O>
    where
        F: FnOnce(&WasmiModule) -> O,
    {
        self.0.get(key).map(|item| callback(&item))
    }
}
