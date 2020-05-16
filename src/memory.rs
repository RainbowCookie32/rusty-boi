use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

use log::warn;

use super::cart::CartData;

pub struct Memory {
    bootrom: Vec<u8>,
    cartridge: CartData,

    ram: Vec<u8>,
    hram: Vec<u8>,

    serial_data: String,
    bootrom_enabled: bool,

    shared_memory: Arc<SharedMemory>,
}

impl Memory {
    pub fn new(bootrom: Option<Vec<u8>>, cart: CartData, shared: Arc<SharedMemory>) -> Memory {
        let use_brom = bootrom.is_some();
        let brom_data = if use_brom {bootrom.unwrap()} else {Vec::new()};

        Memory {
            bootrom: brom_data,
            cartridge: cart,

            ram: vec![0; 8192],
            hram: vec![0; 128],

            serial_data: String::new(),
            bootrom_enabled: use_brom,

            shared_memory: shared,
        }
    }

    pub fn get_bootrom_state(&self) -> bool {
        self.bootrom_enabled
    }

    pub fn disable_bootrom(&mut self) {
        self.bootrom_enabled = false;
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
        else if address >= 0xA000 && address <= 0xBFFF {
            self.cartridge.read(address)
        }
        else if address >= 0xC000 && address <= 0xDFFF {
            self.ram[address as usize - 0xC000]
        }
        else if address >= 0xE000 && address <= 0xFDFF {
            self.ram[address as usize - 0xE000]
        }
        else if address >= 0xFF80 && address <= 0xFFFE {
            self.hram[address as usize - 0xFF80]
        }
        else {
            self.shared_memory.read(address)
        }
    }

    pub fn write(&mut self, address: u16, value: u8, cpu: bool) {
        if address <= 0x7FFF {
            if !self.bootrom_enabled {
                self.cartridge.write(address, value);
            }
        }
        else if address >= 0xA000 && address <= 0xBFFF {
            self.cartridge.write(address, value);
        }
        else if address >= 0xC000 && address <= 0xDFFF {
            self.ram[address as usize - 0xC000] = value;
        }
        else if address >= 0xE000 && address <= 0xFDFF {
            self.ram[address as usize - 0xE000] = value;
        }
        else if address >= 0xFF80 && address <= 0xFFFE {
            self.hram[address as usize - 0xFF80] = value;
        }
        else if address == 0xFF46 {
            self.dma_transfer(value);
        }
        else {
            if address == 0xFF01 {
                if value == 10 {
                    log::info!("Serial: {}", self.serial_data);
                    self.serial_data = String::new();
                }
                else {
                    self.serial_data.push(value as char);
                }
            }

            self.shared_memory.write(address, value, cpu);
        }
    }

    fn dma_transfer(&mut self, value: u8) {
        let address = (value as u16) << 8;
        let end_address = address + 0x009F;

        let mut transfer_progress = (address, 0xFE00);

        self.shared_memory.write(0xFF46, value, true);

        while transfer_progress.0 < end_address {
            let value = self.read(transfer_progress.0);
            self.write(transfer_progress.1, value, false);
            transfer_progress.0 += 1;
            transfer_progress.1 += 1;
        }
    }
}

pub struct SharedMemory {
    character_ram: Vec<AtomicU8>,
    background_memory: Vec<AtomicU8>,
    oam_memory: Vec<AtomicU8>,

    io_registers: Vec<AtomicU8>,
    interrupts_enabled: AtomicU8,

    tile0_hash: AtomicU64,
    tile1_hash: AtomicU64,
    oam_hash: AtomicU64,
}

impl SharedMemory {
    pub fn new() -> SharedMemory {
        SharedMemory {
            character_ram: new_atomic_vec(6144),
            background_memory: new_atomic_vec(2048),
            oam_memory: new_atomic_vec(160),

            io_registers: new_atomic_vec(128),
            interrupts_enabled: AtomicU8::new(0),

            tile0_hash: AtomicU64::new(0),
            tile1_hash: AtomicU64::new(0),
            oam_hash: AtomicU64::new(0),
        }
    }

