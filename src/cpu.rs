use std::fs::File;
use std::io::Read;

use super::Cartridge;
use pia::Pia6532;
use tia::Tia1A;

pub struct Mos6507 {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    flags: u8,
}

impl Mos6507 {
    pub fn new() -> Mos6507 {
        Mos6507 {
            a: 0u8,
            x: 0u8,
            y: 0u8,
            sp: 0u8,
            pc: 0u16,
            flags: 0u8, //TODO - set flags to appropriate defaults
        }
    }

    pub fn run(&mut self, pia: &Pia6532, tia: &Tia1A, rom: &Cartridge) {
        // begin execution from the RESET vector in rom 0xFFFC-0xFFFB
        self.pc = self.read_word(pia, tia, rom, 0xFFFB);

        loop {
            //Fetch the opcode
            let opcode = self.read_byte(pia, tia, rom, self.pc);

            //TODO - begin execution loop
            break;
        }
    }

    fn read_word(&self, pia: &Pia6532, tia: &Tia1A, rom: &Cartridge, address: u16) -> u16 {
        //TODO - retrieve two bytes from the supplied address
        0
    }

    fn read_byte(&self, pia: &Pia6532, tia: &Tia1A, rom: &Cartridge, address: u16) -> u8 {
            //TODO map address to underlying components
            0
    }

    fn write(&self, pia: Pia6532, tia: Tia1A, rom: Cartridge, address: u16) {
            //TODO - map address to underlying components
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
