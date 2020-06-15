use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use log::warn;

use super::cart;
use super::cart::GameboyCart;

pub struct EmulatedMemory {
    bootrom: Vec<AtomicU8>,
    cartridge: Box<dyn GameboyCart + Send + Sync>,

    character_ram: Vec<AtomicU8>,
    background_data: Vec<AtomicU8>,

    ram: Vec<AtomicU8>,
    oam: Vec<AtomicU8>,
    io_registers: Vec<AtomicU8>,
    hram: Vec<AtomicU8>,

    interrupts: AtomicU8,
    bootrom_enabled: AtomicBool,

    t0_hash: AtomicU64,
    t1_hash: AtomicU64,
    oam_hash: AtomicU64
}

impl EmulatedMemory {
    pub fn new(bootrom: Option<Vec<u8>>) -> EmulatedMemory {
        let mut data = Vec::new();

        let use_bootrom = bootrom.is_some();

        if use_bootrom {
            for byte in bootrom.unwrap() {
                data.push(AtomicU8::new(byte));
            }
        }

        EmulatedMemory {
            bootrom: data,
            cartridge: cart::dummy_cart(),

            character_ram: create_atomic_vec(6144),
            background_data: create_atomic_vec(2048),

            ram: create_atomic_vec(8192),
            oam: create_atomic_vec(160),
            io_registers: create_atomic_vec(128),
            hram: create_atomic_vec(128),

            interrupts: AtomicU8::new(0),
            bootrom_enabled: AtomicBool::from(use_bootrom),

            t0_hash: AtomicU64::new(0),
            t1_hash: AtomicU64::new(0),
            oam_hash: AtomicU64::new(0)
        }
    }

    pub fn get_bootrom_state(&self) -> bool {
        self.bootrom_enabled.load(Ordering::Relaxed)
    }

    pub fn disable_bootrom(&self) {
        self.bootrom_enabled.store(false, Ordering::Relaxed);
    }

    pub fn set_cart_data(&mut self, data: Vec<u8>) {
        self.cartridge = cart::new_cart(data);
    }

    pub fn get_t0_hash(&self) -> u64 {
        self.t0_hash.load(Ordering::Relaxed)
    }

    pub fn get_t1_hash(&self) -> u64 {
        self.t1_hash.load(Ordering::Relaxed)
    }

    pub fn get_oam_hash(&self) -> u64 {
        self.oam_hash.load(Ordering::Relaxed)
    }

    fn hash_signed_tiles(&self) {
        let mut index: usize = 2047;
        let mut hashable_vec: Vec<u8> = Vec::with_capacity(3072);

        while index < 6144 {
            hashable_vec.push(self.character_ram[index].load(Ordering::Relaxed));
            index += 1;
        }

        let mut hasher = DefaultHasher::new();
        hashable_vec.hash(&mut hasher);
        self.t1_hash.store(hasher.finish(), Ordering::Relaxed);
    }

    fn hash_unsigned_tiles(&self) {
        let mut index: usize = 0;
        let mut hashable_vec: Vec<u8> = Vec::with_capacity(3072);

        while index < 4096 {
            hashable_vec.push(self.character_ram[index].load(Ordering::Relaxed));
            index += 1;
        }

        let mut hasher = DefaultHasher::new();
        hashable_vec.hash(&mut hasher);
        self.t0_hash.store(hasher.finish(), Ordering::Relaxed);
    }

    fn hash_oam(&self) {
        let mut index: usize = 0;
        let mut hashable_vec: Vec<u8> = Vec::with_capacity(3072);

        while index < self.oam.len() {
            hashable_vec.push(self.character_ram[index].load(Ordering::Relaxed));
            index += 1;
        }

        let mut hasher = DefaultHasher::new();
        hashable_vec.hash(&mut hasher);
        self.oam_hash.store(hasher.finish(), Ordering::Relaxed);
    }

