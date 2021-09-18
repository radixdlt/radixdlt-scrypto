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
const STORAGES: &str = "storages";
const RESOURCES: &str = "resources";
const VAULTS: &str = "vaults";

const FILE_EXT: &str = "sbor";

impl FileBasedLedger {
    pub fn new(root: PathBuf) -> Self {
        for folder in [PACKAGES, COMPONENTS, STORAGES, RESOURCES, VAULTS] {
            let mut path = root.clone();
            path.push(folder);
            if !path.exists() {
                fs::create_dir_all(&path)
                    .unwrap_or_else(|_| panic!("Failed to create dir: {:?}", path));
            }
        }

        Self { root }
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
    fn get_package(&self, address: Address) -> Option<Package> {
        Self::read(self.get_path(PACKAGES, address.to_string(), FILE_EXT)).map(Self::decode)
    }

    fn put_package(&mut self, address: Address, package: Package) {
        Self::write(
            self.get_path(PACKAGES, address.to_string(), FILE_EXT),
            Self::encode(&package),
        )
    }

    fn get_resource(&self, address: Address) -> Option<Resource> {
        Self::read(self.get_path(RESOURCES, address.to_string(), FILE_EXT)).map(Self::decode)
    }

    fn put_resource(&mut self, address: Address, resource: Resource) {
        Self::write(
            self.get_path(RESOURCES, address.to_string(), FILE_EXT),
            Self::encode(&resource),
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

    fn get_storage(&self, sid: SID) -> Option<Storage> {
        Self::read(self.get_path(STORAGES, sid.to_string(), FILE_EXT)).map(Self::decode)
    }

    fn put_storage(&mut self, sid: SID, storage: Storage) {
        Self::write(
            self.get_path(STORAGES, sid.to_string(), FILE_EXT),
            Self::encode(&storage),
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
