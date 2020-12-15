mod cpu;
mod cart;
mod video;
mod memory;

use cpu::Cpu;
use cart::CartData;
use memory::Memory;
use video::VideoChip;

use std::path::*;
use std::fs::File;
use std::sync::Arc;
use std::io::prelude::*;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use log::info;
use simple_logger::SimpleLogger;

// The hash of the bootrom I use for development
const BOOTROM_HASH: u64 = 11527078312544683961;


fn main() {
    let logger = SimpleLogger::new();
    logger.with_level(log::LevelFilter::Info).init().unwrap();

    info!("Rusty Boi");

    // Try to load the bootrom.
    info!("Loader: Looking for bootrom file in emulator's folder...");

    let mut bootrom_contents = get_bootrom();

    if bootrom_contents.is_none() {
        // The emu couldn't find a bootrom file on its folder.
        // Ask the user if they want to provide a custom path.

        log::warn!("Loader: Can't find a known bootrom file on the emu's directory. Do you want to enter a path to it? (y/n)");

        let mut usr_ans = String::new();
        std::io::stdin().read_line(&mut usr_ans).expect("Error while reading answer");
        let answer = usr_ans.trim().to_lowercase();

        if answer == "y" || answer == "yes" {
            info!("Loader: Enter the path to a GameBoy bootrom");
            let mut usr_path = String::new();
            std::io::stdin().read_line(&mut usr_path).expect("Please provide a valid path");
            bootrom_contents = load_file(&PathBuf::from(usr_path.trim()));
        }
        else if answer == "n" || answer == "no" {
            bootrom_contents = None;
        }
        else {
            log::error!("Loader: Invalid answer");
            return;
        }
    }
    
    if bootrom_contents.is_some() {
        info!("Loader: Bootrom loaded");

        let data = bootrom_contents.clone().unwrap();
        let mut hasher = DefaultHasher::new();
        
        data.hash(&mut hasher);
        let hash = hasher.finish();
        
        if hash != BOOTROM_HASH {
            log::warn!("Loader: Loaded bootrom hash doesn't match with known valid bootrom.");
        }
    }
    else {
        info!("Loader: Proceeding without a bootrom, issues can occur");
    }

    info!("Loader: Point me to a GameBoy ROM file!");

    let rom_contents = get_rom();

    if rom_contents.is_none() {
        log::error!("Loader: Failed to read ROM file");
        return;
    }

    let emulated_memory_cpu = Arc::new(Memory::new(bootrom_contents, rom_contents.unwrap()));
    let emulated_memory_video = Arc::clone(&emulated_memory_cpu);

    let _cpu_thread = std::thread::Builder::new().name(String::from("cpu_thread")).spawn(move || {
        let mut emulated_cpu = Cpu::new(emulated_memory_cpu);
        emulated_cpu.execution_loop();
    });

    let video_thread = std::thread::Builder::new().name(String::from("video_thread")).spawn(move || {
        let mut emulated_video = VideoChip::new(emulated_memory_video);
        emulated_video.execution_loop();
    }).expect("Failed to create the video thread");

    video_thread.join().unwrap();
}

fn get_bootrom() -> Option<Vec<u8>> {
    let mut result = load_file(&PathBuf::from("Bootrom.bin"));
    
    if result.is_none() {
        result = load_file(&PathBuf::from("Bootrom.gb"));
    }

    result
}

fn get_rom() -> Option<CartData> {
    let mut rom_path = String::new();

    std::io::stdin().read_line(&mut rom_path).expect("Error while reading path");

    let result = load_file(&PathBuf::from(rom_path.trim()));

    if let Some(data) = result {
        Some(CartData::new(data))
    }
    else {
        None
    }
}

fn load_file(path: &PathBuf) -> Option<Vec<u8>> {
    let file = File::open(path);

    if file.is_ok() {
        let mut file = file.unwrap();
        let mut file_contents = Vec::new();

        let result = file.read_to_end(&mut file_contents);

        if result.is_ok() {
            Some(file_contents)
        }
        else {
            None
        }
    }
    else {
        None
    }
}