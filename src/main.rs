mod cpu;
mod gpu;
mod cart;
mod timer;
mod utils;
mod memory;
mod register;
mod emulator;

mod opcodes;
mod opcodes_prefixed;

use log::info;


fn main() {

    // Initialize the logger
    simple_logger::init_with_level(log::Level::Info).unwrap();
    info!("Rusty Boi");

    emulator::initialize();
}