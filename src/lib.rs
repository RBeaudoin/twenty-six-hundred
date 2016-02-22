pub mod cpu;
pub mod tia;
pub mod pia;

use std::fs::File;
use std::io::Read;
use cpu::Mos6507;

pub struct Atari2600 {
    cpu: Mos6507,
}

impl Atari2600 {
    pub fn new(cartridge: Vec<u8>) -> Atari2600 {
        Atari2600 {
            cpu: Mos6507::new(cartridge),
        }
    }

    pub fn power_on(&self) -> Result<i32,()> {
        //TODO - emulate turning the Atari on
        println!("Atari2600: powered on");
        println!("Atari2600: running program");
        self.cpu.run();
        Ok(1)
    }
}
