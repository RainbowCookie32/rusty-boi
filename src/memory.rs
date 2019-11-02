use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use log::{info, warn};

use super::cart::CartData;


pub struct CpuMemory {

    pub bootrom: Vec<u8>,
    pub cartridge: CartData,
    pub ram: Vec<u8>,
    pub echo_ram: Vec<u8>,
    pub hram: Vec<u8>,
    
    pub bootrom_finished: bool,
}

pub struct IoRegisters {

    pub io_regs: Vec<AtomicU8>,
    pub interrupts: AtomicU8,
}

impl IoRegisters {
    pub fn new() -> IoRegisters {

        let mut regs: Vec<AtomicU8> = Vec::new();

        for _idx in 0..128 {
            regs.push(AtomicU8::new(0));
        }

        IoRegisters {
            io_regs: regs,
            interrupts: AtomicU8::new(0),
        }
    }
}

pub struct GpuMemory {

    pub char_ram: Vec<AtomicU8>,
    pub bg_map: Vec<AtomicU8>,
    pub oam_mem: Vec<AtomicU8>,

    pub tile_palette_dirty: AtomicBool,
    pub sprite_palettes_dirty: AtomicBool,

    pub tiles_dirty_flags: AtomicU8,
    pub sprites_dirty_flags: AtomicU8,
    pub background_dirty_flags: AtomicU8,
}

impl GpuMemory {

    pub fn new() -> GpuMemory {

        let mut char_ram: Vec<AtomicU8> = Vec::new();
        let mut bg_map: Vec<AtomicU8> = Vec::new();
        let mut oam_mem: Vec<AtomicU8> = Vec::new();

        let tile_palette_dirty = AtomicBool::new(false);
        let sprite_palettes_dirty = AtomicBool::new(false);

        let tiles_dirty_flags = AtomicU8::new(1);
        let sprites_dirty_flags = AtomicU8::new(1);
        let background_dirty_flags = AtomicU8::new(1);

        for _idx in 0..6144 {
            char_ram.push(AtomicU8::new(1));
        }

        for _idx in 0..2048 {
            bg_map.push(AtomicU8::new(1));
        }

        for _idx in 0..160 {
            oam_mem.push(AtomicU8::new(1));
        }

        GpuMemory {
            char_ram: char_ram,
            bg_map: bg_map,
            oam_mem: oam_mem,
            tile_palette_dirty: tile_palette_dirty,
            sprite_palettes_dirty: sprite_palettes_dirty,
            tiles_dirty_flags: tiles_dirty_flags,
            sprites_dirty_flags: sprites_dirty_flags,
            background_dirty_flags: background_dirty_flags,
        }
    }
}


pub fn init_memory(data: ((Vec<u8>, bool), CartData)) -> (CpuMemory, Arc<IoRegisters>, Arc<GpuMemory>) {
    
    let bootrom_info = data.0;

    let cpu_memory = CpuMemory {
        bootrom: bootrom_info.0,
        cartridge: data.1,
        ram: vec![0; 8192],
        echo_ram: vec![0; 8192],
        hram: vec![0; 127],

        bootrom_finished: !bootrom_info.1,
    };

    let io_regs = IoRegisters::new();

    let gpu_memory = GpuMemory::new();

    (cpu_memory, Arc::new(io_regs), Arc::new(gpu_memory))
}

