use std::fs::File;
use std::io::Read;

pub struct Mos6507 {
    program: Vec<u8>,
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    flags: u8,
}

impl Mos6507 {
    pub fn new(program: Vec<u8>) -> Mos6507 {
        Mos6507 {
            program: program,
            a: 0u8,
            x: 0u8,
            y: 0u8,
            sp: 0u8,
            pc: 0u16,
            flags: 0u8, //TODO - set flags to appropriate defaults
        }
    }

    pub fn run(&self) -> Result<i32,()> {
        let mut bytes = self.program.bytes();
        
        loop  {
            
            match bytes.next() {
                Some(opcode)    => {
                    
                    match opcode {
                        //ADC - Add with carry
                        Ok(0x69) => self.adc_immediate(bytes.next().unwrap().unwrap()),
                        Ok(0x65) => self.adc_zero_page(bytes.next().unwrap().unwrap()),
                        Ok(0x75) => self.adc_zero_page_x(bytes.next().unwrap().unwrap()),
                        Ok(0x6D) => self.adc_absolute(  bytes.next().unwrap().unwrap(), 
                                                        bytes.next().unwrap().unwrap()),
                        Ok(0x7D) => self.adc_absolute_x(bytes.next().unwrap().unwrap(), 
                                                        bytes.next().unwrap().unwrap()),
                        Ok(0x79) => self.adc_absolute_y(bytes.next().unwrap().unwrap(), 
                                                        bytes.next().unwrap().unwrap()),
                        Ok(0x61) => self.adc_indirect_x(bytes.next().unwrap().unwrap()),
                        Ok(0x71) => self.adc_indirect_y(bytes.next().unwrap().unwrap()),
                        //AND
                        //ASL
                        //BCC
                        //BCS
                        //BEQ
                        //BIT
                        //BMI
                        //BNE
                        //BPL
                        //BRK
                        //BVC
                        //BVS
                        //CLC
                        //CLD
                        //CLI
                        //CLV
                        //CMP
                        //CPX
                        //CPY
                        //DEC
                        //DEX
                        //DEY
                        //EOR
                        //INC
                        //INX
                        //INY
                        //JMP
                        //JSR
                        //LDA
                        //LDX
                        //LDY
                        //LSR
                        //NOP
                        //ORA
                        //PHA
                        //PHP
                        //PLA
                        //PLP
                        //ROL
                        //ROR
                        //RTI
                        //RTS
                        //SBS
                        //SEC
                        //SED
                        //SEI
                        //STA
                        //STX
                        //STY
                        //TAX
                        //TAY
                        //TYA
                        //TSX
                        //TXA
                        //TXS
                        Ok(x) => println!("Ignoring opcode {}", x),
                        Err(err) => return Err(()), //TODO better error handling
                    }
                },
                None            => break, //Program done executing
            }
        }
        Ok(0)
    }

    fn adc_immediate(&self, operand: u8) {
        
    }
    
    fn adc_zero_page(&self, operand: u8) {
    
    }

    fn adc_zero_page_x(&self, operand: u8) {
    
    }

    fn adc_absolute(&self, operand1: u8, operand2: u8) {
    
    }

    fn adc_absolute_x(&self, operand1: u8, operand2: u8) {
    
    }

    fn adc_absolute_y(&self, operand1: u8, operand2: u8) {
    
    }

    fn adc_indirect_x(&self, operand: u8) {
    
    }

    fn adc_indirect_y(&self, operand: u8) {
    
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
