use log::{error, warn, info};

use std::io;
use std::fs;
use std::path;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::iter::FromIterator;

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
    
    rom_banks: Vec<Vec<u8>>,
    ram_banks: Vec<Vec<u8>>,

    rom_title: String,
    
    has_ram: bool,
    has_battery: bool,
    ram_enabled: bool,

    selected_rom_bank: u8,
    selected_ram_bank: u8,

    rom_banking_mode: bool,

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
        let mut ram_banks: Vec<Vec<u8>> = vec![vec![0; 8192]; ram_size];

        if ram_path.exists() && ram_size > 0 {

            info!("Cart: RAM file found at {:#?}, loading.", ram_path);
            let mut ram_contents: Vec<u8> = Vec::new();
            let mut ram_file = File::open(ram_path).unwrap();
            let mut loaded_banks = 0;

            ram_file.read_to_end(&mut ram_contents).unwrap();

            while loaded_banks < ram_size {
                let bank = Vec::from_iter(ram_contents[8192 * loaded_banks..(8192 * loaded_banks) + 8192].iter().cloned());
                ram_banks[loaded_banks] = bank;
                loaded_banks += 1;
            }
        }

        let mut loaded_banks = 0;
        let mut rom_banks: Vec<Vec<u8>> = vec![Vec::new(); rom_size];

        while loaded_banks < rom_size {

            let bank = Vec::from_iter(data[16384 * loaded_banks..(16384 * loaded_banks) + 16384].iter().cloned());
            rom_banks[loaded_banks] = bank;
            loaded_banks += 1;
        }

        info!("Loader: Cart loaded successfully.");
        println!("\nROM Title: {} \nMBC Type: {:#?} \nROM Size: {} kb \nRAM Size: {}kb\n", title, cart_type, rom_size, ram_size);

        CartData {
            rom_banks: rom_banks,
            ram_banks: ram_banks,
            rom_title: title.to_lowercase(),
            has_ram: ram_size > 0,
            has_battery: battery,
            ram_enabled: false,
            selected_rom_bank: 1,
            selected_ram_bank: 0,
            rom_banking_mode: true,
            mbc: cart_type,
        }
    }

    pub fn read(&self, address: u16) -> u8 {

        if address <= 0x3FFF {
            self.rom_banks[0][address as usize]
        }
        else if address >= 0x4000 && address <= 0x7FFF {
            let result = self.rom_banks.get(self.selected_rom_bank as usize);
            match result {
                Some(value) => value[(address - 0x4000) as usize],
                None => {
                    warn!("Memory: ROM Bank selection was out of bounds, returning 0");
                    0
                }
            }
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.ram_enabled {
                let result = self.ram_banks.get(self.selected_ram_bank as usize);
                match result {
                    Some(value) => value[(address - 0xA000) as usize],
                    None => {
                        warn!("Memory: RAM Bank selection was out of bounds, returning 0");
                        0
                    }
                }
            }
            else {
                0
            }
        }
        else {
            0
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        
        match self.mbc {
            CartType::None => warn!("Memory: Attempting write to cart without a MBC, ignoring."),
            CartType::MBC1 | CartType::MBC1RAM | CartType::MBC1RAMBattery => self.mbc1_write(address, value),
            CartType::MBC2 | CartType::MBC2Battery => self.mbc2_write(address, value),
            CartType::MBC3 | CartType::MBC3RAM | CartType::MBC3RAMBattery => self.mbc3_write(address, value),
            // TODO: At least MBC5 is missing.
            CartType::Other => warn!("Memory: Attempting write to unsupported cart type, ignoring.")
        }
    }

    fn mbc1_write(&mut self, address: u16, value: u8) {

        if address <= 0x1FFF {
            self.ram_enabled = (value & 0x0A) == 0x0A;
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            if value == 0x0 {self.selected_rom_bank = 0x01}
            else if value == 0x20 {self.selected_rom_bank = 0x21}
            else if value == 0x40 {self.selected_rom_bank = 0x41}
            else if value == 0x60 {self.selected_rom_bank = 0x61}
            else {self.selected_rom_bank = value}
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            
            if self.ram_enabled && self.has_ram {
                let result = self.ram_banks.get_mut(self.selected_ram_bank as usize);
                match result {
                    Some(bank) => {
                        bank[(address - 0xA000) as usize] = value;
                        if self.has_battery{self.save_cart_ram()}
                    }
                    None => warn!("Memory: Selected RAM Bank is out of bounds, ignoring write"),
                }
            }
        }
        else if address >= 0x4000 && address <= 0x5FFF {

            if self.rom_banking_mode {
                self.selected_rom_bank = value;
            }
            else {
                self.selected_ram_bank = value;
            }
        }
        else if address >= 0x6000 && address <= 0x7FFF {

            self.rom_banking_mode = value == 0x1;
        }
    }
    
    fn mbc2_write(&mut self, address: u16, value: u8) {

        if address < 0x1FFF {
            self.ram_enabled = value == 0x1;
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            if value == 0x0 {self.selected_rom_bank = 0x01}
            else if value == 0x20 {self.selected_rom_bank = 0x21}
            else if value == 0x40 {self.selected_rom_bank = 0x41}
            else if value == 0x60 {self.selected_rom_bank = 0x61}
            else {self.selected_rom_bank = value}
        }
        else if address >= 0xA000 && address <= 0xA1FF {
            // TODO: Implement MBC2 RAM.
            warn!("Memory: MBC2 RAM is unimplemented, ignoring write.");
        }
    }

    fn mbc3_write(&mut self, address: u16, value: u8) {

        if address < 0x1FFF {
            // TODO: Also enables R/W to RTC registers.
            self.ram_enabled = (value & 0x0A) == 0x0A;
        }
        else if address >= 0x2000 && address <= 0x3FFF {
            if value == 0x0 {self.selected_rom_bank = 0x1}
            else {self.selected_rom_bank = value}
        }
        else if address >= 0x4000 && address <= 0x5FFF {
            // TODO: Can be either RAM bank, or RTC register selection
            self.selected_ram_bank = value;
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.ram_enabled && self.has_ram {
                let result = self.ram_banks.get_mut(self.selected_ram_bank as usize);
                match result {
                    Some(bank) => {
                        bank[(address - 0xA000) as usize] = value;
                        if self.has_battery{self.save_cart_ram()}
                    }
                    None => warn!("Memory: Selected RAM Bank is out of bounds, ignoring write"),
                }
            }
        }
    }

    fn save_cart_ram(&mut self) {
        let path = format!("saved_ram/{}.rr", self.rom_title);
        let mut ram: Vec<u8> = Vec::new();

        for bank in self.ram_banks.iter_mut() {
            let mut cloned_bank = bank.clone();
            ram.append(&mut cloned_bank);
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