    pub fn get_t0_hash(&self) -> u64 {
        self.tile0_hash.load(Ordering::Relaxed)
    }

    pub fn get_t1_hash(&self) -> u64 {
        self.tile1_hash.load(Ordering::Relaxed)
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
        self.tile1_hash.store(hasher.finish(), Ordering::Relaxed);
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
        self.tile0_hash.store(hasher.finish(), Ordering::Relaxed);
    }

    fn hash_oam(&self) {
        let mut index: usize = 0;
        let mut hashable_vec: Vec<u8> = Vec::with_capacity(3072);

        while index < 160 {
            hashable_vec.push(self.oam_memory[index].load(Ordering::Relaxed));
            index += 1;
        }

        let mut hasher = DefaultHasher::new();
        hashable_vec.hash(&mut hasher);
        self.oam_hash.store(hasher.finish(), Ordering::Relaxed);
    }

    pub fn read(&self, address: u16) -> u8 {
        if address >= 0x8000 && address <= 0x97FF
        {
            self.character_ram[(address - 0x8000) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0x9800 && address <= 0x9FFF
        {
            self.background_memory[(address - 0x9800) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xFE00 && address <= 0xFE9F 
        {
            self.oam_memory[(address - 0xFE00) as usize].load(Ordering::Relaxed)
        }
        else if address >= 0xFEA0 && address <= 0xFEFF {
            warn!("Memory: Invalid read to unusuable memory on address {:X}", address);
            0
        }
        else if address >= 0xFF00 && address <= 0xFF7F
        {
            self.io_registers[(address - 0xFF00) as usize].load(Ordering::Relaxed)
        }
        else if address == 0xFFFF {
            self.interrupts_enabled.load(Ordering::Relaxed)
        }
        else {
            warn!("Memory: Invalid read on shared memory to address {:X}", address);
            0
        }
    }

    pub fn write(&self, address: u16, value: u8, cpu: bool) {
        if address >= 0x8000 && address <= 0x97FF {
            self.character_ram[(address - 0x8000) as usize].store(value, Ordering::Relaxed);
            if address >= 0x8000 && address <= 0x9000 {
                self.hash_unsigned_tiles();
            }
            else if address >= 0x87FF && address <= 0x97FF {
                self.hash_signed_tiles();
            }
        }
        else if address >= 0x9800 && address <= 0x9FFF {
            self.background_memory[(address - 0x9800) as usize].store(value, Ordering::Relaxed);
        }
        else if address >= 0xFE00 && address <= 0xFE9F {
            self.oam_memory[(address - 0xFE00) as usize].store(value, Ordering::Relaxed);
            self.hash_oam();
        }
        else if address >= 0xFEA0 && address <= 0xFEFF {
            warn!("Memory: Invalid write to unusuable memory on address {:X}", address);
        }
        else if address >= 0xFF00 && address <= 0xFF7F {
            if cpu {
                if address == 0xFF04 || address == 0xFF44 {
                    self.io_registers[(address - 0xFF00) as usize].store(0, Ordering::Relaxed);
                    return;
                }

                if address == 0xFF42 && value == 0xE0 {
                    println!("xd");
                }
            }

            self.io_registers[(address - 0xFF00) as usize].store(value, Ordering::Relaxed);
        } 
        else if address == 0xFFFF {
            self.interrupts_enabled.store(value, Ordering::Relaxed);
        }
        else {
            warn!("Memory: Invalid write on shared memory to address {:X}", address);
        }
    }
}

fn new_atomic_vec(size: usize) -> Vec<AtomicU8> {
    let mut new_vec = Vec::with_capacity(size);

    for _idx in 0..size {
        new_vec.push(AtomicU8::new(0));
    }

    new_vec
}