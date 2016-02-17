extern crate twenty_six;

use std::env;
use std::fs::File;
use std::io::Read;
use twenty_six::Atari2600;

fn main() {

    //TODO - error checking on # of args, etc.
    let args = env::args().collect::<Vec<String>>();
    let input_file = File::open(&args[1]).unwrap();
   
    let cmos_6507 =  Atari2600::new(input_file.bytes());

    println!("TODO: add the code here");
}
