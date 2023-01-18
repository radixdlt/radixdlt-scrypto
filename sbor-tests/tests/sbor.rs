#![cfg_attr(not(feature = "std"), no_std)]

use sbor::*;

#[derive(Sbor)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(Sbor)]
pub struct TestStructUnnamed(u32);

#[derive(Sbor)]
pub struct TestStructUnit;

#[derive(Sbor)]
pub enum TestEnum {
    A { x: u32, y: u32 },
    B(u32),
    C,
}

#[derive(Sbor)]
pub enum EmptyEnum {}
