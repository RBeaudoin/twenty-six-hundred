use super::Cartridge;
use pia::Pia6532;
use tia::Tia1A;

const CARRY_MASK: u8                = 0x01;
const ZERO_RESULT_MASK: u8          = 0x02;
const INTERRUPT_DISABLE_MASK: u8    = 0x04;
const DECIMAL_MASK: u8              = 0x08;
const BREAK_COMMAND_MASK: u8        = 0x10;
const OVERFLOW_MASK: u8             = 0x40;
const NEGATIVE_MASK: u8             = 0x80;

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
                0x69    => self.adc(operand),
                0x65    => println!("TODO: implement ADC zero page"),
                0x75    => println!("TODO: implement ADC zero page x"),
                0x6D    => println!("TODO: implement ADC absolute"),
                0x7D    => println!("TODO: implement ADC absolute x"),
                0x79    => println!("TODO: implement ADC absolute y"),
                0x61    => println!("TODO: implement ADC indirect x"),
                0x71    => println!("TODO: implement ADC indirect y"),
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
    
    fn flag_value(&self, mask: u8) -> u8 {
        if mask & self.flags == 0 {0} else {1}
    }

    fn set_flag(&mut self, value: bool, mask: u8) {
        if value {
            self.flags = self.flags | mask;
        } else { 
            self.flags = self.flags & !mask;
        }
    }

    fn adc(&mut self, memory: u8) {
        if self.flag_value(DECIMAL_MASK) == 1 {
            // TODO - packed BCD arithmetic is hard...
        } else {
            self.a = self.a.wrapping_add(memory.wrapping_add(self.flag_value(CARRY_MASK)));
            
            // use u16 to easily check for carry, overload
            let wide_result = (self.a as u16) + (self.flag_value(CARRY_MASK) as u16) + (memory as u16); 
            
            // carry check
            self.set_flag((wide_result > 255), CARRY_MASK);

            // overflow check
            self.set_flag((wide_result as i16) > 127 || 
                          (wide_result as i16) < -128,
                          OVERFLOW_MASK);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn adc() {
        let mut cpu = Mos6507::new();

        cpu.adc(1);

        assert_eq!(cpu.a, 1);
    }

    #[test]
    fn adc_with_carry() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x01;

        cpu.adc(1);

        assert_eq!(cpu.a, 2);
    }

    #[test]
    fn flag_value() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x01 | 0x02 | 0x04 | 0x08 |
                    0x10 | 0x40 | 0x80;

        assert_eq!(cpu.flag_value(super::CARRY_MASK), 1);
        assert_eq!(cpu.flag_value(super::ZERO_RESULT_MASK), 1);
        assert_eq!(cpu.flag_value(super::INTERRUPT_DISABLE_MASK), 1);
        assert_eq!(cpu.flag_value(super::DECIMAL_MASK), 1);
        assert_eq!(cpu.flag_value(super::BREAK_COMMAND_MASK), 1);
        assert_eq!(cpu.flag_value(super::OVERFLOW_MASK), 1);
        assert_eq!(cpu.flag_value(super::NEGATIVE_MASK), 1);
    }

    #[test]
    fn set_value() {
        let mut cpu = Mos6507::new();
        
        cpu.set_flag(true, 0x01);
        cpu.set_flag(true, 0x02);
        cpu.set_flag(true, 0x04);
        cpu.set_flag(true, 0x08);
        cpu.set_flag(true, 0x10);
        cpu.set_flag(true, 0x40);
        cpu.set_flag(true, 0x80);
        
        assert_eq!(cpu.flags,
                    super::CARRY_MASK | 
                    super::ZERO_RESULT_MASK |
                    super::INTERRUPT_DISABLE_MASK | 
                    super::DECIMAL_MASK | 
                    super::BREAK_COMMAND_MASK |
                    super::OVERFLOW_MASK | 
                    super::NEGATIVE_MASK);
    
        
        cpu.set_flag(false, 0x01);
        cpu.set_flag(false, 0x02);
        cpu.set_flag(false, 0x04);
        cpu.set_flag(false, 0x08);
        cpu.set_flag(false, 0x10);
        cpu.set_flag(false, 0x40);
        cpu.set_flag(false, 0x80);

        assert_eq!(cpu.flags, 0);
    }
}
