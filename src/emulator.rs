use std::io;
use std::thread;
use std::io::Read;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::atomic::AtomicU16;

use log::info;
use log::error;

use super::cpu;
use super::video;
use super::cart::CartData;
use super::memory::{Memory, SharedMemory};

pub static GLOBAL_CYCLE_COUNTER: AtomicU16 = AtomicU16::new(0);


#[derive(PartialEq)]
pub enum InputEvent {
    
    // SDL Quit event.
    Quit,

    // Buttons being pressed.
    APressed,
    BPressed,
    UpPressed,
    DownPressed,
    LeftPressed,
    RightPressed,
    StartPressed,
    SelectPressed,
}

pub fn initialize() {
    let shared_memory = Arc::new(SharedMemory::new());
    let cpu_memory = Memory::new(load_bootrom(), load_rom(), shared_memory.clone());
    
    start_emulation(cpu_memory, shared_memory);
}

pub fn start_emulation(cpu_mem: Memory, shared_mem: Arc<SharedMemory>) {
    
    let (input_tx, input_rx) = mpsc::channel();

    let cpu_thread = thread::Builder::new().name("cpu_thread".to_string()).spawn(move || {
        let mut emulated_cpu = cpu::Cpu::new(input_rx, cpu_mem);
        emulated_cpu.execution_loop();
    }).unwrap();

    let _video_thread = thread::Builder::new().name("video_thread".to_string()).spawn(move || {
        let mut emulated_video = video::VideoChip::new(shared_mem, input_tx);
        emulated_video.execution_loop();
    }).unwrap();

    cpu_thread.join().unwrap();

    info!("Emu: Stopped emulation.");
}

fn load_bootrom() -> Option<Vec<u8>> {
    
    match File::open("Bootrom.gb") {
        Ok(file) => {
            let mut bootrom_file = file;
            let mut data = Vec::new();

            let result = match bootrom_file.read_to_end(&mut data) {
                Ok(_) => {
                    info!("Loader: Bootrom loaded");
                    Some(data)
                },
                Err(error) => {
                    error!("Loader: Failed to open the Bootrom file. Error: {}", error);
                    None
                }
            };

            result
        },
        Err(error) => {
            error!("Loader: Failed to open the Bootrom file. Error: {}", error);
            None
        }
    }
}

fn load_rom() -> CartData {
    
    let mut path_str = String::new();
    info!("Loader: Point me to a Gameboy ROM");
    io::stdin().read_line(&mut path_str).expect("Loader: Failed to read ROM path");
    let mut rom_file = File::open(PathBuf::from(path_str.trim())).expect("Loader: Failed to open ROM");
    let mut data = Vec::new();

    match rom_file.read_to_end(&mut data){
        Ok(_) => info!("Loader: ROM loaded"),
        Err(_) => error!("Loader: Failed to open the ROM file"),
    };

    CartData::new(data)
}