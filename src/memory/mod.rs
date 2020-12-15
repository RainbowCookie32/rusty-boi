use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::atomic::{AtomicU8, AtomicU64, AtomicBool, Ordering};

use log::warn;

use super::cart::CartData;


pub struct Memory {
    bootrom: Vec<u8>,
    loaded_cart: CartData,

    char_ram: Vec<AtomicU8>,
    background_memory: Vec<AtomicU8>,

    ram: Vec<AtomicU8>,
    oam_mem: Vec<AtomicU8>,
    io_registers: Vec<AtomicU8>,
    
    hram: Vec<AtomicU8>,

    bootrom_enabled: AtomicBool,
    interrupts_enabled: AtomicU8,

    tiles_signed_hash: AtomicU64,
    tiles_unsigned_hash: AtomicU64,
}

impl Memory {

    pub fn new(bootrom: Option<Vec<u8>>, cart: CartData) -> Memory {
        let bootrom_enabled = bootrom.is_some();

        Memory {
            bootrom: bootrom.unwrap_or(Vec::new()),
            loaded_cart: cart,

            char_ram: new_atomic_vec(6144),
            background_memory: new_atomic_vec(2048),

            ram: new_atomic_vec(8192),
            oam_mem: new_atomic_vec(160),
            io_registers: new_atomic_vec(128),

            hram: new_atomic_vec(128),

            bootrom_enabled: AtomicBool::from(bootrom_enabled),
            interrupts_enabled: AtomicU8::new(0),

            tiles_signed_hash: AtomicU64::from(0),
            tiles_unsigned_hash: AtomicU64::from(0),
        }
    }

    pub fn is_bootrom_enabled(&self) -> bool {
        self.bootrom_enabled.load(Ordering::Relaxed)
    }

    pub fn bootrom_finished(&self) {
        self.bootrom_enabled.store(false, Ordering::Relaxed);
    }

    fn hash_signed_tiles(&self) {
        let mut index: usize = 2047;
        let mut hashable_vec: Vec<u8> = Vec::with_capacity(3072);

        while index < 6144 {
            hashable_vec.push(self.char_ram[index].load(Ordering::Relaxed));
            index += 1;
        }

        let mut hasher = DefaultHasher::new();
        hashable_vec.hash(&mut hasher);
        self.tiles_signed_hash.store(hasher.finish(), Ordering::Relaxed);
    }

    fn hash_unsigned_tiles(&self) {
        let mut index: usize = 0;
        let mut hashable_vec: Vec<u8> = Vec::with_capacity(3072);

        while index < 4096 {
            hashable_vec.push(self.char_ram[index].load(Ordering::Relaxed));
            index += 1;
        }

        let mut hasher = DefaultHasher::new();
        hashable_vec.hash(&mut hasher);
        self.tiles_unsigned_hash.store(hasher.finish(), Ordering::Relaxed);
    }

    pub fn get_signed_hash(&self) -> u64 {
        self.tiles_signed_hash.load(Ordering::Relaxed)
    }

    pub fn get_unsigned_hash(&self) -> u64 {
        self.tiles_unsigned_hash.load(Ordering::Relaxed)
    }

    pub fn read(&self, address: u16) -> u8 {
        if address < 0x0100 {
            if self.bootrom_enabled.load(Ordering::Relaxed) {
                self.bootrom[address as usize]
            }
            else {
                self.loaded_cart.read(address)
            }
        }

        else if address <= 0x7FFF {
            self.loaded_cart.read(address)
        }

        else if address >= 0x8000 && address <= 0x97FF {
            self.char_ram[address as usize - 0x8000].load(Ordering::Relaxed)
        }

        else if address >= 0x9800 && address <= 0x9FFF {
            self.background_memory[address as usize - 0x9800].load(Ordering::Relaxed)
        }

        else if address >= 0xA000 && address <= 0xBFFF {
            self.loaded_cart.read(address)
        }

        else if address >= 0xC000 && address <= 0xDFFF {
            self.ram[address as usize - 0xC000].load(Ordering::Relaxed)
        }

        else if address >= 0xE000 && address <= 0xFDFF {
            self.ram[address as usize - 0xE000].load(Ordering::Relaxed)
        }

        else if address >= 0xFE00 && address <= 0xFE9F {
            self.oam_mem[address as usize - 0xFE00].load(Ordering::Relaxed)
        }

        else if address >= 0xFEA0 && address <= 0xFEFF {
            warn!("Memory: Read to unusable memory at 0x{:X}", address);
            0
        }

        else if address >= 0xFF00 && address <= 0xFF7F {
            self.io_registers[address as usize - 0xFF00].load(Ordering::Relaxed)
        }

        else if address >= 0xFF80 && address <= 0xFFFE {
            self.hram[address as usize - 0xFF80].load(Ordering::Relaxed)
        }

        else if address == 0xFFFF {
            self.interrupts_enabled.load(Ordering::Relaxed)
        }

        else {
            unreachable!();
        }
    }

