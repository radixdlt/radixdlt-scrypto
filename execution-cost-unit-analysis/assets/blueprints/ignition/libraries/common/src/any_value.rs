// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use radix_common::prelude::*;

#[derive(Clone, Debug, ScryptoSbor, PartialEq, Eq)]
#[sbor(transparent)]
pub struct AnyValue((ScryptoValue,));

impl AnyValue {
    pub fn from_typed<T>(typed: &T) -> Result<Self, AnyValueError>
    where
        T: ScryptoEncode,
    {
        scrypto_encode(typed)
            .map_err(Into::into)
            .and_then(|value| scrypto_decode(&value).map_err(Into::into))
            .map(|value| Self((value,)))
    }

    pub fn as_typed<T>(&self) -> Result<T, AnyValueError>
    where
        T: ScryptoDecode,
    {
        scrypto_encode(&self.0 .0)
            .map_err(Into::into)
            .and_then(|value| scrypto_decode(&value).map_err(Into::into))
    }
}

#[derive(Clone, Debug)]
pub enum AnyValueError {
    EncodeError(EncodeError),
    DecodeError(DecodeError),
}

impl From<EncodeError> for AnyValueError {
    fn from(value: EncodeError) -> Self {
        Self::EncodeError(value)
    }
}

impl From<DecodeError> for AnyValueError {
    fn from(value: DecodeError) -> Self {
        Self::DecodeError(value)
    }
}

#[cfg(test)]
mod test {
    use super::AnyValue;

    #[test]
    fn simple_roundtrip_test() {
        // Arrange
        let value = 12;

        // Act
        let any_value = AnyValue::from_typed(&value).unwrap();

        // Assert
        assert_eq!(any_value.as_typed::<i32>().unwrap(), value)
    }
}
