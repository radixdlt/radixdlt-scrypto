/// Range start of the measuremnts
const MIN_SIZE: usize = 1;
/// Range end of the measuremnts
const MAX_SIZE: usize = 4 * 1024 * 1024;
/// Range step
const SIZE_STEP: usize = 20 * 1024;
/// Each step write and read
const COUNT: usize = 20;
/// Multiplication of each step read (COUNT * READ_REPEATS)
const READ_REPEATS: usize = 200;

#[cfg(test)]
mod common;

#[cfg(test)]
mod read;

#[cfg(test)]
mod write;

#[cfg(test)]
mod delete;
