mod bsp;
mod error;
mod trace;

use bsp::*;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("opening map: {}", args[1]);
    let map = BSP::open(args[1].to_owned()).unwrap();
    trace::is_visible(&map, [0f32; 3], [10f32; 3]);
}
