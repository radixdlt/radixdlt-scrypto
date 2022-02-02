use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use radix_engine::ledger::*;
use radix_engine::model::*;
use scrypto::types::*;

/// A file-based ledger that stores substates in a folder.
pub struct FileBasedLedger {
    root: PathBuf,
}

const PACKAGES: &str = "packages";
const COMPONENTS: &str = "components";
const LAZY_MAPS: &str = "lazy_maps";
const RESOURCE_DEFS: &str = "resource_defs";
const VAULTS: &str = "vaults";
const NFTS: &str = "nfts";
const OTHERS: &str = "others";

const FILE_EXT: &str = "sbor";

impl FileBasedLedger {
    pub fn new(root: PathBuf) -> Self {
        for folder in [
            PACKAGES,
            COMPONENTS,
            LAZY_MAPS,
            RESOURCE_DEFS,
            VAULTS,
            NFTS,
            OTHERS,
        ] {
            let mut path = root.clone();
            path.push(folder);
            if !path.exists() {
                fs::create_dir_all(&path)
                    .unwrap_or_else(|_| panic!("Failed to create dir: {:?}", path));
            }
        }

        Self { root }
    }

    pub fn with_bootstrap(root: PathBuf) -> Self {
        let mut ledger = Self::new(root);
        ledger.bootstrap();
        ledger
    }

    pub fn list_packages(&self) -> Vec<Address> {
        self.list_items(PACKAGES)
    }

    pub fn list_components(&self) -> Vec<Address> {
        self.list_items(COMPONENTS)
    }

    pub fn list_resource_defs(&self) -> Vec<Address> {
        self.list_items(RESOURCE_DEFS)
    }

    fn list_items(&self, kind: &str) -> Vec<Address> {
        let mut path = self.root.clone();
        path.push(kind);

        let mut results = Vec::new();
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap().to_str().unwrap();
                let address = Address::from_str(&name[0..name.rfind('.').unwrap()]).unwrap();
                results.push(address);
            }
        }
        results
    }

    fn get_path<T: AsRef<str>>(&self, kind: &str, name: T, ext: &str) -> PathBuf {
        let mut path = self.root.clone();
        path.push(kind);
        path.push(name.as_ref());
        path.set_extension(ext);
        path
    }

    fn write<P: AsRef<Path>, T: AsRef<[u8]>>(path: P, value: T) {
        let p = path.as_ref();

        File::create(p)
            .unwrap_or_else(|_| panic!("Failed to create file: {:?}", p))
            .write_all(value.as_ref())
            .unwrap_or_else(|_| panic!("Failed to write file: {:?}", p));
    }

    fn read<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
        let p = path.as_ref();

        if p.exists() {
            Some(fs::read(p).unwrap_or_else(|_| panic!("Failed to read file: {:?}", p)))
        } else {
            None
        }
    }

    pub fn encode<T: sbor::Encode>(v: &T) -> Vec<u8> {
        sbor::encode_with_type(Vec::with_capacity(512), v)
    }

    pub fn decode<T: sbor::Decode>(bytes: Vec<u8>) -> T {
        sbor::decode_with_type(&bytes).unwrap()
    }
}

impl Ledger for FileBasedLedger {
    fn get_resource_def(&self, address: Address) -> Option<ResourceDef> {
        Self::read(self.get_path(RESOURCE_DEFS, address.to_string(), FILE_EXT)).map(Self::decode)
    }

    fn put_resource_def(&mut self, address: Address, resource_def: ResourceDef) {
        Self::write(
            self.get_path(RESOURCE_DEFS, address.to_string(), FILE_EXT),
            Self::encode(&resource_def),
        )
    }

    fn get_package(&self, address: Address) -> Option<Package> {
        Self::read(self.get_path(PACKAGES, address.to_string(), FILE_EXT)).map(Self::decode)
    }

    fn put_package(&mut self, address: Address, package: Package) {
        Self::write(
            self.get_path(PACKAGES, address.to_string(), FILE_EXT),
            Self::encode(&package),
        )
    }

    fn get_component(&self, address: Address) -> Option<Component> {
        Self::read(self.get_path(COMPONENTS, address.to_string(), FILE_EXT)).map(Self::decode)
    }

    fn put_component(&mut self, address: Address, component: Component) {
        Self::write(
            self.get_path(COMPONENTS, address.to_string(), FILE_EXT),
            Self::encode(&component),
        )
    }

    fn get_lazy_map(&self, mid: Mid) -> Option<LazyMap> {
        Self::read(self.get_path(LAZY_MAPS, format!("{}_{}", mid.0, mid.1), FILE_EXT))
            .map(Self::decode)
    }

    fn put_lazy_map(&mut self, mid: Mid, lazy_map: LazyMap) {
        Self::write(
            self.get_path(LAZY_MAPS, format!("{}_{}", mid.0, mid.1), FILE_EXT),
            Self::encode(&lazy_map),
        )
    }

    fn get_vault(&self, vid: Vid) -> Option<Vault> {
        Self::read(self.get_path(VAULTS, format!("{}_{}", vid.0, vid.1), FILE_EXT))
            .map(Self::decode)
    }

    fn put_vault(&mut self, vid: Vid, vault: Vault) {
        Self::write(
            self.get_path(VAULTS, format!("{}_{}", vid.0, vid.1), FILE_EXT),
            Self::encode(&vault),
        )
    }

    fn get_nft(&self, resource_address: Address, id: u128) -> Option<Nft> {
        Self::read(self.get_path(NFTS, format!("{}_{}", resource_address, id), FILE_EXT))
            .map(Self::decode)
    }

    fn put_nft(&mut self, resource_address: Address, id: u128, nft: Nft) {
        Self::write(
            self.get_path(NFTS, format!("{}_{}", resource_address, id), FILE_EXT),
            Self::encode(&nft),
        )
    }

    fn get_epoch(&self) -> u64 {
        Self::read(self.get_path(OTHERS, "epoch", FILE_EXT))
            .map(Self::decode)
            .unwrap_or(0)
    }

    fn set_epoch(&mut self, epoch: u64) {
        Self::write(
            self.get_path(OTHERS, "epoch", FILE_EXT),
            Self::encode(&epoch),
        )
    }

    fn get_nonce(&mut self) -> u64 {
        Self::read(self.get_path(OTHERS, "nonce", FILE_EXT))
            .map(Self::decode)
            .unwrap_or(0)
    }

    fn increase_nonce(&mut self) {
        Self::write(
            self.get_path(OTHERS, "nonce", FILE_EXT),
            Self::encode(&(self.get_nonce() + 1)),
        )
    }
}
