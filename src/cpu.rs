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
const LOW_BYTE_MASK: u16            = 0xFF;

// to extract nibbles for BCD operations
const LOW_NIBBLE_MASK: u8           = 0x0F;
const HIGH_NIBBLE_MASK: u8          = 0xF0;

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
        let next_word = read_word(pia,tia,rom,self.pc + 1);
        let next_byte = (next_word & LOW_BYTE_MASK) as u8;  

        // TODO - as other opcodes are added I'll want to
        // add another level of match inside this one, with
        // this level handling address mode and fetch, and
        // the inner level calling the func that exeutes the instruction
        match opcode {
            0x69    => {
                let address_mode = AddressMode::Immediate{oper: next_byte};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 1;
            },
            0x65    => {
                let address_mode = AddressMode::ZeroPage{oper: next_byte};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 1;
            },
            0x75    => {
                let address_mode = AddressMode::ZeroPageX{oper: next_byte, x: self.x};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 1;
            },
            0x6D    => {
                let address_mode = AddressMode::Absolute{oper: next_word};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 2; 
            },
            0x7D    => {
                let address_mode = AddressMode::AbsoluteX{oper: next_word, x: self.x};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 2; 
            },
            0x79    => {
                let address_mode = AddressMode::AbsoluteY{oper: next_word, y: self.y};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 2;
            },
            0x61    => {
                let address_mode = AddressMode::IndirectX{oper: next_byte, x: self.x};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 1; 
            },
            0x71    => {
                let address_mode = AddressMode::IndirectY{oper: next_byte, y: self.y};
                self.adc(read_byte(pia,tia,rom,address_mode));
                self.pc += 1; 
            },
            _       => panic!("Unknown opcode {}", opcode),
        }
    }

    fn flag_set(&self, mask: u8) -> bool {
        mask & self.flags > 0
    }

    fn set_flag(&mut self, value: bool, mask: u8) {
        if value {
            self.flags = self.flags | mask;
        } else { 
            self.flags = self.flags & !mask;
        }
    } 

    fn adc(&mut self, operand: u8) {
        let carry = CARRY_MASK & self.flags;

        if self.flag_set(DECIMAL_MASK) {
            let low_nibble;
            let high_nibble;
            let mut nibble_carry;

            // TODO - do I need to validate the nibbles? What does
            // the 6507 do when it has invalid BCD operands?

            // TODO - I can probably optimize this later
            let mut temp = (LOW_NIBBLE_MASK & self.a) + (LOW_NIBBLE_MASK & operand) + carry;
            println!("temp 1 is: {}", temp); 
            if temp <= 9 
            {
                low_nibble = temp;
                nibble_carry = 0;
            } else 
            {
                low_nibble = (temp + 6) & LOW_NIBBLE_MASK; // BCD correction by adding 6
                nibble_carry = 1;
            };
            println!("low nibble is: {}", low_nibble); 

            temp = (self.a >> 4) + (operand >> 4) + nibble_carry;
            println!("temp 2 is: {}", temp); 
            
            if temp <= 9 
            {
                high_nibble = temp;
                nibble_carry = 0;
            } else 
            {
                high_nibble = (temp + 6) & LOW_NIBBLE_MASK; // BCD correction by adding 6
                nibble_carry = 1;
            };
            
            println!("high nibble is: {}", high_nibble); 
            
            // carry check
            self.set_flag(nibble_carry == 1, CARRY_MASK);

            self.a = (high_nibble << 4) | low_nibble;
        
        } else {
            let temp = self.a.wrapping_add(operand.wrapping_add(carry));
            
            // use u16 to easily check for carry, overload
            let wide_result = (self.a as u16) 
                            + (carry as u16) + (operand as u16); 
            
            // carry check
            self.set_flag((wide_result > 255), CARRY_MASK);

            // overflow check
            self.set_flag((wide_result as i16) > 127 || 
                          (wide_result as i16) < -128,
                          OVERFLOW_MASK);
            
            self.a = temp;
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
    fn adc_set_carry_flag() {
        let mut cpu = Mos6507::new();
        cpu.a = 255;

        cpu.adc(1);

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
    }
   
    #[test]
    fn adc_set_overflow_flag() {
        let mut cpu = Mos6507::new();
        cpu.a = 127;

        cpu.adc(1);

        assert_eq!(cpu.a, 128);
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), true);
    }
   
     #[test]
    fn adc_decimal() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x08;
        cpu.a = 17; // '11' in BCD

        cpu.adc(35); // '23' in BCD

        assert_eq!(cpu.a, 52); // '34' in BCD
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
    }
    
    #[test]
    fn adc_decimal_one_column_carry() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x08;
        cpu.a = 53; // '35' in BCD

        cpu.adc(38); // '26' in BCD

        assert_eq!(cpu.a, 97); // '61' in BCD
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
    }

    #[test]
    fn adc_decimal_set_carry_flag() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x08;
        cpu.a = 71; // '47' in BCD

        cpu.adc(83); // '53' in BCD

        assert_eq!(cpu.a, 0); // '00' in BCD
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
    }


    #[test]
    fn flag_value() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x01 | 0x02 | 0x04 | 0x08 |
                    0x10 | 0x40 | 0x80;
        
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
        assert_eq!(cpu.flag_set(super::INTERRUPT_DISABLE_MASK), true);
        assert_eq!(cpu.flag_set(super::DECIMAL_MASK), true);
        assert_eq!(cpu.flag_set(super::BREAK_COMMAND_MASK), true);
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
    }
   
    #[test]
    fn flag_value_mixed() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x01 | 0x04 |
                    0x10 | 0x80;
        
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
        assert_eq!(cpu.flag_set(super::INTERRUPT_DISABLE_MASK), true);
        assert_eq!(cpu.flag_set(super::DECIMAL_MASK), false);
        assert_eq!(cpu.flag_set(super::BREAK_COMMAND_MASK), true);
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
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
