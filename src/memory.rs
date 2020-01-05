use std::sync::atomic::{AtomicU8, AtomicBool, Ordering};

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

    using_bootrom: AtomicBool,
    interrupts_enabled: AtomicU8,

    pub tiles_dirty_flags: AtomicU8,
    pub sprites_dirty_flags: AtomicU8,
    pub background_dirty_flags: AtomicU8,
}

impl Memory {

    pub fn new(bootrom_data: Vec<u8>, use_bootrom: bool, loaded_cart: CartData) -> Memory {

        Memory {
            bootrom: bootrom_data,
            loaded_cart: loaded_cart,
            char_ram: new_atomic_vec(6144),
            background_memory: new_atomic_vec(2048),
            ram: new_atomic_vec(8192),
            oam_mem: new_atomic_vec(160),
            io_registers: new_atomic_vec(128),
            hram: new_atomic_vec(128),
            using_bootrom: AtomicBool::from(use_bootrom),
            interrupts_enabled: AtomicU8::new(0),
            tiles_dirty_flags: AtomicU8::new(0),
            sprites_dirty_flags: AtomicU8::new(0),
            background_dirty_flags: AtomicU8::new(0),
        }
    }

    pub fn is_bootrom_loaded(&self) -> bool {
        self.using_bootrom.load(Ordering::Relaxed)
    }

    pub fn bootrom_finished(&self) {
        self.using_bootrom.store(false, Ordering::Relaxed);
    }

    pub fn read(&self, address: u16) -> u8 {
        if address < 0x100 {
            if self.using_bootrom.load(Ordering::Relaxed) {
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

    pub fn write(&self, address: u16, value: u8, cpu: bool) {

        if address <= 0x7FFF {
            self.loaded_cart.write(address, value);
        }

        else if address >= 0x8000 && address <= 0x97FF {
            self.char_ram[address as usize - 0x8000].store(value, Ordering::Relaxed);
            self.tiles_dirty_flags.fetch_add(1, Ordering::Relaxed);
            self.background_dirty_flags.fetch_add(1, Ordering::Relaxed);
            self.sprites_dirty_flags.fetch_add(1, Ordering::Relaxed);
        }

        else if address >= 0x9800 && address <= 0x9FFF {
            self.background_memory[address as usize - 0x9800].store(value, Ordering::Relaxed);
            self.background_dirty_flags.fetch_add(1, Ordering::Relaxed);
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
            self.sprites_dirty_flags.fetch_add(1, Ordering::Relaxed);
        }

        else if address >= 0xFEA0 && address <= 0xFEFF {
            warn!("Memory: Write to unusable memory at 0x{:X} with value {:X}", address, value);
        }

        else if address >= 0xFF00 && address <= 0xFF7F {

            if cpu {
                if address == 0xFF04 || address == 0xFF44 {
                    self.io_registers[address as usize - 0xFF00].store(0, Ordering::Relaxed);
                    return;
                }
            }

            self.io_registers[address as usize - 0xFF00].store(value, Ordering::Relaxed);

            if address == 0xFF46 {
                self.dma_transfer(value);
            }
        }

        else if address >= 0xFF80 && address <= 0xFFFE {
            self.hram[address as usize - 0xFF80].store(value, Ordering::Relaxed);
        }

        else if address == 0xFFFF {
            self.interrupts_enabled.store(value, Ordering::Relaxed);
        }
    }

    fn dma_transfer(&self, value: u8) {
        let address = (value as u16) << 8;
        let end_address = address + 0x009F;

        let mut transfer_progress = (address, 0xFE00);

        while transfer_progress.0 < end_address {
            let value = self.read(transfer_progress.0);
            self.write(transfer_progress.1, value, true);
            transfer_progress.0 += 1;
            transfer_progress.1 += 1;
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
