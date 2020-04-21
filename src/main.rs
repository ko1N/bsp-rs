mod bsp;
mod error;

use bsp::*;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("opening map: {}", args[1]);
    BSP::open(args[1].to_owned()).unwrap();
}
