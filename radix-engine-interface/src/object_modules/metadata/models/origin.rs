use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// This regular expressions only cover the most commonly used types of Origins.
    ///
    /// Based on https://en.wikipedia.org/wiki/URL#/media/File:URI_syntax_diagram.svg
    ///
    static ref ORIGIN_REGEX: Regex = Regex::new(
        concat!(
            // 1. Start
            "^",
            // 2. Schema, http or https only
            "https?",
            // 3. ://
            ":\\/\\/",
            // 4. Userinfo, not allowed
            // 5. Host, ip address or host name
            //    From https://stackoverflow.com/questions/106179/regular-expression-to-match-dns-hostname-or-ip-address
            "(",
                "((([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5]))",
                "|",
                "((([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\\-]*[a-zA-Z0-9])\\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\\-]*[A-Za-z0-9]))",
            ")",
            // 6. Port number, optional
            //    From https://stackoverflow.com/questions/12968093/regex-to-validate-port-number
            "(:([1-9][0-9]{0,3}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5]))?",
            // 7. End
            "$"
        )
    ).unwrap();
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(
    Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoCategorize, ScryptoEncode, ScryptoDecode,
)]
#[sbor(transparent)]
pub struct UncheckedOrigin(pub String);

impl Describe<ScryptoCustomTypeKind> for UncheckedOrigin {
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(well_known_scrypto_custom_types::ORIGIN_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::origin_type_data()
    }
}

impl UncheckedOrigin {
    pub fn of(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub struct CheckedOrigin(String);

impl CheckedOrigin {
    pub fn of(value: impl AsRef<str>) -> Option<Self> {
        let s = value.as_ref();
        if s.len() <= MAX_ORIGIN_LENGTH && ORIGIN_REGEX.is_match(s) {
            Some(Self(s.to_owned()))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<UncheckedOrigin> for CheckedOrigin {
    type Error = ();

    fn try_from(value: UncheckedOrigin) -> Result<Self, Self::Error> {
        CheckedOrigin::of(value.as_str()).ok_or(())
    }
}

impl From<CheckedOrigin> for UncheckedOrigin {
    fn from(value: CheckedOrigin) -> Self {
        Self(value.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin() {
        assert!(CheckedOrigin::of("https://www.google.com").is_some());
        assert!(CheckedOrigin::of("http://gooooooooooooooooooooooooooooooogle.com:8888").is_some());
        assert!(CheckedOrigin::of("https://66.123.1.255:9").is_some());
        assert!(CheckedOrigin::of("https://www.google.com/").is_none());
    }
}
