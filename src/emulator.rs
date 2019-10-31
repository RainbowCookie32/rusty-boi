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

    let rom_data = load_roms();
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

fn load_roms() -> (Vec<u8>, CartData) {

    let bootrom: Vec<u8>;
    let rom: CartData;
    
    bootrom = load_bootrom();
    rom = load_rom();

    (bootrom, rom)
}

fn load_bootrom() -> Vec<u8> {
    
    let mut rom_file = File::open("Bootrom.gb").expect("Loader: Failed to open Bootrom");
    let mut data = Vec::new();

    match rom_file.read_to_end(&mut data){
        Ok(_) => info!("Loader: Bootrom loaded"),
        Err(_) => error!("Loader: Failed to open the Bootrom file"),
    };

    data
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