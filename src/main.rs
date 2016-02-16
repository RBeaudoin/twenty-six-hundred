extern crate twenty_six;

use std::env;

fn main() {

    //let x = &env::args().collect::<Vec<String>>()[1];
    let x = env::args().collect::<Vec<String>>();
    
    println!("Argument is: {}", x[1]);

    for argument in env::args() {
        println!("{}", argument);
    }

    println!("TODO: add the code here");
}
