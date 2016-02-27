use std::fs::File;
use std::io::Read;

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

    fn read(&self, pia: Pia6532, tia: Tia1A, address: u16) -> u8 {
            //TODO map address to underlying components
            0
    }

    fn write(&self, pia: Pia6532, tia: Tia1A, address: u16) {
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
