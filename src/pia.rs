pub struct Pia6532 {
    ram: [u8; 128],
    port1: u8,
    port2: u8,
    pit_timer: i32
}

impl Pia6532 {
    pub fn new() -> Pia6532 {
        Pia6532 {
            ram: [0; 128],            
            port1: 0,            
            port2: 0,
            pit_timer: 0,
        }
    }
}
