use simulator::rev2;

pub fn main() -> Result<(), rev2::Error> {
    rev2::run(std::env::args())
}
