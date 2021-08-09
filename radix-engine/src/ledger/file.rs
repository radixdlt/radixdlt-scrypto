use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use scrypto::types::*;

use crate::ledger::*;
use crate::model::*;

pub struct FileBasedLedger {
    root: PathBuf,
}

const BLUEPRINTS: &'static str = "blueprints";
const COMPONENTS: &'static str = "components";
const ACCOUNTS: &'static str = "accounts";
const RESOURCES: &'static str = "resources";
const BUCKETS: &'static str = "buckets";

const FILE_EXT: &'static str = "sbor";

impl FileBasedLedger {
    pub fn new(root: PathBuf) -> Self {
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

        p.parent().map(|par| {
            fs::create_dir_all(par)
                .expect(format!("Failed to create directory: {:?}", par).as_str())
        });

        File::create(p)
            .expect(format!("Failed to create file: {:?}", p).as_str())
            .write_all(value.as_ref())
            .expect(format!("Failed to write file: {:?}", p).as_str());
    }

    fn read<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
        let p = path.as_ref();

        if p.exists() {
            Some(fs::read(p).expect(format!("Failed to read file: {:?}", p).as_str()))
        } else {
            None
        }
    }

    pub fn encode<T: sbor::Encode>(v: &T) -> Vec<u8> {
        sbor::sbor_encode_with_metadata(v)
    }

    pub fn decode<'de, T: sbor::Decode>(bytes: Vec<u8>) -> T {
        sbor::sbor_decode_with_metadata(&bytes).unwrap()
    }
}

impl Ledger for FileBasedLedger {
    fn get_blueprint(&self, address: Address) -> Option<Blueprint> {
        Self::read(self.get_path(BLUEPRINTS, address.to_string(), FILE_EXT))
            .map(|v| Self::decode(v))
    }

    fn put_blueprint(&mut self, address: Address, blueprint: Blueprint) {
        Self::write(
            self.get_path(BLUEPRINTS, address.to_string(), FILE_EXT),
            Self::encode(&blueprint),
        )
    }

    fn get_resource(&self, address: Address) -> Option<Resource> {
        Self::read(self.get_path(RESOURCES, address.to_string(), FILE_EXT)).map(|v| Self::decode(v))
    }

    fn put_resource(&mut self, address: Address, resource: Resource) {
        Self::write(
            self.get_path(RESOURCES, address.to_string(), FILE_EXT),
            Self::encode(&resource),
        )
    }

    fn get_component(&self, address: Address) -> Option<Component> {
        Self::read(self.get_path(COMPONENTS, address.to_string(), FILE_EXT))
            .map(|v| Self::decode(v))
    }

    fn put_component(&mut self, address: Address, component: Component) {
        Self::write(
            self.get_path(COMPONENTS, address.to_string(), FILE_EXT),
            Self::encode(&component),
        )
    }

    fn get_account(&self, address: Address) -> Option<Account> {
        Self::read(self.get_path(ACCOUNTS, address.to_string(), FILE_EXT)).map(|v| Self::decode(v))
    }

    fn put_account(&mut self, address: Address, account: Account) {
        Self::write(
            self.get_path(ACCOUNTS, address.to_string(), FILE_EXT),
            Self::encode(&account),
        )
    }

    fn get_bucket(&self, bid: BID) -> Option<Bucket> {
        Self::read(self.get_path(BUCKETS, bid.to_string(), FILE_EXT)).map(|v| Self::decode(v))
    }

    fn put_bucket(&mut self, bid: BID, bucket: Bucket) {
        Self::write(
            self.get_path(BUCKETS, bid.to_string(), FILE_EXT),
            Self::encode(&bucket),
        )
    }
}
