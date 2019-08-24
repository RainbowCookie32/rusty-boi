extern crate sdl2;

mod cpu;
mod gpu;
mod register;

mod utils;
mod emulator;

mod opcodes;
mod opcodes_prefixed;


fn main() {
    
    println!("Rusty Boi");
    emulator::init_emu();
    
}