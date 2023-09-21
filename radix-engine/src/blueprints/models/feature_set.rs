use crate::internal_prelude::*;

pub trait HasFeatures: Debug {
    fn feature_names_str(&self) -> Vec<&'static str>;

    fn feature_names_str_set(&self) -> IndexSet<&'static str> {
        self.feature_names_str().into_iter().collect()
    }

    fn feature_names_string(&self) -> Vec<String> {
        self.feature_names_str()
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
    }

    fn feature_names_string_set(&self) -> IndexSet<String> {
        self.feature_names_str()
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MyFeatures;

    impl HasFeatures for MyFeatures {
        fn feature_names_str(&self) -> Vec<&'static str> {
            vec!["feature_1", "feature_2"]
        }
    }

    #[test]
    fn validate_feature_names_getters() {
        let my_features = MyFeatures;

        let idx_set = my_features.feature_names_str_set();
        assert_eq!(idx_set.get_index_of("feature_1").unwrap(), 0);
        assert_eq!(idx_set.get_index_of("feature_2").unwrap(), 1);

        let v = my_features.feature_names_string();
        assert_eq!(v[0], "feature_1");
        assert_eq!(v[1], "feature_2");
    }
}
