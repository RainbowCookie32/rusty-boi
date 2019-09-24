extern crate sdl2;
extern crate log;
extern crate simple_logger;

mod cpu;
mod gpu;
mod memory;
mod register;

mod utils;
mod emulator;

mod opcodes;
mod opcodes_prefixed;

use log::info;

fn main() {

    simple_logger::init_with_level(log::Level::Info).unwrap();
    info!("Rusty Boi");
    emulator::init_emu();
}