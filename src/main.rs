extern crate twenty_six;

use std::env;
use std::fs::File;
use std::io::Read;
use twenty_six::Atari2600;

fn main() {

    //TODO - error checking on # of args, etc.
    let args = env::args().collect::<Vec<String>>();
    println!("args: {}", &args[1]);
    let file = File::open(&args[1]).unwrap();
    
    //TODO is it weird that I can't get all the bytes
    //in the file without iterating over the whole file?
    let bytes: Vec<u8> = file.bytes()
                            .map(|x| x.unwrap())
                            .collect(); 
     
    println!("Byte size is {}", bytes.len());

    let atari_2600 =  Atari2600::new(bytes);
    let exit_code = atari_2600.power_on().unwrap();
    
    println!("Atari2600: Exited with result {}", exit_code);
}
