use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct AsHex<T>(#[serde_as(as = "serde_with::hex::Hex")] T)
where
    T: AsRef<[u8]> + TryFrom<Vec<u8>>;

impl<T> core::ops::Deref for AsHex<T>
where
    T: AsRef<[u8]> + TryFrom<Vec<u8>>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> core::ops::DerefMut for AsHex<T>
where
    T: AsRef<[u8]> + TryFrom<Vec<u8>>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for AsHex<T>
where
    T: AsRef<[u8]> + TryFrom<Vec<u8>>,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> AsHex<T>
where
    T: AsRef<[u8]> + TryFrom<Vec<u8>>,
{
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Copy for AsHex<T> where T: AsRef<[u8]> + TryFrom<Vec<u8>> + Copy {}
