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

/// For feature checks against a non-inner object
#[derive(Debug)]
pub enum FeatureChecks<TOwn: HasFeatures> {
    None,
    RequireAllSubstates,
    ForFeatures { own_features: TOwn },
}

impl<T: HasFeatures> From<T> for FeatureChecks<T> {
    fn from(value: T) -> Self {
        FeatureChecks::ForFeatures {
            own_features: value,
        }
    }
}

impl<TOwn: HasFeatures> FeatureChecks<TOwn> {
    pub fn assert_valid(
        &self,
        substate_name: &'static str,
        condition: &Condition,
        is_present: bool,
    ) -> Result<(), RuntimeError> {
        let is_valid = match self {
            FeatureChecks::None => Ok(()),
            FeatureChecks::RequireAllSubstates => {
                if is_present {
                    Ok(())
                } else {
                    Err(format!("Required all substates to be present, but {} was not present", substate_name))
                }
            },
            FeatureChecks::ForFeatures { own_features } => {
                match condition {
                    Condition::Always => {
                        if is_present {
                            Ok(())
                        } else {
                            Err(format!("Substate condition for {} required it to be always present, but it was not", substate_name))
                        }
                    }
                    Condition::IfFeature(feature) => {
                        let feature_enabled = own_features.feature_names_str().contains(&feature.as_str());
                        if feature_enabled && !is_present {
                            Err(format!("Substate condition for {} required it to be present when the feature {} was enabled, but it was absent", substate_name, feature))
                        } else if !feature_enabled && is_present {
                            Err(format!("Substate condition for {} required it to be absent when the feature {} was disabled, but it was present", substate_name, feature))
                        } else {
                            Ok(())
                        }
                    },
                    Condition::IfOuterFeature(_) => {
                        Err(format!("Substate condition for {} required an outer object feature, but the blueprint does not have an outer blueprint defined", substate_name))
                    }
                }
            },
        };
        is_valid.map_err(|error_message| {
            RuntimeError::SystemError(SystemError::InvalidNativeSubstatesForFeature(error_message))
        })
    }
}

/// For feature checks against an inner object
pub enum InnerObjectFeatureChecks<TOwn, TOuter> {
    None,
    RequireAllSubstates,
    ForFeatures {
        own_features: TOwn,
        outer_object_features: TOuter,
    },
}

impl<TOwn: HasFeatures, TOuter: HasFeatures> InnerObjectFeatureChecks<TOwn, TOuter> {
    pub fn assert_valid(
        &self,
        substate_name: &'static str,
        condition: &Condition,
        is_present: bool,
    ) -> Result<(), RuntimeError> {
        let is_valid = match self {
            Self::None => Ok(()),
            Self::RequireAllSubstates => {
                if is_present {
                    Ok(())
                } else {
                    Err(format!(
                        "Required all substates to be present, but {} was not present",
                        substate_name
                    ))
                }
            }
            Self::ForFeatures {
                own_features,
                outer_object_features,
            } => match condition {
                Condition::Always => {
                    if is_present {
                        Ok(())
                    } else {
                        Err(format!("Substate condition for {} required it to be always present, but it was not", substate_name))
                    }
                }
                Condition::IfFeature(feature) => {
                    let feature_enabled =
                        own_features.feature_names_str().contains(&feature.as_str());
                    if feature_enabled && !is_present {
                        Err(format!("Substate condition for {} required it to be present when the feature {} was enabled, but it was absent", substate_name, feature))
                    } else if !feature_enabled && is_present {
                        Err(format!("Substate condition for {} required it to be absent when the feature {} was disabled, but it was present", substate_name, feature))
                    } else {
                        Ok(())
                    }
                }
                Condition::IfOuterFeature(feature) => {
                    let feature_enabled = outer_object_features
                        .feature_names_str()
                        .contains(&feature.as_str());
                    if feature_enabled && !is_present {
                        Err(format!("Substate condition for {} required it to be present when the outer object feature {} was enabled, but it was absent", substate_name, feature))
                    } else if !feature_enabled && is_present {
                        Err(format!("Substate condition for {} required it to be absent when the outer object feature {} was disabled, but it was present", substate_name, feature))
                    } else {
                        Ok(())
                    }
                }
            },
        };
        is_valid.map_err(|error_message| {
            RuntimeError::SystemError(SystemError::InvalidNativeSubstatesForFeature(error_message))
        })
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
