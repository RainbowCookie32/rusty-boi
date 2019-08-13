use std::io;
use std::io::prelude::*;
use std::fs::File;

mod cpu;
mod utils;

fn main() {
    
    let mut rom_path = String::new();
    let mut bootrom_path = String::new();
    let bootrom: Vec<u8>;
    let rom: Vec<u8>;
    
    println!("Rusty Boi");
    println!("Point me to a Gameboy Bootrom");

    io::stdin().read_line(&mut bootrom_path).expect("Error while reading path to ROM");
    bootrom_path = bootrom_path.trim().to_string();
    bootrom = load_bootrom(bootrom_path);

    println!("Point me to a GameBoy ROM");

    io::stdin().read_line(&mut rom_path).expect("Error while reading path to Bootrom");
    rom_path = rom_path.trim().to_string();
    rom = load_rom(rom_path);

    cpu::init_cpu(bootrom, rom);
    
}

fn load_bootrom(path: String) -> Vec<u8> {
    
    let mut rom_file = File::open(path).expect("Failed to open Bootrom");
    let mut data = Vec::new();

    match rom_file.read_to_end(&mut data){
        Ok(_) => println!("Bootrom loaded"),
        Err(_) => println!("Failed to open the Bootrom file"),
    };

    data
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
