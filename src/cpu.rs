use super::Cartridge;
use pia::Pia6532;
use tia::Tia1A;

// for flags register
const CARRY_MASK: u8                = 0x01;
const ZERO_RESULT_MASK: u8          = 0x02;
const INTERRUPT_DISABLE_MASK: u8    = 0x04;
const DECIMAL_MASK: u8              = 0x08;
const BREAK_COMMAND_MASK: u8        = 0x10;
const OVERFLOW_MASK: u8             = 0x40;
const NEGATIVE_MASK: u8             = 0x80;

// to extract lower byte from word
const LOW_BYTE_MASK: u16             = 0xFF;

enum AddressMode {
    Immediate{oper: u8},
    ZeroPage{oper: u8},
    ZeroPageX{oper: u8, x: u8},
    Absolute{oper: u16},
    AbsoluteX{oper: u16, x: u8},
    AbsoluteY{oper: u16, y: u8},
    IndirectX{oper: u8, x: u8},
    IndirectY{oper: u8, y: u8},
}

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
            flags: 0u8,
        }
    }

    pub fn run(&mut self, pia: &Pia6532, tia: &Tia1A, rom: &Cartridge) {
        // begin execution from the RESET vector in rom 0xFFFC-0xFFFB
        self.pc = read_word(pia, tia, rom, 0xFFFB);

        loop {
           self.execute_instruction(pia, tia, rom);
           // TODO - handle interupts
        }
    }

    fn execute_instruction(&mut self, pia: &Pia6532, tia: &Tia1A, rom: &Cartridge) {
        let opcode = read_byte(pia, tia, rom, AddressMode::Absolute{oper: self.pc});

        // TODO - as other opcodes are added I'll want to
        // add another level of match inside this one, with
        // this level handling address mode and fetch, and
        // the inner level calling the func that exeutes the instruction
        match opcode {
            0x69    => {
                let next_byte = read_byte(pia,tia,rom,AddressMode::Absolute{oper: self.pc + 1});
                let address_mode = AddressMode::Immediate{oper: next_byte};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 1;
            },
            0x65    => {
                let next_byte = read_byte(pia,tia,rom,AddressMode::Absolute{oper: self.pc + 1});
                let address_mode = AddressMode::ZeroPage{oper: next_byte};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 1;
            },
            0x75    => {
                let next_byte = read_byte(pia,tia,rom,AddressMode::Absolute{oper: self.pc + 1});
                let address_mode = AddressMode::ZeroPageX{oper: next_byte, x: self.x};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 1;
            },
            0x6D    => {
                let next_word = read_word(pia,tia,rom,self.pc + 1);
                let address_mode = AddressMode::Absolute{oper: next_word};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 2; 
            },
            0x7D    => {
                let next_word = read_word(pia,tia,rom,self.pc + 1);
                let address_mode = AddressMode::AbsoluteX{oper: next_word, x: self.x};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 2; 
            },
            0x79    => {
                let next_word = read_word(pia,tia,rom,self.pc + 1);
                let address_mode = AddressMode::AbsoluteY{oper: next_word, y: self.y};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 2;
            },
            0x61    => {
                let next_byte = read_byte(pia,tia,rom,AddressMode::Absolute{oper: self.pc + 1});
                let address_mode = AddressMode::IndirectX{oper: next_byte, x: self.x};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 1; 
            },
            0x71    => {
                let next_byte = read_byte(pia,tia,rom,AddressMode::Absolute{oper: self.pc + 1});
                let address_mode = AddressMode::IndirectY{oper: next_byte, y: self.y};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 1; 
            },
            _       => panic!("Unknown opcode {}", opcode),
        }
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
            let wide_result = (self.a as u16) 
                + (self.flag_value(CARRY_MASK) as u16) + (memory as u16); 
            
            // carry check
            self.set_flag((wide_result > 255), CARRY_MASK);

            // overflow check
            self.set_flag((wide_result as i16) > 127 || 
                          (wide_result as i16) < -128,
                          OVERFLOW_MASK);
        }
    }
}

fn read_word(pia: &Pia6532, tia: &Tia1A, rom: &Cartridge, address: u16) -> u16 {
    let low_byte = read_byte(pia, tia, rom, AddressMode::Absolute{oper: address}) as u16;
    let high_byte = read_byte(pia, tia, rom, AddressMode::Absolute{oper: address + 1}) as u16;
    (high_byte << 8) + low_byte
    
}

fn read_byte(pia: &Pia6532, tia: &Tia1A, rom: &Cartridge, address: AddressMode) -> u8 {
    //TODO map address to underlying components
    0
}

fn write(pia: Pia6532, tia: Tia1A, rom: Cartridge, address: AddressMode, data: u8) {
    //TODO - map address to underlying components
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
