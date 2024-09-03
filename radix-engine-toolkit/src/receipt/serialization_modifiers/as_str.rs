use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use core::fmt::Display;
use core::str::FromStr;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct AsStr<T>(#[serde_as(as = "serde_with::DisplayFromStr")] T)
where
    T: Display + FromStr,
    <T as FromStr>::Err: Display;

impl<T> core::ops::Deref for AsStr<T>
where
    T: Display + FromStr,
    <T as FromStr>::Err: Display,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> core::ops::DerefMut for AsStr<T>
where
    T: Display + FromStr,
    <T as FromStr>::Err: Display,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for AsStr<T>
where
    T: Display + FromStr,
    <T as FromStr>::Err: Display,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> AsStr<T>
where
    T: Display + FromStr,
    <T as FromStr>::Err: Display,
{
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Copy for AsStr<T>
where
    T: Display + FromStr + Copy,
    <T as FromStr>::Err: Display,
{
}
