mod cpu;
mod cart;
mod timer;
mod video;
mod memory;
mod emulator;

use log::info;


fn main() {

    // Initialize the logger
    simple_logger::init_with_level(log::Level::Info).unwrap();
    info!("Rusty Boi");

    emulator::initialize();
}