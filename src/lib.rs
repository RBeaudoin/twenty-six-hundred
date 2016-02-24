pub mod cpu;
pub mod tia;
pub mod pia;

use std::fs::File;
use std::io::Read;
use cpu::Mos6507;

pub type Cartridge = Vec<u8>;

pub struct Atari2600 {
    cpu: Mos6507,
}

impl Atari2600 {
    pub fn new() -> Atari2600 {
        Atari2600 {
            cpu: Mos6507::new(),
        }
    }

    pub fn power_on(&self) -> Result<i32,()> {
        Ok(1)
    }
}