pub fn cpu_read(address: u16, cpu_mem: &mut CpuMemory, shared_mem: &(Arc<IoRegisters>, Arc<GpuMemory>)) -> u8 {

    if address < 0x0100 
    {
        if cpu_mem.bootrom_finished {
            cpu_mem.cartridge.read(address)
        }
        else {
            cpu_mem.bootrom[address as usize]
        }
    }
    else if address <= 0x7FFF
    {
        cpu_mem.cartridge.read(address)
    }
    else if address >= 0x8000 && address <= 0x97FF
    {
        shared_mem.1.char_ram[(address - 0x8000) as usize].load(Ordering::Relaxed)
    }
    else if address >= 0x9800 && address <= 0x9FFF
    {
        shared_mem.1.bg_map[(address - 0x9800) as usize].load(Ordering::Relaxed)
    }
    else if address >= 0xA000 && address <= 0xBFFF 
    {
        cpu_mem.cartridge.read(address)
    }
    else if address >= 0xC000 && address <= 0xDFFF
    {
        cpu_mem.ram[(address - 0xC000) as usize]
    }
    else if address >= 0xE000 && address <= 0xFDFF 
    {
        cpu_mem.echo_ram[(address - 0xE000) as usize]
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        shared_mem.1.oam_mem[(address - 0xFE00) as usize].load(Ordering::Relaxed)
    }
    else if address >= 0xFEA0 && address <= 0xFEFF
    {
        warn!("Memory: Read to unusable memory at address {}. Returning 0", format!("{:#X}", address));
        0
    }
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        shared_mem.0.io_regs[(address - 0xFF00) as usize].load(Ordering::Relaxed)
    }
    else if address >= 0xFF80 && address <= 0xFFFE
    {
        cpu_mem.hram[(address - 0xFF80) as usize]
    }
    else if address == 0xFFFF
    {
        shared_mem.0.interrupts.load(Ordering::Relaxed)
    }
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", address));
    }
}

pub fn cpu_write(address: u16, value: u8, cpu_mem: &mut CpuMemory, shared_mem: &(Arc<IoRegisters>, Arc<GpuMemory>)) {

    if address <= 0x7FFF
    {
        cpu_mem.cartridge.write(address, value);
    }
    else if address >= 0x8000 && address <= 0x97FF
    {
        if shared_mem.1.char_ram[(address - 0x8000) as usize].load(Ordering::Relaxed) != value {
            shared_mem.1.tiles_dirty_flags.fetch_add(1, Ordering::Relaxed);
            shared_mem.1.sprites_dirty_flags.fetch_add(1, Ordering::Relaxed);
            shared_mem.1.background_dirty_flags.fetch_add(1, Ordering::Relaxed);
        }
        shared_mem.1.char_ram[(address - 0x8000) as usize].store(value, Ordering::Relaxed);
    }
    else if address >= 0x9800 && address <= 0x9FFF
    {
        if shared_mem.1.bg_map[(address - 0x9800) as usize].load(Ordering::Relaxed) != value {
            shared_mem.1.background_dirty_flags.fetch_add(1, Ordering::Relaxed);
        }
        shared_mem.1.bg_map[(address - 0x9800) as usize].store(value, Ordering::Relaxed);
    }
    else if address >= 0xA000 && address <= 0xBFFF 
    {
        cpu_mem.cartridge.write(address, value);
    }
    else if address >= 0xC000 && address <= 0xDFFF
    {
        cpu_mem.ram[(address - 0xC000) as usize] = value;
        cpu_mem.echo_ram[(address - 0xC000) as usize] = value;
    }
    else if address >= 0xE000 && address <= 0xFDFF 
    {
        if value != 0 {warn!("Memory: Write to echo ram. Address {}, value {}.", format!("{:#X}", address), format!("{:#X}", value))}
        cpu_mem.ram[(address - 0xE000) as usize] = value;
        cpu_mem.echo_ram[(address - 0xE000) as usize] = value;
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        if shared_mem.1.oam_mem[(address - 0xFE00) as usize].load(Ordering::Relaxed) != value {
            shared_mem.1.sprites_dirty_flags.fetch_add(1, Ordering::Relaxed);
        }
        shared_mem.1.oam_mem[(address - 0xFE00) as usize].store(value, Ordering::Relaxed);
    }
    else if address >= 0xFEA0 && address <= 0xFEFF
    {
        if value != 0 {warn!("Memory: Write to unusable memory at address {}, value {}. Ignoring...", format!("{:#X}", address), format!("{:#X}", value))}
    }
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        if address == 0xFF46 {
            do_dma_transfer(value, cpu_mem, shared_mem);
        }
        else {
            if address == 0xFF04 || address == 0xFF44 {
                shared_mem.0.io_regs[(address - 0xFF00) as usize].store(0, Ordering::Relaxed);
            }
            else {
                if address == 0xFF47 {
                    shared_mem.1.tile_palette_dirty.store(true, Ordering::Relaxed);
                }
                if address == 0xFF48 || address == 0xFF49 {
                    shared_mem.1.sprite_palettes_dirty.store(true, Ordering::Relaxed);
                }
                shared_mem.0.io_regs[(address - 0xFF00) as usize].store(value, Ordering::Relaxed);
            }
        }
        
    }
    else if address >= 0xFF80 && address <= 0xFFFE 
    {
        cpu_mem.hram[(address - 0xFF80) as usize] = value;
    }
    else if address == 0xFFFF
    {
        shared_mem.0.interrupts.store(value,Ordering::Relaxed);
    }
    else
    {
        panic!("Invalid or unimplemented write at {}", format!("{:#X}", address));
    }
}

