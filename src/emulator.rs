use std::io;
use std::thread;
use std::io::Read;
use std::fs::File;
use std::sync::mpsc;

use log::info;
use log::error;

use super::cpu;
use super::gpu;
use super::timer;
use super::memory::start_memory;


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
    // TODO: Double check, does the GameBoy care? Does it trigger
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

pub fn init_emu() {

    execution_loop();
}

fn execution_loop() {
    
    let rom_data = get_roms_data();

    let (cycles_tx, cycles_rx) = mpsc::channel();
    let (timer_cycles_tx, timer_cycles_rx) = mpsc::channel();
    let (mem_init_tx, mem_init_rx) = mpsc::channel();
    let (input_tx, input_rx) = mpsc::channel();

    let _memory_thread = thread::Builder::new().name("memory_thread".to_string()).spawn(move || {
        start_memory(rom_data, mem_init_tx);
    }).unwrap();

    let mem_channels = mem_init_rx.recv().unwrap();
    let cpu_channels = mem_channels.cpu;
    let gpu_channels = mem_channels.gpu;
    let timer_channels = mem_channels.timer;

    let _cpu_thread = thread::Builder::new().name("cpu_thread".to_string()).spawn(move || {
        cpu::exec_loop(cycles_tx, timer_cycles_tx, cpu_channels);
    }).unwrap();

    let _gpu_thread = thread::Builder::new().name("gpu_thread".to_string()).spawn(move || {
        gpu::start_gpu(cycles_rx, gpu_channels, input_tx);
    }).unwrap();

    let _timer_thread = thread::Builder::new().name("timer_thread".to_string()).spawn(move || {
        timer::timer_loop(timer_cycles_rx, timer_channels);
    }).unwrap();

    loop {

        let input_event = input_rx.try_recv();
        let received_message: InputEvent;

        match input_event {
            Ok(result) => {

                received_message = result;
                match received_message {
                    InputEvent::Quit => break,
                    InputEvent::APressed => { info!("Emu: Pressed A") },
                    InputEvent::AReleased => { info!("Emu: Released A") },
                    InputEvent::BPressed => { info!("Emu: Pressed B") },
                    InputEvent::BReleased => { info!("Emu: Released B") },
                    InputEvent::UpPressed => { info!("Emu: Pressed Up") },
                    InputEvent::UpReleased => { info!("Emu: Released Up") },
                    InputEvent::DownPressed => { info!("Emu: Pressed Down") },
                    InputEvent::DownReleased => { info!("Emu: Released Down") },
                    InputEvent::LeftPressed => { info!("Emu: Pressed Left") },
                    InputEvent::LeftReleased => { info!("Emu: Released Left") },
                    InputEvent::RightPressed => { info!("Emu: Pressed Right") },
                    InputEvent::RightReleased => { info!("Emu: Released Right") },
                    InputEvent::StartPressed => { info!("Emu: Pressed Start") },
                    InputEvent::StartReleased => { info!("Emu: Released Start") },
                    InputEvent::SelectPressed => { info!("Emu: Pressed Select") },
                    InputEvent::SelectReleased => { info!("Emu: Released Select") },
                }
            },
            Err(_error) => {}
        };
    }
    
    info!("Emu: Stopped emulation.");
}

fn get_roms_data() -> (Vec<u8>, Vec<u8>) {

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

    (bootrom, rom)
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