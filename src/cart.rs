use log::warn;

use std::iter::FromIterator;


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
    
    has_ram: bool,
    ram_enabled: bool,

    selected_rom_bank: u8,
    selected_ram_bank: u8,

    rom_banking_mode: bool,

    mbc: CartType,
}

impl CartData {

    pub fn new(data: Vec<u8>) -> CartData {

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

        let ram_banks: Vec<Vec<u8>> = vec![vec![0; 8192]; ram_size];

        let mut loaded_banks: usize = 0;
        let mut rom_banks: Vec<Vec<u8>> = vec![Vec::new(); rom_size];

        while loaded_banks < rom_size {

            let bank = Vec::from_iter(data[16384 * loaded_banks..(16384 * loaded_banks) + 16384].iter().cloned());
            rom_banks[loaded_banks] = bank;
            loaded_banks += 1;
        }

        CartData {
            rom_banks: rom_banks,
            ram_banks: ram_banks,
            has_ram: ram_size > 0,
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
            self.rom_banks[self.selected_rom_bank as usize][(address - 0x4000) as usize]
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            if self.ram_enabled {
                self.ram_banks[self.selected_ram_bank as usize][(address - 0xA000) as usize]
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
                self.ram_banks[self.selected_ram_bank as usize][(address - 0xA000) as usize] = value;
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
                self.ram_banks[self.selected_ram_bank as usize][(address - 0xA000) as usize] = value;
            }
        }
    }
}