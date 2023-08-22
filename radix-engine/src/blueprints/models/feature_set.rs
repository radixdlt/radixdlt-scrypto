use crate::internal_prelude::*;

pub trait HasFeatures: Debug {
    fn feature_names_str(&self) -> Vec<&'static str>;

    fn feature_names_str_set(&self) -> BTreeSet<&'static str> {
        self.feature_names_str().into_iter().collect()
    }

    fn feature_names_string(&self) -> Vec<String> {
        self.feature_names_str()
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
    }

    fn feature_names_string_set(&self) -> BTreeSet<String> {
        self.feature_names_str()
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
    }
}
