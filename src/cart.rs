use log::{error, warn, info};

use std::io;
use std::fs;
use std::path;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::sync::atomic::{AtomicU8, AtomicBool, Ordering};

#[derive(Debug)]
pub enum CartType {

    None,
    MBC1,
    MBC1RAM,
    MBC1RAMBattery,
    MBC2,
    MBC2Battery,
    MBC3,
    MBC3RAM,
    MBC3RAMBattery,
    Other,
}

pub struct CartData {
    
    rom_data: Vec<AtomicU8>,
    ram_data: Vec<AtomicU8>,

    rom_title: String,
    
    has_ram: bool,
    has_battery: bool,
    ram_enabled: AtomicBool,

    selected_rom_bank: AtomicU8,
    selected_ram_bank: AtomicU8,

    rom_banking_mode: AtomicBool,

    mbc: CartType,
}

impl CartData {

    pub fn new(data: Vec<u8>) -> CartData {

        let title = (String::from_utf8(data[308..323].to_vec()).unwrap().trim_matches(char::from(0))).to_string();

        let battery = data[0x0147] == 0x03 || data[0x0147] == 0x06 || data[0x0147] == 0x09 || data[0x0147] == 0x10
        || data[0x0147] == 0x13 || data[0x0147] == 0x1B || data[0x0147] == 0x1E;

        let cart_type = match data[0x0147] {

            0x00 => CartType::None,
            0x01 => CartType::MBC1,
            0x02 => CartType::MBC1RAM,
            0x03 => CartType::MBC1RAMBattery,
            0x05 => CartType::MBC2,
            0x06 => CartType::MBC2Battery,
            0x11 => CartType::MBC3,
            0x12 => CartType::MBC3RAM,
            0x13 => CartType::MBC3RAMBattery,
            _ => CartType::Other,
        };

        let rom_size = match data[0x0148] {
            0x0 => 2,
            0x1 => 4,
            0x2 => 8,
            0x3 => 16,
            0x4 => 32,
            0x5 => 64,
            0x6 => 128,
            _ => 2,
        };

        let ram_size = match data[0x0149] {
            0x0 => 0,
            0x1 => 1,
            0x2 => 1,
            0x3 => 4,
            0x4 => 16,
            0x5 => 8,
            _ => 0,
        };

        let ram_path = path::PathBuf::from(format!("saved_ram/{}.rr", title.to_lowercase()));
        let mut ram_banks: Vec<AtomicU8> = Vec::with_capacity(8192 * ram_size);

        for _item in 0..8192 * ram_size {
            ram_banks.push(AtomicU8::new(0));
        }

        if ram_path.exists() && ram_size > 0 {

            info!("Cart: RAM file found at {:#?}, loading.", ram_path);
            let mut ram_contents: Vec<u8> = Vec::new();
            let mut ram_file = File::open(ram_path).unwrap();
            ram_file.read_to_end(&mut ram_contents).unwrap();

            let mut data_idx: usize = 0;

            for item in ram_contents.iter() {
                ram_banks[data_idx] = AtomicU8::from(*item);
                data_idx += 1;
            }
        }

        let mut data_idx: usize = 0;
        let mut rom_banks: Vec<AtomicU8> = Vec::with_capacity(rom_size);

        for item in data.iter() {
            rom_banks.insert(data_idx, AtomicU8::from(*item));
            data_idx += 1;
        }

        info!("Loader: Cart loaded successfully.");
        println!("\nROM Title: {} \nMBC Type: {:#?} \nROM Size: {} kb \nRAM Size: {}kb\n", title, cart_type, rom_size, ram_size);

        CartData {
            rom_data: rom_banks,
            ram_data: ram_banks,
            rom_title: title.to_lowercase(),
            has_ram: ram_size > 0,
            has_battery: battery,
            ram_enabled: AtomicBool::from(false),
            selected_rom_bank: AtomicU8::from(1),
            selected_ram_bank: AtomicU8::from(0),
            rom_banking_mode: AtomicBool::from(true),
            mbc: cart_type,
        }
    }

    pub fn read(&self, address: u16) -> u8 {

        if address <= 0x3FFF {
            self.rom_data[address as usize].load(Ordering::Relaxed)
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            let bank_offset = 16384 * self.selected_rom_bank.load(Ordering::Relaxed) as usize;
            let address = address as usize - 0x4000 + bank_offset;
            self.rom_data[address].load(Ordering::Relaxed)
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.ram_enabled.load(Ordering::Relaxed) {
                let bank_offset = 8192 * self.selected_ram_bank.load(Ordering::Relaxed) as usize;
                let address = address as usize - 0xA000 + bank_offset;
                self.ram_data[address].load(Ordering::Relaxed)
            }
            else {
                0
            }
        }
        else {
            unreachable!();
        }
    }

