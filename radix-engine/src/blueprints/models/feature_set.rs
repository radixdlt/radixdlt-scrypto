use crate::internal_prelude::*;

pub trait HasFeatures: Debug {
    fn feature_names_str(&self) -> Vec<&'static str>;

    fn feature_names_string_set(&self) -> IndexSet<String> {
        self.feature_names_str()
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
    }
}
