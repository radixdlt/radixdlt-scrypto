use simulator::scrypto;

pub fn main() -> Result<(), scrypto::Error> {
    scrypto::run(std::env::args())
}