pub fn timer_read(address: u16, memory: &Arc<IoRegisters>) -> u8 {

    if address >= 0xFF00 && address <= 0xFF7F
    {
        memory.io_regs[(address - 0xFF00) as usize].load(Ordering::Relaxed)
    }
    else {
        info!("Memory: Timer tried to read at address {}, returning 0", format!("{:#X}", address));
        0
    }
}

pub fn timer_write(address: u16, value: u8, memory: &Arc<IoRegisters>) {

    if address >= 0xFF00 && address <= 0xFF7F
    {
        memory.io_regs[(address - 0xFF00) as usize].store(value, Ordering::Relaxed);
    }
    else {
        info!("Memory: Timer tried to write value {} at address {}", format!("{:#X}", value), format!("{:#X}", address));
    }
}

pub fn gpu_read(address: u16, memory: &(Arc<IoRegisters>, Arc<GpuMemory>)) -> u8 {

    if address >= 0x8000 && address <= 0x97FF
    {
        memory.1.char_ram[(address - 0x8000) as usize].load(Ordering::Relaxed)
    }
    else if address >= 0x9800 && address <= 0x9FFF
    {
        memory.1.bg_map[(address - 0x9800) as usize].load(Ordering::Relaxed)
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        memory.1.oam_mem[(address - 0xFE00) as usize].load(Ordering::Relaxed)
    }
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        memory.0.io_regs[(address - 0xFF00) as usize].load(Ordering::Relaxed)
    }
    else 
    {
        info!("Memory: GPU tried to read at {}, returning 0", format!("{:#X}", address));
        0
    }
}

pub fn gpu_write(address: u16, value: u8, memory: &Arc<IoRegisters>) {

    if address >= 0xFF00 && address <= 0xFF7F
    {
        memory.io_regs[(address - 0xFF00) as usize].store(value, Ordering::Relaxed);
    }
    else {
        info!("Memory: GPU tried to write value {} at address {}", format!("{:#X}", value), format!("{:#X}", address));
    }
}

fn do_dma_transfer(value: u8, cpu_mem: &mut CpuMemory, shared_mem: &(Arc<IoRegisters>, Arc<GpuMemory>)) {

    let start_addr: u16 = (value as u16) << 8;
    let end_addr: u16 = start_addr + 0x009F;

    let mut current_addr = (start_addr, 0xFE00);

    while current_addr.0 < end_addr {
        let value = cpu_read(current_addr.0, cpu_mem, shared_mem);
        cpu_write(current_addr.1, value, cpu_mem, shared_mem);
        current_addr.0 += 1;
        current_addr.1 += 1;
    }
}