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
    Relative,
    Accumulator,
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
        let operand = self.read_byte(pia, tia, rom, &address_mode);

        match opcode {
            // ADC
            0x69 | 0x65 | 0x75 | 0x6D |
            0x7D | 0x79 | 0x61 | 0x71    => {
                self.adc(operand);
            },
            // AND
            0x29 | 0x25 | 0x35 | 0x2D |
            0x3D | 0x39 | 0x21 | 0x31   => {
                self.and(operand);
            },
            // ASL
            0x0A | 0x06 | 0x16 | 0x0E |
            0x1E                        => {
                self.asl(operand, &address_mode);
            },
            // BCC
            0x90                        => {
                self.bcc(operand);
            },
            // BCS
            0xB0                        => {
                self.bcs(operand);
            },
            // BEQ
            0xF0                        => {
                self.beq(operand);
            },
            // BIT
            0x24 | 0x2C                 => {
                self.bit(operand);
            },
            // BMI
            0x30                        => {
                self.bmi(operand);
            },
            // BNE
            0xD0                        => {
                self.bmi(operand);
            },
            // BPL
            0x10                        => {
                self.bpl(operand);
            },
            // BRK - TODO
            // BVC
            0x50                        => {
                self.bvc(operand);
            },
            // BVS
            0x70                        => {
                self.bvs(operand);
            },
            // CLC
            0x18                        => {
                self.clc();
            },
            // CLD
            0xD8                        => {
                self.cld();
            },
            // CLI
            0x58                        => {
                self.cli();
            },
            // CLV
            0xB8                        => {
                self.clv();
            },
            // CMP
            0xC9 | 0xC5 | 0xD5 | 0xCD |   
            0xDD | 0xD9 | 0xC1 | 0xD1   => {
                self.cmp(operand);
            },
            // CPX
            0xE0 | 0xE4 | 0xEC          => {
                self.cpx(operand);
            },
            // CPY
            0xC0 | 0xC4 | 0xCC          => {
                self.cpy(operand);
            },
            // DEC TODO
            // DEX
            0xCA                        => {
                self.dex();
            },
            // EOR TODO
            // INC TODO
            // INX
            0xE8                        => {
                self.inc();
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
            0x69 | 0x29 | 0xC9 | 0xE0 |
            0xC0
                => AddressMode::Immediate,
            0x65 | 0x25 | 0x06 | 0x24 |
            0xC5 | 0xE4 | 0xC4
                => AddressMode::ZeroPage,
            0x75 | 0x35 | 0x16 | 0xD5
                => AddressMode::ZeroPageX,
            0x6D | 0x2D | 0x0E | 0x2C |
            0xCD | 0xEC | 0xCC
                => AddressMode::Absolute,
            0x7D | 0x3D | 0x1E | 0xDD
                => AddressMode::AbsoluteX,
            0x79 | 0x39 | 0xD9
                => AddressMode::AbsoluteY,
            0x61 | 0x21 | 0xC1
                => AddressMode::IndirectX,
            0x71 | 0x31 | 0xD1
                => AddressMode::IndirectY,
            0x0A
                => AddressMode::Accumulator,
            0x90 | 0xB0 | 0x30 | 0xD0 |
            0x10 | 0x50
                => AddressMode::Relative,
            _   => AddressMode::None,
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
 
    fn branch_on_flag(&mut self, operand: u8, predicate: bool, mask: u8) {
        // 6507 uses signed operand for branch, hence this mess
        let signed_operand = operand as i8;

        if self.flag_set(mask) == predicate {
            if signed_operand < 0 {
                let temp = -signed_operand;
                self.pc -= temp as u16;
            } else {
                self.pc += operand as u16;
            }
        }
    }

    fn inc(&mut self) {
        let result = (self.x as i8) + 1;

        self.set_flag(result == 0, ZERO_RESULT_MASK);
        self.set_flag(result < 0, NEGATIVE_MASK);
        self.x = (result as u8);
    }

    fn dex(&mut self) {
       let result = (self.x as i8) - 1;

       self.set_flag(result == 0, ZERO_RESULT_MASK);
       self.set_flag(result < 0, NEGATIVE_MASK);

       self.x = (result as u8);
    }

    fn cpy(&mut self, operand: u8) {
        let result = (self.y as i8) - (operand as i8);

        self.set_flag(result == 0, ZERO_RESULT_MASK);
        self.set_flag(result < 0, NEGATIVE_MASK);
        self.set_flag(result >= 0, CARRY_MASK);
    }

    fn cpx(&mut self, operand: u8) {
        let result = (self.x as i8) - (operand as i8);

        self.set_flag(result == 0, ZERO_RESULT_MASK);
        self.set_flag(result < 0, NEGATIVE_MASK);
        self.set_flag(result >= 0, CARRY_MASK);
    }

    fn cmp(&mut self, operand: u8) {
        let result = (self.a as i8) - (operand as i8);

        self.set_flag(result == 0, ZERO_RESULT_MASK);
        self.set_flag(result < 0, NEGATIVE_MASK);
        self.set_flag(result >= 0, CARRY_MASK);
    }

    fn clv(&mut self) {
        self.set_flag(false, OVERFLOW_MASK);
    }

    fn cli(&mut self) {
        self.set_flag(false, INTERRUPT_DISABLE_MASK);
    }

    fn cld(&mut self) {
        self.set_flag(false, DECIMAL_MASK);
    }

    fn clc(&mut self) {
        self.set_flag(false, CARRY_MASK);
    }

    fn bvs(&mut self, operand: u8) {
        self.branch_on_flag(operand, true, OVERFLOW_MASK);
    }
    
    fn bvc(&mut self, operand: u8) {
        self.branch_on_flag(operand, false, OVERFLOW_MASK);
    }

    fn bpl(&mut self, operand: u8) {
        self.branch_on_flag(operand, false, NEGATIVE_MASK);
    }

    fn bne(&mut self, operand: u8) {
        self.branch_on_flag(operand, false, ZERO_RESULT_MASK);
    }

    fn bmi(&mut self, operand: u8) {
        self.branch_on_flag(operand, true, NEGATIVE_MASK);
    }

    fn bit(&mut self, operand: u8) {
        let result = self.a & operand;
        
        println!("Result is {}", result);

        self.set_flag(result == 0, ZERO_RESULT_MASK);
        self.set_flag((result & OVERFLOW_MASK) > 0, OVERFLOW_MASK);
        self.set_flag((result & NEGATIVE_MASK) > 0, NEGATIVE_MASK);
    }

    fn beq(&mut self, operand: u8) {
        self.branch_on_flag(operand, true, ZERO_RESULT_MASK);
    }

    fn bcs(&mut self, operand: u8) {
        self.branch_on_flag(operand, true, CARRY_MASK);
    }

    fn bcc(&mut self, operand: u8) {
        self.branch_on_flag(operand, false, CARRY_MASK);
    }

    fn asl(&mut self, operand: u8, address_mode: &AddressMode) {
        // carry check
        self.set_flag((operand >> 7) == 1, CARRY_MASK);

        let temp = operand << 1;

        // negative check
        self.set_flag((temp >> 7) == 1, NEGATIVE_MASK);

        // zero check
        self.set_flag(temp == 0, ZERO_RESULT_MASK);

        if let &AddressMode::Accumulator = address_mode {
                self.a = temp;
        }
    }

    fn and(&mut self, operand: u8) {
        self.a = self.a & operand;
       
        let temp = self.a;

        // negative check
        self.set_flag((temp >> 7) == 1, NEGATIVE_MASK);

        // zero check
        self.set_flag(temp == 0, ZERO_RESULT_MASK);
    }

    fn adc(&mut self, operand: u8) {
        
        let carry = CARRY_MASK & self.flags;

        if self.flag_set(DECIMAL_MASK) {
            let low_nibble;
            let high_nibble;
            let mut nibble_carry;

            // TODO - do I need to validate the nibbles? What does
            // the 6507 do when it has invalid BCD operands?

            let mut temp = (LOW_NIBBLE_MASK & self.a) + (LOW_NIBBLE_MASK & operand) + carry;
            
            if temp <= 9 {
                low_nibble = temp;
                nibble_carry = 0;
            } else {
                low_nibble = (temp + 6) & LOW_NIBBLE_MASK; // BCD correction by adding 6
                nibble_carry = 1;
            };

            temp = (self.a >> 4) + (operand >> 4) + nibble_carry;
            
            if temp <= 9 {
                high_nibble = temp;
                nibble_carry = 0;
            } else {
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
    fn and() {
        let mut cpu = Mos6507::new();
        cpu.a = 1;

        cpu.and(1);

        assert_eq!(cpu.a, 1);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 

    #[test]
    fn and_zero() {
        let mut cpu = Mos6507::new();
        cpu.a = 0;

        cpu.and(0);

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
    } 

    #[test]
    fn and_negative() {
        let mut cpu = Mos6507::new();
        cpu.a = 128; // 128 unsigned has leftmost (negative) bit 1

        cpu.and(128); // 128 unsigned has leftmost (negative) bit 1

        assert_eq!(cpu.a, 128);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 

    #[test]
    fn asl() {
        let mut cpu = Mos6507::new();
        cpu.a = 128; // 128 unsigned has leftmost (negative) bit 1

        cpu.asl(128, &super::AddressMode::Accumulator);

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
    } 
    
    #[test]
    fn asl_no_carry_no_negative_no_zero() {
        let mut cpu = Mos6507::new();
        cpu.a = 32;

        cpu.asl(32, &super::AddressMode::Accumulator);

        assert_eq!(cpu.a, 64);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 
    
    #[test]
    fn asl_no_carry_negative_no_zero() {
        let mut cpu = Mos6507::new();
        cpu.a = 64;

        cpu.asl(64, &super::AddressMode::Accumulator);

        assert_eq!(cpu.a, 128);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 
    
    #[test]
    fn asl_no_carry_no_negative_zero() {
        let mut cpu = Mos6507::new();
        cpu.a = 0;

        cpu.asl(0, &super::AddressMode::Accumulator);

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
    } 
   
    #[test]
    fn asl_no_accumulator() {
        let mut cpu = Mos6507::new();
        cpu.a = 10;

        cpu.asl(192, &super::AddressMode::ZeroPage);

        assert_eq!(cpu.a, 10);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 

    #[test]
    fn bcc() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;

        cpu.bcc(operand);

        assert_eq!(cpu.pc, 127);
    }
    
    #[test]
    fn bcc_carry_flag_set() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;
        cpu.set_flag(true, super::CARRY_MASK);
        
        cpu.bcc(operand);

        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn bcc_negative_operand() {
        let mut cpu = Mos6507::new();
        let operand: i8 = -127;
        cpu.pc = 128;

        cpu.bcc(operand as u8);

        assert_eq!(cpu.pc, 1);
    }

    #[test]
    fn bcs() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;
        cpu.set_flag(true, super::CARRY_MASK);

        cpu.bcs(operand);

        assert_eq!(cpu.pc, 127);
    }
    
    #[test]
    fn bcs_carry_flag_not_set() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;
        
        cpu.bcs(operand);

        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn bcs_negative_operand() {
        let mut cpu = Mos6507::new();
        let operand: i8 = -127;
        cpu.pc = 128;
        cpu.set_flag(true, super::CARRY_MASK);

        cpu.bcs(operand as u8);

        assert_eq!(cpu.pc, 1);
    }

    #[test]
    fn beq() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;
        cpu.set_flag(true, super::ZERO_RESULT_MASK);

        cpu.beq(operand);

        assert_eq!(cpu.pc, 127);
    }
    
    #[test]
    fn beq_zero_flag_not_set() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;
        
        cpu.beq(operand);

        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn beq_negative_operand() {
        let mut cpu = Mos6507::new();
        let operand: i8 = -127;
        cpu.pc = 128;
        cpu.set_flag(true, super::ZERO_RESULT_MASK);

        cpu.beq(operand as u8);

        assert_eq!(cpu.pc, 1);
    }

    #[test]
    fn bit() {
        let mut cpu = Mos6507::new();
        let operand = 0xC0; // This value has bits 6/7 set
        cpu.a = 0xF0; // This value as 6/7 bits set

        cpu.bit(operand);

        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
    }

    #[test]
    fn bmi() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;
        cpu.flags = super::NEGATIVE_MASK;

        cpu.bmi(operand);

        assert_eq!(cpu.pc, 127);
    }
    
    #[test]
    fn bmi_negative_flag_not_set() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;
        
        cpu.bmi(operand);

        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn clc() {
        let mut cpu = Mos6507::new();

        cpu.clc();

        assert_eq!(cpu.flags, 0);
    }

    #[test]
    fn clc_carry_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.flags = super::CARRY_MASK;

        cpu.clc();

        assert_eq!(cpu.flags, 0);
    }

    #[test]
    fn clc_carry_and_other_flags_set() {
        let mut cpu = Mos6507::new();
        cpu.flags = super::CARRY_MASK | super::ZERO_RESULT_MASK | super::INTERRUPT_DISABLE_MASK;
        
        cpu.clc();

        assert_eq!(cpu.flags, super::ZERO_RESULT_MASK | super::INTERRUPT_DISABLE_MASK);
    }

    #[test]
    fn cld() {
        let mut cpu = Mos6507::new();

        cpu.cld();

        assert_eq!(cpu.flags, 0);
    }

    #[test]
    fn cld_decimal_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.flags = super::DECIMAL_MASK;

        cpu.cld();

        assert_eq!(cpu.flags, 0);
    }

    #[test]
    fn cld_decimal_and_other_flags_set() {
        let mut cpu = Mos6507::new();
        cpu.flags = super::DECIMAL_MASK | super::ZERO_RESULT_MASK | super::INTERRUPT_DISABLE_MASK;
        
        cpu.cld();

        assert_eq!(cpu.flags, super::ZERO_RESULT_MASK | super::INTERRUPT_DISABLE_MASK);
    }

    #[test]
    fn cli() {
        let mut cpu = Mos6507::new();

        cpu.cli();

        assert_eq!(cpu.flags, 0);
    }

    #[test]
    fn cli_interrupt_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.flags = super::INTERRUPT_DISABLE_MASK;

        cpu.cli();

        assert_eq!(cpu.flags, 0);
    }

    #[test]
    fn cli_interrupt_and_other_flags_set() {
        let mut cpu = Mos6507::new();
        cpu.flags = super::DECIMAL_MASK | super::ZERO_RESULT_MASK | super::INTERRUPT_DISABLE_MASK;
        
        cpu.cli();

        assert_eq!(cpu.flags, super::DECIMAL_MASK | super::ZERO_RESULT_MASK);
    }

    #[test]
    fn clv() {
        let mut cpu = Mos6507::new();

        cpu.clv();

        assert_eq!(cpu.flags, 0);
    }

    #[test]
    fn clv_overflow_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.flags = super::OVERFLOW_MASK;

        cpu.clv();

        assert_eq!(cpu.flags, 0);
    }

    #[test]
    fn clv_interrupt_and_other_flags_set() {
        let mut cpu = Mos6507::new();
        cpu.flags = super::OVERFLOW_MASK | super::ZERO_RESULT_MASK | super::INTERRUPT_DISABLE_MASK;
        
        cpu.clv();

        assert_eq!(cpu.flags, super::ZERO_RESULT_MASK | super::INTERRUPT_DISABLE_MASK);
    }

    #[test]
    fn bit_zero_flag_set() {
        let mut cpu = Mos6507::new();
        let operand = 0;
        cpu.a = 0;

        cpu.bit(operand);

        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
        assert_eq!(cpu.flag_set(super::OVERFLOW_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
    }
    
    #[test]
    fn bne() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;

        cpu.bne(operand);

        assert_eq!(cpu.pc, 127);
    }
    
    #[test]
    fn bne_zero_flag_not_set() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;
       
        cpu.set_flag(true, super::ZERO_RESULT_MASK);

        cpu.bne(operand);

        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn bne_negative_operand() {
        let mut cpu = Mos6507::new();
        let operand: i8 = -127;
        cpu.pc = 128;

        cpu.bne(operand as u8);

        assert_eq!(cpu.pc, 1);
    }

    #[test]
    fn bpl() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;

        cpu.bpl(operand);

        assert_eq!(cpu.pc, 127);
    }

    #[test]
    fn bpl_negative_flag_set() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;
        cpu.set_flag(true, super::NEGATIVE_MASK);

        cpu.bpl(operand);

        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn bvc() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;

        cpu.bvc(operand);

        assert_eq!(cpu.pc, 127);
    }

    #[test]
    fn bvc_overflow_flag_set() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;
        cpu.set_flag(true, super::OVERFLOW_MASK);

        cpu.bvc(operand);

        assert_eq!(cpu.pc, 0);
    }
    
    #[test]
    fn bvs() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;
        cpu.set_flag(true, super::OVERFLOW_MASK);
        
        cpu.bvs(operand);

        assert_eq!(cpu.pc, 127);
    }

    #[test]
    fn bvs_overflow_flag_not_set() {
        let mut cpu = Mos6507::new();
        let operand: u8 = 127;
        cpu.pc = 0;

        cpu.bvs(operand);

        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn cmp() {
        let mut cpu = Mos6507::new();
        cpu.a = 128;
        
        cpu.cmp(128);

        assert_eq!(cpu.a, 128);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
    }
    
    #[test]
    fn cmp_negative_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.a = 32;

        cpu.cmp(33);

        assert_eq!(cpu.a, 32);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 
   
    #[test]
    fn cpx() {
        let mut cpu = Mos6507::new();
        cpu.x = 100;
        
        cpu.cpx(100);

        assert_eq!(cpu.x, 100);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
    }
    
    #[test]
    fn cpx_negative_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.x = 32;

        cpu.cpx(33);

        assert_eq!(cpu.x, 32);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 
    
    #[test]
    fn cpx_carry_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.x = 33;

        cpu.cpx(32);

        assert_eq!(cpu.x, 33);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 

    #[test]
    fn cpy() {
        let mut cpu = Mos6507::new();
        cpu.y = 100;
        
        cpu.cpy(100);

        assert_eq!(cpu.y, 100);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
    }
    
    #[test]
    fn cpy_negative_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.y = 32;

        cpu.cpy(33);

        assert_eq!(cpu.y, 32);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), false);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 
    
    #[test]
    fn cpy_carry_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.y = 33;

        cpu.cpy(32);

        assert_eq!(cpu.y, 33);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 

    #[test]
    fn cmp_carry_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.a = 33;

        cpu.cmp(32);

        assert_eq!(cpu.a, 33);
        assert_eq!(cpu.flag_set(super::CARRY_MASK), true);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 
 
    #[test]
    fn dex() {
        let mut cpu = Mos6507::new();
        cpu.x = 23;

        cpu.dex();

        assert_eq!(cpu.x, 22);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 
   
    #[test]
    fn dex_zero_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.x = 1;

        cpu.dex();

        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
    } 
   
    #[test]
    fn dex_negative_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.x = 0;

        cpu.dex();

        assert_eq!(cpu.x, 255);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 

    #[test]
    fn inc() {
        let mut cpu = Mos6507::new();
        cpu.x = 23;

        cpu.inc();

        assert_eq!(cpu.x, 24);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
    } 
   
    #[test]
    fn inc_zero_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.x = 255;

        cpu.inc();

        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), false);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), true);
    } 
   
    #[test]
    fn inc_negative_flag_set() {
        let mut cpu = Mos6507::new();
        cpu.x = 254;

        cpu.inc();

        assert_eq!(cpu.x, 255);
        assert_eq!(cpu.flag_set(super::NEGATIVE_MASK), true);
        assert_eq!(cpu.flag_set(super::ZERO_RESULT_MASK), false);
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
