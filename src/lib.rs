use std::fs::File;
use std::io::Bytes;

pub struct Atari2600 {
    cpu: Mos6507,
}

impl Atari2600 {
    pub fn new(cartridge: Bytes<File>) -> Atari2600 {
        Atari2600 {
            cpu: Mos6507::new(cartridge),
        }
    }

    fn power_on(&self) -> Result<T, E> {
        //TODO - emulate turning the Atari on
        println!("Atari2600 powered on");
    }
}

struct Mos6507 {
    program: Bytes<File>,
}

impl Mos6507 {
    fn new(program: Bytes<File>) -> Mos6507 {
        Mos6507 {
            program: program,
        }
    }
}
