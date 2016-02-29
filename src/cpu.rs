use super::Cartridge;
use pia::Pia6532;
use tia::Tia1A;

const CARRY_MASK: u8 = 0x01;
const DECIMAL_MASK: u8 = 0x08;
const OVERFLOW_MASK: u8 = 0x40;
const NEGATIVE_MASK: u8 = 0x80;

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
            // fetch the opcode
            let opcode = self.read_byte(pia, tia, rom, self.pc);
            
            // fetch the first operand
            self.pc += 1;
            let operand = self.read_byte(pia, tia, rom, self.pc);

            match opcode {
                // ADC - Add with carry
                0x69    => self.adc_immediate(operand),
                _       => panic!("Unrecognized opcode: {}", opcode),
            }
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
    
    fn carry_flag_value(&self) -> u8 {
        if CARRY_MASK & self.flags == 0 {0} else {1}
    }

    fn decimal_flag_value(&self) -> u8 {
        if DECIMAL_MASK & self.flags == 0 {0} else {1}
    }

    fn set_carry(&mut self) {
        self.flags = self.flags | CARRY_MASK;
    }

    fn set_overflow(&mut self) {
        self.flags = self.flags | OVERFLOW_MASK;
    }

    fn adc_immediate(&mut self, operand: u8) {
        // add acc to operand and carry, store in acc
        // set the carry, overflow, and sign bits accordingly
        
        if self.decimal_flag_value() == 1 {
            // TODO - packed BCD arithmetic is hard...
        } else {
            let temp = self.a.wrapping_add(operand.wrapping_add(self.carry_flag_value()));
            
            // carry check
            if self.a > temp {
                self.set_carry(); // TODO clear if necessary
            }

            // overflow check
            if  (operand as i8).checked_add(self.carry_flag_value() as i8) == None {
                self.set_overflow(); // TODO - clear if necessary
            } else if (self.a as i8).checked_add((operand as i8) + (self.carry_flag_value() as i8)) == None {
                self.set_overflow(); // TODO - clear if necessary
            }
            self.a = temp;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn adc_immediate() {
        let mut cpu = Mos6507::new();

        cpu.adc_immediate(1);

        assert_eq!(cpu.a, 1);
    }

    #[test]
    fn adc_immediate_with_carry() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x01;

        cpu.adc_immediate(1);

        assert_eq!(cpu.a, 2);
    }
}
