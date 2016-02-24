extern crate twenty_six;

use std::env;
use std::fs::File;
use std::io::Read;
use twenty_six::Atari2600;
use twenty_six::Cartridge;

fn main() {

    //TODO - error checking on # of args, etc.
    let args = env::args().collect::<Vec<String>>();
    let file = File::open(&args[1]).unwrap();
    
    let cartridge: Cartridge = file.bytes()
                                .map(|x| x.unwrap())
                                .collect(); 
     
    let atari_2600 =  Atari2600::new();
    let exit_code = atari_2600.power_on().unwrap();
    
    println!("Atari2600: Exited with result {}", exit_code);
}
