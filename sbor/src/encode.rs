pub trait Encode {
    fn encode(&self, encoder: Encoder);
}

pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Self {
        todo!()
    }

    pub fn encode_bool(&self, value: bool) {
        todo!()
    }

    pub fn encode_i8(&self, value: i8) {
        todo!()
    }
}