    pub fn write(&self, address: u16, value: u8) {
        
        match self.mbc {
            CartType::None => warn!("Memory: Attempting write to cart without a MBC, ignoring."),
            CartType::MBC1 | CartType::MBC1RAM | CartType::MBC1RAMBattery => self.mbc1_write(address, value),
            CartType::MBC2 | CartType::MBC2Battery => self.mbc2_write(address, value),
            CartType::MBC3 | CartType::MBC3RAM | CartType::MBC3RAMBattery => self.mbc3_write(address, value),
            // TODO: At least MBC5 is missing.
            CartType::Other => warn!("Memory: Attempting write to unsupported cart type, ignoring.")
        }
    }

    fn mbc1_write(&self, address: u16, value: u8) {

        if address <= 0x1FFF {
            self.ram_enabled.store((value & 0x0A) == 0x0A, Ordering::Relaxed);
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            let bank = match value {
                0x0 => 0x01,
                0x20 => 0x21,
                0x40 => 0x41,
                0x60 => 0x61,
                _ => value,
            };

            self.selected_rom_bank.store(bank, Ordering::Relaxed);
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            
            if self.ram_enabled.load(Ordering::Relaxed) && self.has_ram {
                let bank_offset = 8192 * self.selected_ram_bank.load(Ordering::Relaxed) as usize;
                let address = address as usize - 0xA000 + bank_offset;
                self.ram_data[address].store(value, Ordering::Relaxed);

                if self.has_battery {
                    self.save_cart_ram();
                }
            }
        }
        else if address >= 0x4000 && address <= 0x5FFF {

            if self.rom_banking_mode.load(Ordering::Relaxed) {
                self.selected_rom_bank.store(value, Ordering::Relaxed);
            }
            else {
                self.selected_ram_bank.store(value, Ordering::Relaxed);
            }
        }
        else if address >= 0x6000 && address <= 0x7FFF {

            self.rom_banking_mode.store(value == 0x1, Ordering::Relaxed);
        }
    }
    
    fn mbc2_write(&self, address: u16, value: u8) {

        if address < 0x1FFF {
            self.ram_enabled.store(value == 0x1, Ordering::Relaxed);
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            let bank = match value {
                0x0 => 0x01,
                0x20 => 0x21,
                0x40 => 0x41,
                0x60 => 0x61,
                _ => value,
            };

            self.selected_rom_bank.store(bank, Ordering::Relaxed);
        }
        else if address >= 0xA000 && address <= 0xA1FF {
            // TODO: Implement MBC2 RAM.
            warn!("Memory: MBC2 RAM is unimplemented, ignoring write.");
        }
    }

    fn mbc3_write(&self, address: u16, value: u8) {

        if address < 0x1FFF {
            // TODO: Also enables R/W to RTC registers.
            self.ram_enabled.store((value & 0x0A) == 0x0A, Ordering::Relaxed);
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            if value == 0x0 {self.selected_rom_bank.store(0x1, Ordering::Relaxed)}
            else {self.selected_rom_bank.store(value, Ordering::Relaxed)}
        }
        else if address >= 0x4000 && address <= 0x5FFF {
            // TODO: Can be either RAM bank, or RTC register selection
            self.selected_ram_bank.store(value, Ordering::Relaxed);
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.ram_enabled.load(Ordering::Relaxed) && self.has_ram {
                let bank_offset = 8192 * self.selected_ram_bank.load(Ordering::Relaxed) as usize;
                let address = address as usize - 0xA000 + bank_offset;
                self.ram_data[address].store(value, Ordering::Relaxed);

                if self.has_battery {
                    self.save_cart_ram();
                }
            }
        }
    }

    fn save_cart_ram(&self) {
        let path = format!("saved_ram/{}.rr", self.rom_title);
        let mut ram: Vec<u8> = Vec::new();
        let mut index: usize = 0;

        for item in self.ram_data.iter() {
            ram.insert(index, item.load(Ordering::Relaxed));
            index += 1;
        }
        match fs::create_dir("saved_ram") {
            Ok(_) => {},
            Err(error) => match error.kind() {
                io::ErrorKind::AlreadyExists => {},
                _ => error!("Failed to create directory for cart RAM, error: {}", error),
            }
        };
        let mut file = File::create(path).unwrap();
        file.write_all(&ram).unwrap();
    }
}