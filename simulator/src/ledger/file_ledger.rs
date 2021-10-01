use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

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

const FILE_EXT: &str = "sbor";

impl FileBasedLedger {
    pub fn new(root: PathBuf) -> Self {
        for folder in [PACKAGES, COMPONENTS, LAZY_MAPS, RESOURCE_DEFS, VAULTS] {
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

    fn get_lazy_map(&self, mid: MID) -> Option<LazyMap> {
        Self::read(self.get_path(LAZY_MAPS, mid.to_string(), FILE_EXT)).map(Self::decode)
    }

    fn put_lazy_map(&mut self, mid: MID, lazy_map: LazyMap) {
        Self::write(
            self.get_path(LAZY_MAPS, mid.to_string(), FILE_EXT),
            Self::encode(&lazy_map),
        )
    }

    fn get_vault(&self, vid: VID) -> Option<Vault> {
        Self::read(self.get_path(VAULTS, vid.to_string(), FILE_EXT)).map(Self::decode)
    }

    fn put_vault(&mut self, vid: VID, vault: Vault) {
        Self::write(
            self.get_path(VAULTS, vid.to_string(), FILE_EXT),
            Self::encode(&vault),
        )
    }
}
