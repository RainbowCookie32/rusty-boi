use std::io;
use std::thread;
use std::io::Read;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

use log::info;
use log::error;

use super::cpu;
use super::gpu;
use super::cart::CartData;
use super::memory::init_memory;
use super::memory::{CpuMemory, IoRegisters, GpuMemory};


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
    let mem_arcs = init_memory(rom_data);
    
    start_emulation(mem_arcs);
}

pub fn start_emulation(arcs: (CpuMemory, Arc<Mutex<IoRegisters>>, Arc<Mutex<GpuMemory>>)) {
        
    let cpu_cycles = Arc::new(Mutex::new(0 as u16));
    let cpu_arc = (Arc::clone(&arcs.1), Arc::clone(&arcs.2));
    let gpu_arc = (Arc::clone(&arcs.1), Arc::clone(&arcs.2));
    let cycles_gpu = cpu_cycles.clone();
    let (input_tx, input_rx) = mpsc::channel();

    let cpu_thread = thread::Builder::new().name("cpu_thread".to_string()).spawn(move || {
        cpu::start_cpu(cpu_cycles, (arcs.0, cpu_arc.0, cpu_arc.1), input_rx);
    }).unwrap();

    let _gpu_thread = thread::Builder::new().name("gpu_thread".to_string()).spawn(move || {
        gpu::start_gpu(cycles_gpu, input_tx, gpu_arc);
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