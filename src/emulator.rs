use std::thread;
use std::io::Read;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use log::info;
use log::error;

use super::cpu;
use super::cart::CartData;
use super::memory::init_memory;
use super::memory::{RomMemory, CpuMemory, GpuMemory};


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

pub struct EmuInit {

    pub cpu: (Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>),
    pub gpu: (Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>),

    pub cycles_arc: Arc<Mutex<u16>>,
    pub input_tx: Sender<InputEvent>,
    pub input_rx: Receiver<InputEvent>,
}


pub fn initialize(path: &PathBuf) -> EmuInit {

    let rom_data = get_roms_data(&path);
    let mem_arcs = init_memory(rom_data);
    let (tx, rx) = mpsc::channel();

    let gpu_arc = (Arc::clone(&mem_arcs.1), Arc::clone(&mem_arcs.2));

    EmuInit {
        cpu: (mem_arcs.0, mem_arcs.1, mem_arcs.2),
        gpu: gpu_arc,

        cycles_arc: Arc::new(Mutex::new(0 as u16)),
        input_tx: tx,
        input_rx: rx,
    }
}

pub fn start_emulation(arcs: &EmuInit, input: Receiver<InputEvent>) {
        
    let cpu_cycles = Arc::clone(&arcs.cycles_arc);
    let cpu_arc = (Arc::clone(&arcs.cpu.0), Arc::clone(&arcs.cpu.1), Arc::clone(&arcs.cpu.2));

    let _cpu_thread = thread::Builder::new().name("cpu_thread".to_string()).spawn(move || {
        cpu::cpu_loop(cpu_cycles, cpu_arc, input);
    }).unwrap();

    info!("Emu: Stopped emulation.");
}

fn get_roms_data(rom_path: &PathBuf) -> (Vec<u8>, CartData) {

    let bootrom: Vec<u8>;
    let rom: CartData;
    
    bootrom = load_bootrom();
    rom = load_rom(&rom_path);

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

fn load_rom(rom_path: &PathBuf) -> CartData {
    
    let mut rom_file = File::open(rom_path).expect("Loader: Failed to open ROM");
    let mut data = Vec::new();

    match rom_file.read_to_end(&mut data){
        Ok(_) => info!("Loader: ROM loaded"),
        Err(_) => error!("Loader: Failed to open the ROM file"),
    };

    CartData::new(data)
}