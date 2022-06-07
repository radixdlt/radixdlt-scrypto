use scrypto::crypto::Hash;

pub trait IntentHashManager {
    fn allows(&self, hash: &Hash) -> bool;
}

pub trait EpochManager {
    fn current_epoch(&self) -> u64;
}
