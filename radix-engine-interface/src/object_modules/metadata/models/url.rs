use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// This regular expressions only cover the most commonly used types of URLs.
    ///
    /// Based on https://en.wikipedia.org/wiki/URL#/media/File:URI_syntax_diagram.svg
    ///
    static ref URL_REGEX: Regex = Regex::new(
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
            // 7. Path, optional
            //    * -+
            //    * a-zA-Z0-9
            //    * ()
            //    * []
            //    * @ : % _ . ~ & =
            "(\\/[-\\+a-zA-Z0-9\\(\\)\\[\\]@:%_.~&=]*)*",
            // 8. Query, optional
            //    * -+
            //    * a-zA-Z0-9
            //    * ()
            //    * []
            //    * @ : % _ . ~ & =
            //    * /
            "(\\?[-\\+a-zA-Z0-9\\(\\)\\[\\]@:%_.~&=\\/]*)?",
            // 9. Fragment, optional
            //    * -+
            //    * a-zA-Z0-9
            //    * ()
            //    * []
            //    * @ : % _ . ~ & =
            //    * /
            "(#[-\\+a-zA-Z0-9\\(\\)\\[\\]@:%_.~&=\\/]*)?",
            // 10. End
            "$"
        )
    ).unwrap();
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(
    Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoCategorize, ScryptoEncode, ScryptoDecode,
)]
#[sbor(transparent)]
pub struct UncheckedUrl(pub String);

impl Describe<ScryptoCustomTypeKind> for UncheckedUrl {
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(well_known_scrypto_custom_types::URL_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::url_type_data()
    }
}

impl UncheckedUrl {
    pub fn of(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub struct CheckedUrl(String);

impl CheckedUrl {
    pub fn of(value: impl AsRef<str>) -> Option<Self> {
        let s = value.as_ref();
        if s.len() <= MAX_URL_LENGTH && URL_REGEX.is_match(s) {
            Some(Self(s.to_owned()))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<UncheckedUrl> for CheckedUrl {
    type Error = ();

    fn try_from(value: UncheckedUrl) -> Result<Self, Self::Error> {
        CheckedUrl::of(value.as_str()).ok_or(())
    }
}

impl From<CheckedUrl> for UncheckedUrl {
    fn from(value: CheckedUrl) -> Self {
        Self(value.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url() {
        assert!(CheckedUrl::of("https://66.123.1.255:9999").is_some());
        assert!(CheckedUrl::of("https://66.123.1.255:9999/hi").is_some());
        assert!(CheckedUrl::of("https://www.google.com").is_some());
        assert!(CheckedUrl::of("https://www.google.com/").is_some());
        assert!(CheckedUrl::of("https://www.google.com/test/_abc/path").is_some());
        assert!(CheckedUrl::of("https://www.google.com/test/_abc/path?").is_some());
        assert!(CheckedUrl::of("https://www.google.com/test/_abc/path?abc=%12&def=test").is_some());
        assert!(CheckedUrl::of("https://www.google.com/q?-+a-zA-Z0-9()[]@:%_.~&=/").is_some());
        assert!(CheckedUrl::of("https://www.google.com/ /q").is_none());
        assert!(CheckedUrl::of("https://username:password@www.google.com").is_none()); // not supported
        assert!(CheckedUrl::of("https://www.google.com/path?#").is_some());
        assert!(CheckedUrl::of("https://www.google.com/path?#-+a-zA-Z0-9()[]@:%_.~&=/").is_some());
    }
}
