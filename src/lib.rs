pub mod cpu;
pub mod pia;
pub mod tia;

use std::fs::File;
use std::io::Read;

use cpu::Mos6507;
use pia::Pia6532;
use tia::Tia1A;

pub type Cartridge = Vec<u8>;

pub struct Atari2600 {
    cpu: Mos6507,
    pia: Pia6532,
    tia: Tia1A,
    cartridge: Cartridge,
}

impl Atari2600 {
    pub fn new() -> Atari2600 {
        Atari2600 {
            cpu: Mos6507::new(),
            pia: Pia6532::new(),
            tia: Tia1A::new(),
            cartridge: vec![0],
        }
    }

    pub fn insert_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = cartridge;
    }

    pub fn power_on(&self) -> Result<i32,()> {
        Ok(1)
    }
}
