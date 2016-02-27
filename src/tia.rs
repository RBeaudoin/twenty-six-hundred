use std::collections::HashMap;

pub struct Tia1A {
    registers: HashMap<u16, u8>,
}

impl Tia1A {
    pub fn new() -> Tia1A {
        Tia1A {
            registers: HashMap::new(),
        }
    }
}
