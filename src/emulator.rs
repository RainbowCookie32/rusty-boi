use std::io;
use std::io::prelude::*;
use std::fs::File;

use sdl2::pixels::Color;

use super::cpu;
use super::gpu;

pub struct ConsoleState {
    pub current_cpu: cpu::CpuState,
    pub current_gpu: gpu::GpuState,
    pub current_memory: cpu::Memory,
}


pub fn init_emu() {

    let mut rom_path = String::new();
    let mut bootrom_path = String::new();
    let bootrom: Vec<u8>;
    let rom: Vec<u8>;
    
    println!("Point me to a Gameboy Bootrom");

    io::stdin().read_line(&mut bootrom_path).expect("Error while reading path to ROM");
    bootrom_path = bootrom_path.trim().to_string();
    bootrom = load_bootrom(bootrom_path);

    println!("Point me to a GameBoy ROM");

    io::stdin().read_line(&mut rom_path).expect("Error while reading path to Bootrom");
    rom_path = rom_path.trim().to_string();
    rom = load_rom(rom_path);

    let initial_state = ConsoleState {
        current_cpu: cpu::init_cpu(),
        current_gpu: gpu::init_gpu(),
        current_memory: cpu::init_memory(bootrom, rom),
    };

    execution_loop(initial_state);
}

fn execution_loop(state: ConsoleState) {

    let mut current_state = state;

    let mut cpu_result = cpu::CycleResult::Success;

    let sdl_context = sdl2::init().unwrap();
    let sdl_video = sdl_context.video().unwrap();
    let sdl_window = sdl_video.window("Rusty Boi", 160, 144).position_centered().build().unwrap();
    let mut sdl_canvas = sdl_window.into_canvas().build().unwrap();

    sdl_canvas.set_draw_color(Color::RGB(255, 255, 255));
    sdl_canvas.clear();
    sdl_canvas.present();

    while cpu_result == cpu::CycleResult::Success {

        cpu_result = cpu::exec_loop(&mut current_state.current_cpu, &mut current_state.current_memory);
        gpu::gpu_tick(&mut sdl_canvas, &mut current_state.current_gpu, &mut current_state.current_memory, &mut current_state.current_cpu.cycles.value);
    }

    println!("Stopped CPU execution with state {:?}, stopping emulator", cpu_result);
}


fn load_bootrom(path: String) -> Vec<u8> {
    
    let mut rom_file = File::open(path).expect("Failed to open Bootrom");
    let mut data = Vec::new();

    match rom_file.read_to_end(&mut data){
        Ok(_) => println!("Bootrom loaded"),
        Err(_) => println!("Failed to open the Bootrom file"),
    };

    data
}

fn load_rom(path: String) -> Vec<u8> {
    
    let mut rom_file = File::open(path).expect("Failed to open ROM");
    let mut data = Vec::new();

    match rom_file.read_to_end(&mut data){
        Ok(_) => println!("ROM loaded"),
        Err(_) => println!("Failed to open the ROM file"),
    };

    data
}