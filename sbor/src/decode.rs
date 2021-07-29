pub trait Decode<'de>: Sized {
    fn decode(decoder: Decoder<'de>) -> Result<Self, ()>;
}

pub struct Decoder<'de> {
    data: &'de [u8],
    offset: usize,
}

impl<'de> Decoder<'de> {
    pub fn new(data: &'de [u8]) -> Self {
        Self { data, offset: 0 }
    }
}
