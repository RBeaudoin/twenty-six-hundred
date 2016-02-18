use std::fs::File;
use std::io::Read;

pub struct Mos6507 {
    program: Vec<u8>,
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u8,
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
            pc: 0u8,
            flags: 0u8,
        }
    }

    pub fn run(&self) -> Result<i32,()> {
        println!("Mos6507: Running program");
        
        let mut bytes = self.program.bytes();
        
        loop  {
            
            match bytes.next() {
                Some(opcode)    => {
                    
                    match opcode {
                        //ADC
                        Ok(0x29) => self.adc_immediate(bytes.next().unwrap().unwrap()),
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
                        Ok(x) => println!("Ignoring opcode {}", x),
                        Err(err) => return Err(()), //TODO better error handling
                    }
                },
                None            => break, //Program done executing
            }
        }

        Ok(1)
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
