use std::io;
use std::io::Read;

use std::thread;
use std::fs::File;
use std::time::Instant;
use std::path::PathBuf;

use std::sync::Arc;
use std::sync::mpsc;
use std::sync::atomic::AtomicU16;

use log::info;
use log::error;

use super::cpu::Cpu;
use super::gpu::Gpu;
use super::cart::CartData;
use super::memory::Memory;


#[derive(Clone, Copy, PartialEq)]
pub enum KeyType {
    A,
    B,
    Up,
    Down,
    Left,
    Right,
    Start,
    Select,
    QuitEvent,
}

#[derive(Clone, Copy)]
pub struct InputEvent {
    button: KeyType,
    pressed_time: Instant,
}

impl InputEvent {
    pub fn new(key: KeyType, time: Instant) -> InputEvent {
        InputEvent {
            button: key,
            pressed_time: time,
        }
    }

    pub fn should_keep(&self) -> bool {
        self.pressed_time.elapsed() < std::time::Duration::from_millis(200)
    }

    pub fn get_event(&self) -> KeyType {
        self.button
    }
}

pub fn initialize() {

    let cart_data = load_rom();
    let bootrom_data = load_bootrom();
    
    let memory = Arc::new(Memory::new(bootrom_data.0, bootrom_data.1, cart_data));
    let arcs = (Arc::clone(&memory), memory);

    start_emulation(arcs);
}

pub fn start_emulation(memory: (Arc<Memory>, Arc<Memory>)) {
        
    let cpu_cycles = Arc::new(AtomicU16::new(0));
    let gpu_cycles = Arc::clone(&cpu_cycles);

    let cpu_memory = memory.0;
    let gpu_memory = memory.1;
    
    let (input_tx, input_rx) = mpsc::channel();

    let cpu_thread = thread::Builder::new().name("cpu_thread".to_string()).spawn(move || {
        let bootrom = cpu_memory.is_bootrom_loaded();
        let mut current_cpu = Cpu::new(cpu_memory, cpu_cycles, input_rx, bootrom);
        current_cpu.execution_loop();
    }).unwrap();

    let _gpu_thread = thread::Builder::new().name("gpu_thread".to_string()).spawn(move || {
        let mut current_gpu = Gpu::new(gpu_cycles, gpu_memory, input_tx);
        current_gpu.execution_loop();
    }).unwrap();

    cpu_thread.join().unwrap();

    info!("Emu: CPU thread finished execution, stopping emulator...");
}

fn load_bootrom() -> (Vec<u8>, bool) {
    
    match File::open("Bootrom.gb") {
        Ok(file) => {

            let mut bootrom_file = file;
            let mut data = Vec::with_capacity(256);

            let result = match bootrom_file.read_to_end(&mut data) {
                Ok(_) => {
                    info!("Loader: Bootrom loaded");
                    (data, true)
                },
                Err(error) => {
                    error!("Loader: Failed to open the Bootrom file. Error: {}. The emulator will continue without it", error);
                    (Vec::new(), false)
                }
            };

            result
        },
        Err(error) => {
            error!("Loader: Failed to open the Bootrom file. Error: {}. The emulator will continue without it", error);
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
        Err(_) => panic!("Loader: Failed to open the ROM file. Can't continue operation"),
    };

    CartData::new(data)
}