use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use log::info;
use log::error;

use super::cpu;
use super::gpu;
use super::memory;
use super::memory::VramCheck;
use super::memory::MemoryAccess;
use super::register::CycleCounter;


pub struct ConsoleState {
    pub current_cpu: cpu::CpuState,
    pub current_memory: ((Sender<MemoryAccess>, Receiver<u8>), (Sender<MemoryAccess>, Receiver<u8>, Receiver<VramCheck>)),
}

pub struct Interrupt {
    pub interrupt: bool,
    pub interrupt_type: InterruptType,
}

#[derive(PartialEq, Debug)]
pub enum InterruptType {

    Vblank,
    LcdcStat,
    Timer,
    Serial,
    ButtonPress,
}

pub fn init_emu() {

    let mut rom_path = String::new();
    let mut bootrom_path = String::new();
    let bootrom: Vec<u8>;
    let rom: Vec<u8>;
    
    info!("Emu: Point me to a GameBoy Bootrom");

    io::stdin().read_line(&mut bootrom_path).expect("Loader: Error while reading path to ROM");
    bootrom_path = bootrom_path.trim().to_string();
    bootrom = load_bootrom(bootrom_path);

    info!("Emu: Point me to a GameBoy ROM");

    io::stdin().read_line(&mut rom_path).expect("Loader: Error while reading path to Bootrom");
    rom_path = rom_path.trim().to_string();
    rom = load_rom(rom_path);

    let initial_state = ConsoleState {
        current_cpu: cpu::init_cpu(),
        current_memory: memory::start_memory(bootrom, rom),
    };

    execution_loop(initial_state);
}

fn execution_loop(state: ConsoleState) {
    
    let mut current_state = state;
    let mut cpu_result = cpu::CycleResult::Success;
    let mut interrupt_state = Interrupt { 
        interrupt: false,
        interrupt_type: InterruptType::Vblank,
    };

    let (cycles_tx, cycles_rx) = mpsc::channel();
    let (interrupt_tx, interrupt_rx) = mpsc::channel();

    gpu::gpu_loop((interrupt_tx, cycles_rx), current_state.current_memory.1);

    while cpu_result == cpu::CycleResult::Success || cpu_result == cpu::CycleResult::Stop || cpu_result == cpu::CycleResult::Halt {

        cpu_result = cpu::exec_loop(&mut current_state.current_cpu, &current_state.current_memory.0, &mut interrupt_state);
        cycles_tx.send(current_state.current_cpu.cycles.get()).unwrap();
    }

    info!("CPU: Stopped emulation. Last CPU state was '{:?}'.", cpu_result);
}


fn load_bootrom(path: String) -> Vec<u8> {
    
    let mut rom_file = File::open(path).expect("Loader: Failed to open Bootrom");
    let mut data = Vec::new();

    match rom_file.read_to_end(&mut data){
        Ok(_) => info!("Loader: Bootrom loaded"),
        Err(_) => error!("Loader: Failed to open the Bootrom file"),
    };

    data
}

fn load_rom(path: String) -> Vec<u8> {
    
    let mut rom_file = File::open(path).expect("Loader: Failed to open ROM");
    let mut data = Vec::new();

    match rom_file.read_to_end(&mut data){
        Ok(_) => info!("Loader: ROM loaded"),
        Err(_) => error!("Loader: Failed to open the ROM file"),
    };

    data
}