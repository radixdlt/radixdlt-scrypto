use sbor::*;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Blueprint {
    code: Vec<u8>,
}

impl Blueprint {
    pub fn new(code: Vec<u8>) -> Self {
        Self { code }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }
}
