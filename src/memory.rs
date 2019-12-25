use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicBool, Ordering};

use log::warn;

use super::cart::CartData;


pub struct CpuMemory {

    bootrom: Vec<u8>,
    cartridge: CartData,
    ram: Vec<u8>,
    hram: Vec<u8>,
    bootrom_finished: bool,

    shared_memory: Arc<SharedMemory>,
}

impl CpuMemory {

    pub fn new(bootrom: Vec<u8>, use_bootrom: bool, cart: CartData, shared_mem: Arc<SharedMemory>) -> CpuMemory {
        CpuMemory {
            bootrom: bootrom,
            cartridge: cart,
            ram: vec![0; 8192],
            hram: vec![0; 127],
            bootrom_finished: !use_bootrom,

            shared_memory: shared_mem,
        }
    }

    pub fn use_bootrom(&self) -> bool {
        return !self.bootrom_finished;
    }

    pub fn read(&self, address: u16) -> u8 {
        
        if address < 0x0100 
        {
            if self.bootrom_finished {
                return self.cartridge.read(address);
            }
            else {
                return self.bootrom[address as usize];
            }
        }

        if address <= 0x7FFF
        {
            return self.cartridge.read(address);
        }

        if address >= 0xA000 && address <= 0xBFFF
        {
            return self.cartridge.read(address);
        }

        if address >= 0xC000 && address <= 0xDFFF
        {
            return self.ram[(address - 0xC000) as usize];
        }

        if address >= 0xE000 && address <= 0xFDFF 
        {
            return self.ram[(address - 0xE000) as usize];
        }

        if address >= 0xFEA0 && address <= 0xFEFF
        {
            warn!("Memory: Read to unusable memory at address {}. Returning 0", format!("{:#X}", address));
            return 0;
        }

        if address >= 0xFF80 && address <= 0xFFFE
        {
            return self.hram[(address - 0xFF80) as usize];
        }

        return self.shared_memory.read(address);
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if address < 0x0100 && self.bootrom_finished
        {
            self.cartridge.write(address, value);
            return;
        }

        if address <= 0x7FFF
        {
            self.cartridge.write(address, value);
            return;
        }

        if address >= 0xA000 && address <= 0xBFFF
        {
            self.cartridge.write(address, value);
            return;
        }

        if address >= 0xC000 && address <= 0xDFFF
        {
            self.ram[address as usize - 0xC000] = value;
            return;
        }

        if address >= 0xE000 && address <= 0xFDFF 
        {
            self.ram[address as usize - 0xE000] = value;
            return;
        }

        if address >= 0xFEA0 && address <= 0xFEFF
        {
            warn!("Memory: Tried to write {:#X} to unusable memory at address {:#X}. Ignoring...", value, address);
            return;
        }

        if address >= 0xFF80 && address <= 0xFFFE
        {
            self.hram[address as usize - 0xFF80] = value;
            return;
        }

        self.shared_memory.write(address, value, true);
    }
}

pub struct SharedMemory {
    
    pub io_registers: Vec<AtomicU8>,
    pub interrupts_enabled: AtomicU8,

    pub oam_memory: Vec<AtomicU8>,
    pub character_ram: Vec<AtomicU8>,
    pub background_map: Vec<AtomicU8>,

    pub tile_palette_dirty: AtomicBool,
    pub sprite_palettes_dirty: AtomicBool,

    pub tiles_dirty_flags: AtomicU8,
    pub sprites_dirty_flags: AtomicU8,
    pub background_dirty_flags: AtomicU8,
}

impl SharedMemory {

    pub fn new() -> SharedMemory {

        let mut regs: Vec<AtomicU8> = Vec::new();

        let mut char_ram: Vec<AtomicU8> = Vec::new();
        let mut bg_map: Vec<AtomicU8> = Vec::new();
        let mut oam_mem: Vec<AtomicU8> = Vec::new();

        for _item in 0..160 {
            oam_mem.push(AtomicU8::new(0));
        }
        
        for _item in 0..128 {
            regs.push(AtomicU8::new(0));
        }

        for _item in 0..6144 {
            char_ram.push(AtomicU8::new(0));
        }

        for _item in 0..2048 {
            bg_map.push(AtomicU8::new(0));
        }

        SharedMemory {
            io_registers: regs,
            interrupts_enabled: AtomicU8::new(0),

            oam_memory: oam_mem,
            character_ram: char_ram,
            background_map: bg_map,

            tile_palette_dirty: AtomicBool::new(false),
            sprite_palettes_dirty: AtomicBool::new(false),

            tiles_dirty_flags: AtomicU8::new(0),
            sprites_dirty_flags: AtomicU8::new(0),
            background_dirty_flags: AtomicU8::new(0),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        
        if address >= 0x8000 && address <= 0x97FF
        {
            return self.character_ram[address as usize - 0x8000].load(Ordering::Relaxed);
        }

        if address >= 0x9800 && address <= 0x9FFF
        {
            return self.background_map[address as usize - 0x9800].load(Ordering::Relaxed);
        }

        if address >= 0xFE00 && address <= 0xFE9F 
        {
            return self.oam_memory[address as usize - 0xFE00].load(Ordering::Relaxed);
        }

        if address >= 0xFF00 && address <= 0xFF7F
        {
            return self.io_registers[address as usize - 0xFF00].load(Ordering::Relaxed);
        }

        if address == 0xFFFF
        {
            return self.interrupts_enabled.load(Ordering::Relaxed);
        }

        panic!("Memory: Invalid read on shared memory at address {:#X}", address);
    }

    pub fn write(&self, address: u16, value: u8, is_cpu: bool) {
        
        if address >= 0x8000 && address <= 0x97FF
        {
            self.character_ram[address as usize - 0x8000].store(value, Ordering::Relaxed);
            self.tiles_dirty_flags.fetch_add(1, Ordering::Relaxed);
            self.background_dirty_flags.fetch_add(1, Ordering::Relaxed);
            self.sprites_dirty_flags.fetch_add(1, Ordering::Relaxed);
            return;
        }

        if address >= 0x9800 && address <= 0x9FFF
        {
            self.background_map[address as usize - 0x9800].store(value, Ordering::Relaxed);
            self.background_dirty_flags.fetch_add(1, Ordering::Relaxed);
            return;
        }

        if address >= 0xFE00 && address <= 0xFE9F 
        {
            self.oam_memory[address as usize - 0xFE00].store(value, Ordering::Relaxed);
            self.sprites_dirty_flags.fetch_add(1, Ordering::Relaxed);
            return;
        }

        if address >= 0xFF00 && address <= 0xFF7F
        {
            
            if address == 0xFF04 && is_cpu {
                self.io_registers[address as usize - 0xFF00].store(0, Ordering::Relaxed);
                return;
            }
            else if address == 0xFF46 {
                self.io_registers[address as usize - 0xFF00].store(0, Ordering::Relaxed);
                warn!("Memory: Tried to start a DMA transfer");
                //self.dma_transfer();
                return;
            }
            else {
                self.io_registers[address as usize - 0xFF00].store(value, Ordering::Relaxed);
                return;
            }
        }

        if address == 0xFFFF
        {
            self.interrupts_enabled.store(value, Ordering::Relaxed);
            return;
        }

        panic!("Memory: Invalid read on shared memory at address {:#X}", address);
    }


    // FIXME: DMA Transfers are done from the cartridge, so SharedMemory can't directly access it.
    // Currently, a DMA transfer causes a panic.
    fn dma_transfer(&self) {
        let address = (self.read(0xFF46) as u16) << 8;
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