    pub fn video_read(&self, address: u16) -> u8 {
        if address >= 0x8000 && address <= 0x97FF {
            self.char_ram[address as usize - 0x8000].load(Ordering::Relaxed)
        }

        else if address >= 0x9800 && address <= 0x9FFF {
            self.background_memory[address as usize - 0x9800].load(Ordering::Relaxed)
        }

        else if address >= 0xFF00 && address <= 0xFF7F {
            self.io_registers[address as usize - 0xFF00].load(Ordering::Relaxed)
        }

        else {
            unreachable!();
        }
    }

    pub fn write(&self, address: u16, value: u8) {
        if address < 0x0100 && !self.bootrom_enabled.load(Ordering::Relaxed) {
            self.loaded_cart.write(address, value);
        }

        if address <= 0x7FFF {
            self.loaded_cart.write(address, value);
        }

        else if address >= 0x8000 && address <= 0x97FF {
            self.char_ram[address as usize - 0x8000].store(value, Ordering::Relaxed);
            
            if address >= 0x8000 && address <= 0x9000 {
                self.hash_unsigned_tiles();
            }
            else if address >= 0x87FF && address <= 0x97FF {
                self.hash_signed_tiles();
            }
        }

        else if address >= 0x9800 && address <= 0x9FFF {
            self.background_memory[address as usize - 0x9800].store(value, Ordering::Relaxed);
        }

        else if address >= 0xA000 && address <= 0xBFFF {
            self.loaded_cart.write(address, value);
        }

        else if address >= 0xC000 && address <= 0xDFFF {
            self.ram[address as usize - 0xC000].store(value, Ordering::Relaxed);
        }

        else if address >= 0xE000 && address <= 0xFDFF {
            warn!("Memory: Write to Echo RAM at 0x{:X} with value {:X}", address, value);
            self.ram[address as usize - 0xE000].store(value, Ordering::Relaxed);
        }

        else if address >= 0xFE00 && address <= 0xFE9F {
            self.oam_mem[address as usize - 0xFE00].store(value, Ordering::Relaxed);
        }

        else if address >= 0xFEA0 && address <= 0xFEFF {
            warn!("Memory: Write to unusable memory at 0x{:X} with value {:X}", address, value);
        }

        else if address >= 0xFF00 && address <= 0xFF7F {
            if address == 0xFF04 || address == 0xFF44 {
                self.io_registers[address as usize - 0xFF00].store(0, Ordering::Relaxed);
                return;
            }

            self.io_registers[address as usize - 0xFF00].store(value, Ordering::Relaxed);
        }

        else if address >= 0xFF80 && address <= 0xFFFE {
            self.hram[address as usize - 0xFF80].store(value, Ordering::Relaxed);
        }

        else if address == 0xFFFF {
            self.interrupts_enabled.store(value, Ordering::Relaxed);
        }
    }

    pub fn video_write(&self, address: u16, value: u8) {
        if address >= 0xFF00 && address <= 0xFF7F {
            self.io_registers[address as usize - 0xFF00].store(value, Ordering::Relaxed);
        }
        else {
            unreachable!();
        }
    }
}

fn new_atomic_vec(size: usize) -> Vec<AtomicU8> {
    let mut result: Vec<AtomicU8> = Vec::with_capacity(size);

    for _item in 0..size {
        result.push(AtomicU8::new(0));
    }

    result
}

#[derive(Clone)]
pub struct DMATransfer {
    source_location: u16,
    target_location: u16,

    started_at: u32,
    memory: Arc<Memory>,
}

impl DMATransfer {
    pub fn new(source: u16, cycles: u32, memory: Arc<Memory>) -> DMATransfer {
        DMATransfer {
            source_location: source,
            target_location: 0xFE00,

            started_at: cycles,
            memory
        }
    }

    pub fn dma_tick(&mut self, cycles: u32) -> bool {
        let cycles_since_last_move = cycles - self.started_at;
        let bytes_to_transfer = cycles_since_last_move / 4;

        for _i in 0..bytes_to_transfer {
            let value = self.memory.read(self.source_location);

            self.memory.write(self.target_location, value);

            self.source_location += 1;
            self.target_location += 1;

            if self.target_location > 0xFE9F {
                break;
            }
        }
        
        (cycles - self.started_at) >= 160 || self.target_location >= 0xFE9F
    }
}