    pub fn read(&self, address: u16) -> u8 {
        if address < 0x0100 {
            if self.bootrom_enabled.load(Ordering::Relaxed) {
                self.bootrom[address as usize].load(Ordering::Relaxed)
            }
            else {
                self.cartridge.read(address)
            }
        }
        else if address <= 0x7FFF {
            self.cartridge.read(address)
        }
        else if address >= 0x8000 && address <= 0x97FF {
            self.character_ram[(address - 0x8000) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0x9800 && address <= 0x9FFF {
            self.background_data[(address - 0x9800) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            self.cartridge.read(address)
        }
        else if address >= 0xC000 && address <= 0xDFFF {
            self.ram[(address - 0xC000) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xE000 && address <= 0xFDFF {
            self.ram[(address - 0xE000) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xFE00 && address <= 0xFE9F {
            self.oam[(address - 0xFE00) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xFEA0 && address <= 0xFEFF {
            warn!("Memory: Read to unusable memory at address {:X}", address);
            0xFF
        }
        else if address >= 0xFF00 && address <= 0xFF7F {
            self.io_registers[(address - 0xFF00) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xFF80 && address <= 0xFFFE {
            self.hram[(address - 0xFF80) as usize].load(Ordering::Relaxed)
        }
        else if address == 0xFFFF {
            self.interrupts.load(Ordering::Relaxed)
        }
        else {
            unreachable!()
        }
    }

    pub fn write(&self, address: u16, value: u8, cpu: bool) {
        if address <= 0x7FFF {
            self.cartridge.write(address, value);
        }
        else if address >= 0x8000 && address <= 0x97FF {
            self.character_ram[(address - 0x8000) as usize].store(value, Ordering::Relaxed);
            if address >= 0x8000 && address <= 0x9000 {
                self.hash_unsigned_tiles();
            }
            else if address >= 0x87FF && address <= 0x97FF {
                self.hash_signed_tiles();
            }
        }
        else if address >= 0x9800 && address <= 0x9FFF {
            self.background_data[(address - 0x9800) as usize].store(value, Ordering::Relaxed);
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            self.cartridge.write(address, value);
        }
        else if address >= 0xC000 && address <= 0xDFFF {
            self.ram[(address - 0xC000) as usize].store(value, Ordering::Relaxed);
        }
        else if address >= 0xE000 && address <= 0xFDFF {
            self.ram[(address - 0xE000) as usize].store(value, Ordering::Relaxed);
        }
        else if address >= 0xFE00 && address <= 0xFE9F {
            self.oam[(address - 0xFE00) as usize].store(value, Ordering::Relaxed);
            self.hash_oam();
        }
        else if address >= 0xFEA0 && address <= 0xFEFF {
            warn!("Memory: Write to unusable memory at address {:X} with value {:X}", address, value);
        }
        else if address >= 0xFF00 && address <= 0xFF7F {
            let mut value = value;

            // Ignore writes to unused registers, they return $FF.
            if UNUSED_REGS.contains(&address) {
                return;
            }

            match address {
                0xFF00 => {
                    value |= 0xC0;
                },
                0xFF02 => {
                    value |= 0x7E;
                },
                0xFF04 => {
                    if cpu {
                        value = 0;
                    }
                },
                0xFF07 => {
                    value |= 0xF8;
                },
                0xFF10 => {
                    value |= 0x80;
                },
                0xFF1A => {
                    value |= 0x7F;
                },
                0xFF1C => {
                    value |= 0x9F;
                },
                0xFF20 => {
                    value |= 0xC0;
                },
                0xFF23 => {
                    value |= 0x3F;
                },
                0xFF26 => {
                    value |= 0x70;
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
                },
                _ => {}
            }

            self.io_registers[(address - 0xFF00) as usize].store(value, Ordering::Relaxed);

            if address == 0xFF46 {
                self.dma_transfer(value);
            }
        }
        else if address >= 0xFF80 && address <= 0xFFFE {
            self.hram[(address - 0xFF80) as usize].store(value, Ordering::Relaxed);
        }
        else if address == 0xFFFF {
            self.interrupts.store(value, Ordering::Relaxed);
        }
        else {
            unreachable!()
        }
    }

    fn dma_transfer(&self, value: u8) {
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

fn create_atomic_vec(size: usize) -> Vec<AtomicU8> {
    let mut result = Vec::with_capacity(size);

    for _foo in 0..size {
        result.push(AtomicU8::new(0xFF));
    }

    result
}

const UNUSED_REGS: [u16; 71] = [
    0xFF03, 0xFF08, 0xFF09, 0xFF0A, 0xFF0B,
    0xFF0C, 0xFF0D, 0xFF0E, 0xFF15, 0xFF1F,
    0xFF27, 0xFF28, 0xFF29, 0xFF2A, 0xFF2B,
    0xFF2C, 0xFF2D, 0xFF2E, 0xFF2F, 0xFF4C,
    0xFF4D, 0xFF4E, 0xFF4F, 0xFF50, 0xFF51,
    0xFF52, 0xFF53, 0xFF54, 0xFF55, 0xFF56,
    0xFF57, 0xFF58, 0xFF59, 0xFF5A, 0xFF5B,
    0xFF5C, 0xFF5D, 0xFF5E, 0xFF5F, 0xFF60,
    0xFF61, 0xFF62, 0xFF63, 0xFF64, 0xFF65,
    0xFF66, 0xFF67, 0xFF68, 0xFF69, 0xFF6A,
    0xFF6B, 0xFF6C, 0xFF6D, 0xFF6E, 0xFF6F,
    0xFF70, 0xFF71, 0xFF72, 0xFF73, 0xFF74,
    0xFF75, 0xFF76, 0xFF77, 0xFF78, 0xFF79,
    0xFF7A, 0xFF7B, 0xFF7C, 0xFF7D, 0xFF7E,
    0xFF7F
];