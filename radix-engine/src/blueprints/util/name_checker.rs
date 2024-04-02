use radix_common::prelude::*;

pub fn check_name(name: &str) -> Result<(), InvalidNameError> {
    let mut iter = name.chars().enumerate();
    match iter.next() {
        Some((_, 'A'..='Z' | 'a'..='z' | '_')) => {
            for (index, char) in iter {
                if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
                    return Err(InvalidNameError::InvalidChar {
                        name: name.to_owned(),
                        violating_char: char.to_string(),
                        index: index,
                    });
                }
            }
            Ok(())
        }
        Some((index, char)) => Err(InvalidNameError::InvalidChar {
            name: name.to_owned(),
            violating_char: char.to_string(),
            index,
        }),
        None => Err(InvalidNameError::EmptyString),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum InvalidNameError {
    EmptyString,
    InvalidChar {
        name: String,
        violating_char: String,
        index: usize,
    },
}
