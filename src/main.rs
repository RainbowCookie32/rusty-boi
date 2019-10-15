mod cpu;
mod gpu;
mod timer;
mod memory;
mod register;
mod renderer;
mod utils;
mod emulator;

mod opcodes;
mod opcodes_prefixed;

use log::info;


fn main() {

    // Initialize the logger
    simple_logger::init_with_level(log::Level::Info).unwrap();
    info!("Rusty Boi");

    renderer::init_renderer();
}