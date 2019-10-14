use std::thread;
use std::io::Read;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::iter::FromIterator;

use log::info;
use log::error;

use super::cpu;
use super::memory::init_memory;
use super::memory::{RomMemory, CpuMemory, GpuMemory};

pub struct Cart {

    pub cart_type: u8,
    pub rom_size: u8,
    pub ram_size: u16,

    pub rom_banks: Vec<Vec<u8>>,
    pub cart_ram: Vec<u8>,
    pub has_ram: bool,
}

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

    // Buttons being released.
    // TODO: Double check, how much does the GameBoy care? Does it trigger
    // another interrupt, or it's just a value change in the I/O register?
    AReleased,
    BReleased,
    UpReleased,
    DownReleased,
    LeftReleased,
    RightReleased,
    StartReleased,
    SelectReleased,
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

pub fn start_emulation(arcs: &EmuInit) {
        
    let cpu_cycles = Arc::clone(&arcs.cycles_arc);
    let cpu_arc = (Arc::clone(&arcs.cpu.0), Arc::clone(&arcs.cpu.1), Arc::clone(&arcs.cpu.2));

    let _cpu_thread = thread::Builder::new().name("cpu_thread".to_string()).spawn(move || {
        cpu::cpu_loop(cpu_cycles, cpu_arc);
    }).unwrap();

    info!("Emu: Stopped emulation.");
}

fn get_roms_data(rom_path: &PathBuf) -> (Vec<u8>, Cart) {

    let bootrom: Vec<u8>;
    let rom: Cart;
    
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

fn load_rom(rom_path: &PathBuf) -> Cart {
    
    let mut rom_file = File::open(rom_path).expect("Loader: Failed to open ROM");
    let mut data = Vec::new();

    match rom_file.read_to_end(&mut data){
        Ok(_) => info!("Loader: ROM loaded"),
        Err(_) => error!("Loader: Failed to open the ROM file"),
    };

    make_cart(&data)
}

fn make_cart(cart_data: &Vec<u8>) -> Cart {

    let loaded_cart: Cart;

    let cart_type = cart_data[0x0147];
    let rom_size = match cart_data[0x148] {
        0x0 => 2,
        0x1 => 4,
        0x2 => 8,
        0x3 => 16,
        0x4 => 32,
        0x5 => 64,
        0x6 => 128,
        _ => 2,
    };
    let ram_size: u16 = match cart_data[0x149] {
        0 => 0,
        1 => 2048,
        2 => 8192,
        3 => 32768,
        _ => 0,
    };

    let mut rom_banks: Vec<Vec<u8>> = vec![Vec::new(); rom_size];
    let mut loaded_banks: usize = 0;
    let has_ram: bool;
    let mut ram: Vec<u8> = Vec::new();

    while loaded_banks < rom_size {

        if loaded_banks == 0 {
            let bank = Vec::from_iter(cart_data[0..16384].iter().cloned());
            rom_banks[loaded_banks] = bank;
            loaded_banks += 1;
        }
        else {
            let bank = Vec::from_iter(cart_data[16384 * loaded_banks..(16384 * loaded_banks) + 16384].iter().cloned());
            rom_banks[loaded_banks] = bank;
            loaded_banks += 1;
        }
    }

    // TODO: Should also check if it's RAM + battery, since that
    // means that it has data meant to be saved when the cart is removed.
    // Should also add a way to save the RAM contents, to load them later.
    if ram_size > 0 {
        ram = vec![0; ram_size as usize];
        has_ram = true;
    }
    else {
        has_ram = false;
    }

    loaded_cart = Cart {
        cart_type: cart_type,
        rom_size: rom_size as u8,
        ram_size: ram_size,
        rom_banks: rom_banks,
        cart_ram: ram,
        has_ram: has_ram,
    };

    loaded_cart
}