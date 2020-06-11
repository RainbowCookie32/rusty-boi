use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use log::warn;

use super::cart;
use super::cart::GameboyCart;

pub struct EmulatedMemory {
    bootrom: Vec<u8>,
    cartridge: Box<dyn GameboyCart + Send>,

    character_ram: Vec<u8>,
    background_data: Vec<u8>,

    ram: Vec<u8>,
    oam: Vec<u8>,
    io_registers: Vec<u8>,
    hram: Vec<u8>,

    interrupts: u8,
    bootrom_enabled: bool,

    t0_hash: u64,
    t1_hash: u64,
    oam_hash: u64
}

impl EmulatedMemory {
    pub fn new(bootrom: Option<Vec<u8>>) -> EmulatedMemory {
        let bootrom_enabled = bootrom.is_some();

        EmulatedMemory {
            bootrom: if bootrom.is_some() { bootrom.unwrap() } else { Vec::new() },
            cartridge: cart::dummy_cart(),

            character_ram: vec![0xFF; 6144],
            background_data: vec![0xFF; 2048],

            ram: vec![0xFF; 8192],
            oam: vec![0xFF; 160],
            io_registers: vec![0xFF; 128],
            hram: vec![0xFF; 128],

            interrupts: 0,
            bootrom_enabled: bootrom_enabled,

            t0_hash: 0,
            t1_hash: 0,
            oam_hash: 0
        }
    }

    pub fn get_bootrom_state(&self) -> bool {
        self.bootrom_enabled
    }

    pub fn disable_bootrom(&mut self) {
        self.bootrom_enabled = false;
    }

    pub fn set_cart_data(&mut self, data: Vec<u8>) {
        self.cartridge = cart::new_cart(data);
    }

    pub fn get_t0_hash(&self) -> u64 {
        self.t0_hash
    }

    pub fn get_t1_hash(&self) -> u64 {
        self.t1_hash
    }

    pub fn get_oam_hash(&self) -> u64 {
        self.oam_hash
    }

    fn hash_signed_tiles(&mut self) {
        let mut index: usize = 2047;
        let mut hashable_vec: Vec<u8> = Vec::with_capacity(3072);

        while index < 6144 {
            hashable_vec.push(self.character_ram[index]);
            index += 1;
        }

        let mut hasher = DefaultHasher::new();
        hashable_vec.hash(&mut hasher);
        self.t1_hash = hasher.finish();
    }

    fn hash_unsigned_tiles(&mut self) {
        let mut index: usize = 0;
        let mut hashable_vec: Vec<u8> = Vec::with_capacity(3072);

        while index < 4096 {
            hashable_vec.push(self.character_ram[index]);
            index += 1;
        }

        let mut hasher = DefaultHasher::new();
        hashable_vec.hash(&mut hasher);
        self.t0_hash = hasher.finish();
    }

    fn hash_oam(&mut self) {
        let mut hasher = DefaultHasher::new();
        self.oam.hash(&mut hasher);
        self.oam_hash = hasher.finish();
    }

    pub fn read(&self, address: u16) -> u8 {
        if address < 0x0100 {
            if self.bootrom_enabled {
                self.bootrom[address as usize]
            }
            else {
                self.cartridge.read(address)
            }
        }
        else if address <= 0x7FFF {
            self.cartridge.read(address)
        }
        else if address >= 0x8000 && address <= 0x97FF {
            self.character_ram[(address - 0x8000) as usize]
        }
        else if address >= 0x9800 && address <= 0x9FFF {
            self.background_data[(address - 0x9800) as usize]
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            self.cartridge.read(address)
        }
        else if address >= 0xC000 && address <= 0xDFFF {
            self.ram[(address - 0xC000) as usize]
        }
        else if address >= 0xE000 && address <= 0xFDFF {
            self.ram[(address - 0xE000) as usize]
        }
        else if address >= 0xFE00 && address <= 0xFE9F {
            self.oam[(address - 0xFE00) as usize]
        }
        else if address >= 0xFEA0 && address <= 0xFEFF {
            warn!("Memory: Read to unusable memory at address {:X}", address);
            0xFF
        }
        else if address >= 0xFF00 && address <= 0xFF7F {
            self.io_registers[(address - 0xFF00) as usize]
        }
        else if address >= 0xFF80 && address <= 0xFFFE {
            self.hram[(address - 0xFF80) as usize]
        }
        else if address == 0xFFFF {
            self.interrupts
        }
        else {
            unreachable!()
        }
    }

    pub fn write(&mut self, address: u16, value: u8, cpu: bool) {
        if address <= 0x7FFF {
            self.cartridge.write(address, value);
        }
        else if address >= 0x8000 && address <= 0x97FF {
            self.character_ram[(address - 0x8000) as usize] = value;
            if address >= 0x8000 && address <= 0x9000 {
                self.hash_unsigned_tiles();
            }
            else if address >= 0x87FF && address <= 0x97FF {
                self.hash_signed_tiles();
            }
        }
        else if address >= 0x9800 && address <= 0x9FFF {
            self.background_data[(address - 0x9800) as usize] = value;
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            self.cartridge.write(address, value);
        }
        else if address >= 0xC000 && address <= 0xDFFF {
            self.ram[(address - 0xC000) as usize] = value;
        }
        else if address >= 0xE000 && address <= 0xFDFF {
            self.ram[(address - 0xE000) as usize] = value;
        }
        else if address >= 0xFE00 && address <= 0xFE9F {
            self.oam[(address - 0xFE00) as usize] = value;
            self.hash_oam();
        }
        else if address >= 0xFEA0 && address <= 0xFEFF {
            warn!("Memory: Write to unusable memory at address {:X} with value {:X}", address, value);
        }
        else if address >= 0xFF00 && address <= 0xFF7F {
            let mut value = value;

            match address {
                0xFF00 => value |= 0xC0,
                0xFF04 => {
                    if cpu {
                        value = 0;
                    }
                },
                0xFF0F => {
                    value |= 0xE0;
                },
                0xFF41 => {
                    value |= 0x80;
                },
                0xFF44 => {
                    if cpu {
                        value = 0;
                    }
                },
                0xFFFF => {
                    value |= 0xE0;
                }
                _ => {}
            }

            self.io_registers[(address - 0xFF00) as usize] = value;

            if address == 0xFF46 {
                self.dma_transfer(value);
            }
        }
        else if address >= 0xFF80 && address <= 0xFFFE {
            self.hram[(address - 0xFF80) as usize] = value;
        }
        else if address == 0xFFFF {
            self.interrupts = value;
        }
        else {
            unreachable!()
        }
    }

    fn dma_transfer(&mut self, value: u8) {
        let address = (value as u16) << 8;
        let end_address = address + 0x009F;
        let mut transfer_progress = (address, 0xFE00);

        while transfer_progress.0 < end_address {
            let value = self.read(transfer_progress.0);
            self.write(transfer_progress.1, value, false);
            transfer_progress.0 += 1;
            transfer_progress.1 += 1;
        }
    }
}