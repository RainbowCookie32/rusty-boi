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
use super::gpu;
use super::cart::CartData;
use super::memory;
use super::memory::{CpuMemory, GeneralMemory};


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

    let rom_data = (load_bootrom(), load_rom());
    let mem_arcs = memory::init_memory(rom_data);
    
    start_emulation(mem_arcs.0, mem_arcs.1);
}

pub fn start_emulation(cpu_mem: CpuMemory, shared_mem: Arc<GeneralMemory>) {
        
    let cpu_cycles = Arc::new(AtomicU16::new(0));
    let gpu_cycles = Arc::clone(&cpu_cycles);

    let cpu_memory = (cpu_mem, Arc::clone(&shared_mem));
    let gpu_memory = Arc::clone(&shared_mem);
    
    let (input_tx, input_rx) = mpsc::channel();

    let cpu_thread = thread::Builder::new().name("cpu_thread".to_string()).spawn(move || {
        cpu::start_cpu(cpu_cycles, cpu_memory.0, cpu_memory.1, input_rx);
    }).unwrap();

    let _gpu_thread = thread::Builder::new().name("gpu_thread".to_string()).spawn(move || {
        gpu::start_gpu(gpu_cycles, gpu_memory, input_tx);
    }).unwrap();

    cpu_thread.join().unwrap();

    info!("Emu: Stopped emulation.");
}

fn load_bootrom() -> (Vec<u8>, bool) {
    
    match File::open("Bootrom.gb") {
        Ok(file) => {

            let mut bootrom_file = file;
            let mut data = Vec::new();

            let result = match bootrom_file.read_to_end(&mut data) {
                Ok(_) => {
                    info!("Loader: Bootrom loaded");
                    (data, true)
                },
                Err(error) => {
                    error!("Loader: Failed to open the Bootrom file. Error: {}", error);
                    (Vec::new(), false)
                }
            };

            result
        },
        Err(error) => {
            error!("Loader: Failed to open the Bootrom file. Error: {}", error);
            (Vec::new(), false)
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