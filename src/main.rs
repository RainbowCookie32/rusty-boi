use std::io;
use std::io::prelude::*;
use std::fs::File;

mod cpu;

fn main() {
    
    let mut rom_path = String::new();
    
    println!("Rusty Boi");
    println!("Point me to a GameBoy ROM");
    
    io::stdin().read_line(&mut rom_path).expect("Error while reading path to ROM");

    rom_path = rom_path.trim().to_string();
    cpu::init_cpu(load_rom(rom_path));
}

fn load_rom(path: String) -> Vec<u8> {
    
    let mut rom_file = File::open(path).expect("Failed to open ROM");
    let mut data = Vec::new();

    match rom_file.read_to_end(&mut data){
        Ok(_) => println!("ROM loaded"),
        Err(_) => println!("Failed to open the ROM file"),
    };

    data
}
