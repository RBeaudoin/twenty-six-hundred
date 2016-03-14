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

enum AddressMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    None
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
        self.pc = self.read_word(pia, tia, rom, 0xFFFB);

        loop {
           self.execute_instruction(pia, tia, rom);
           // TODO - handle interupts
        }
    }

    fn execute_instruction(&mut self, pia: &Pia6532, tia: &Tia1A, rom: &Cartridge) {
        let opcode = self.read_byte(pia, tia, rom, &AddressMode::Absolute);
        let address_mode = self.get_address_mode(opcode); 

        // outer match for address mode
        match opcode {
            // ADC
            0x69 | 0x65 | 0x75 | 0x6D |
            0x7D | 0x79 | 0x61 | 0x71    => {
                let operand = self.read_byte(pia, tia, rom, &address_mode);
                self.adc(operand);
            },
            _       => panic!("Unknown opcode {}", opcode),
        }

        self.pc += self.get_pc_offset(address_mode);
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

    fn get_address_mode(&self, opcode: u8) -> AddressMode {
        match opcode {
            0x69    => AddressMode::Immediate,
            0x65    => AddressMode::ZeroPage,
            0x75    => AddressMode::ZeroPageX,
            0x6D    => AddressMode::Absolute,
            0x7D    => AddressMode::AbsoluteX,
            0x79    => AddressMode::AbsoluteY,
            0x61    => AddressMode::IndirectX,
            0x71    => AddressMode::IndirectY,
            _       => AddressMode::None,
        }
    }

    fn get_pc_offset(&self, address_mode: AddressMode) -> u16 {
        match address_mode {
            AddressMode::Absolute   |
            AddressMode::AbsoluteX  | 
            AddressMode::AbsoluteY      => 3,
            AddressMode::None           => 1,
            _                           => 2,
        }
    }

    fn read_word(&self, pia: &Pia6532, tia: &Tia1A, rom: &Cartridge, address: u16) -> u16 {
        let low_byte = self.read_byte_by_addr(pia, tia, rom, address) as u16;
        let high_byte = self.read_byte_by_addr(pia, tia, rom, address + 1) as u16;
        (high_byte << 8) + low_byte
    }

    fn read_byte(&self, pia: &Pia6532, tia: &Tia1A, rom: &Cartridge, address_mode: &AddressMode) -> u8 {
        //TODO map address to underlying components
        0
    }
    
    fn read_byte_by_addr(&self, pia: &Pia6532, tia: &Tia1A, rom: &Cartridge, address: u16) -> u8 {
        //TODO map address to underlying components
        0
    }
    
    fn write(&self, pia: Pia6532, tia: Tia1A, rom: Cartridge, address: AddressMode, data: u8) {
        //TODO - map address to underlying components
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
            
            if temp <= 9 
            {
                low_nibble = temp;
                nibble_carry = 0;
            } else 
            {
                low_nibble = (temp + 6) & LOW_NIBBLE_MASK; // BCD correction by adding 6
                nibble_carry = 1;
            };

            temp = (self.a >> 4) + (operand >> 4) + nibble_carry;
            
            if temp <= 9 
            {
                high_nibble = temp;
                nibble_carry = 0;
            } else 
            {
                high_nibble = (temp + 6) & LOW_NIBBLE_MASK; // BCD correction by adding 6
                nibble_carry = 1;
            };
            
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
        
        let temp = self.a;

        // negative check
        self.set_flag((temp >> 7) == 1, NEGATIVE_MASK);

        // zero check
        self.set_flag(temp == 0, ZERO_RESULT_MASK);
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
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), false);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    }

    #[test]
    fn adc_with_carry() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x01;

        cpu.adc(1);

        assert_eq!(cpu.a, 2);
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), false);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    }
    
    #[test]
    fn adc_carry_flag_zero_flag() {
        let mut cpu = Mos6507::new();
        cpu.a = 255;

        cpu.adc(1);

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), true);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
    }
   
    #[test]
    fn adc_overflow_flag_negative_flag() {
        let mut cpu = Mos6507::new();
        cpu.a = 127;

        cpu.adc(1);

        assert_eq!(cpu.a, 128);
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), true);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    }
   
     #[test]
    fn adc_decimal() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x08;
        cpu.a = 17; // '11' in BCD

        cpu.adc(35); // '23' in BCD

        assert_eq!(cpu.a, 52); // '34' in BCD
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), false);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    }
    
    #[test]
    fn adc_decimal_one_column_carry() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x08;
        cpu.a = 53; // '35' in BCD

        cpu.adc(38); // '26' in BCD

        assert_eq!(cpu.a, 97); // '61' in BCD
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), false);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    }

    #[test]
    fn adc_decimal_carry_flag_zero_flag() {
        let mut cpu = Mos6507::new();
        cpu.flags = 0x08;
        cpu.a = 71; // '47' in BCD

        cpu.adc(83); // '53' in BCD

        assert_eq!(cpu.a, 0); // '00' in BCD
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), false);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
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
