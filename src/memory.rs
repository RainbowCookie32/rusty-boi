use std::sync::{Arc, Mutex};

use log::{info, warn};

use super::cart::CartData;


pub struct RomMemory {

    pub loaded_bootrom: Vec<u8>,
    pub loaded_cart: CartData,
    pub bootrom_finished: bool,
}

pub struct CpuMemory {

    pub ram: Vec<u8>,
    pub echo_ram: Vec<u8>,
    pub io_regs: Vec<u8>,
    pub hram: Vec<u8>,
    pub interrupts: u8,

    pub serial_buffer: Vec<u8>,
}

pub struct GpuMemory {

    pub char_ram: Vec<u8>,
    pub bg_map: Vec<u8>,
    pub oam_mem: Vec<u8>,

    pub tile_palette_dirty: bool,
    pub sprite_palettes_dirty: bool,

    pub tiles_dirty_flags: u8,
    pub sprites_dirty_flags: u8,
    pub background_dirty_flags: u8,
}

pub fn init_memory(data: (Vec<u8>, CartData)) -> (Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>) {

    let rom_memory = RomMemory {

        loaded_bootrom: data.0,
        loaded_cart: data.1,
        bootrom_finished: false,
    };
    
    let cpu_memory = CpuMemory {

        ram: vec![0; 8192],
        echo_ram: vec![0; 8192],
        io_regs: vec![0; 256],
        hram: vec![0; 127],
        interrupts: 0,
        serial_buffer: Vec::new(),
    };

    let gpu_memory = GpuMemory {

        char_ram: vec![0; 6144],
        bg_map: vec![0; 2048],
        oam_mem: vec![0; 160],

        tile_palette_dirty: false,
        sprite_palettes_dirty: false,

        tiles_dirty_flags: 0,
        sprites_dirty_flags: 0,
        background_dirty_flags: 0,
    };

    (Arc::new(Mutex::new(rom_memory)), Arc::new(Mutex::new(cpu_memory)), Arc::new(Mutex::new(gpu_memory)))
}

pub fn cpu_read(address: u16, memory: &(Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) -> u8 {

    if address < 0x0100 
    {
        let mem = memory.0.lock().unwrap();
        if mem.bootrom_finished {
            mem.loaded_cart.read(address)
        }
        else {
            mem.loaded_bootrom[address as usize]
        }
    }
    else if address <= 0x7FFF
    {
        let mem = memory.0.lock().unwrap();
        mem.loaded_cart.read(address)
    }
    else if address >= 0x8000 && address <= 0x97FF
    {
        let mem = memory.2.lock().unwrap();
        mem.char_ram[(address - 0x8000) as usize]
    }
    else if address >= 0x9800 && address <= 0x9FFF
    {
        let mem = memory.2.lock().unwrap();
        mem.bg_map[(address - 0x9800) as usize]
    }
    else if address >= 0xA000 && address <= 0xBFFF 
    {
        let mem = memory.0.lock().unwrap();
        mem.loaded_cart.read(address)
    }
    else if address >= 0xC000 && address <= 0xDFFF
    {
        let mem = memory.1.lock().unwrap();
        mem.ram[(address - 0xC000) as usize]
    }
    else if address >= 0xE000 && address <= 0xFDFF 
    {
        let mem = memory.1.lock().unwrap();
        mem.echo_ram[(address - 0xE000) as usize]
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        let mem = memory.2.lock().unwrap();
        mem.oam_mem[(address - 0xFE00) as usize]
    }
    else if address >= 0xFEA0 && address <= 0xFEFF
    {
        warn!("Memory: Read to unusable memory at address {}. Returning 0", format!("{:#X}", address));
        0
    }
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        let mem = memory.1.lock().unwrap();
        mem.io_regs[(address - 0xFF00) as usize]
    }
    else if address >= 0xFF80 && address <= 0xFFFE
    {
        let mem = memory.1.lock().unwrap();
        mem.hram[(address - 0xFF80) as usize]
    }
    else if address == 0xFFFF
    {
        let mem = memory.1.lock().unwrap();
        mem.interrupts
    }
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", address));
    }
}

pub fn cpu_write(address: u16, value: u8, memory: &(Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    if address <= 0x7FFF
    {
        let mut mem = memory.0.lock().unwrap();
        mem.loaded_cart.write(address, value);
    }
    else if address >= 0x8000 && address <= 0x97FF
    {
        let mut mem = memory.2.lock().unwrap();
        if mem.char_ram[(address - 0x8000) as usize] != value {
            mem.tiles_dirty_flags = mem.tiles_dirty_flags.wrapping_add(1);
            mem.sprites_dirty_flags = mem.sprites_dirty_flags.wrapping_add(1);
            mem.background_dirty_flags = mem.background_dirty_flags.wrapping_add(1);
        }
        mem.char_ram[(address - 0x8000) as usize] = value;
    }
    else if address >= 0x9800 && address <= 0x9FFF
    {
        let mut mem = memory.2.lock().unwrap();
        if mem.bg_map[(address - 0x9800) as usize] != value {
            mem.background_dirty_flags = mem.background_dirty_flags.wrapping_add(1);
        }
        mem.bg_map[(address - 0x9800) as usize] = value;
    }
    else if address >= 0xA000 && address <= 0xBFFF 
    {
        let mut mem = memory.0.lock().unwrap();
        mem.loaded_cart.write(address, value);
    }
    else if address >= 0xC000 && address <= 0xDFFF
    {
        let mut mem = memory.1.lock().unwrap();
        mem.ram[(address - 0xC000) as usize] = value;
        mem.echo_ram[(address - 0xC000) as usize] = value;
    }
    else if address >= 0xE000 && address <= 0xFDFF 
    {
        if value != 0 {warn!("Memory: Write to echo ram. Address {}, value {}.", format!("{:#X}", address), format!("{:#X}", value))}
        let mut mem = memory.1.lock().unwrap();
        mem.ram[(address - 0xE000) as usize] = value;
        mem.echo_ram[(address - 0xE000) as usize] = value;
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        let mut mem = memory.2.lock().unwrap();
        if mem.oam_mem[(address - 0xFE00) as usize] != value {
            mem.sprites_dirty_flags = mem.sprites_dirty_flags.wrapping_add(1);
        }
        mem.oam_mem[(address - 0xFE00) as usize] = value;
    }
    else if address >= 0xFEA0 && address <= 0xFEFF
    {
        if value != 0 {warn!("Memory: Write to unusable memory at address {}, value {}. Ignoring...", format!("{:#X}", address), format!("{:#X}", value))}
    }
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        if address == 0xFF46 {
            do_dma_transfer(value, memory);
        }
        else {

            // Basically here to print the output of tests.
            // Holds the values stored in FF01 until a line break, then prints them.
            if address == 0xFF01 {
                if value == 0xA {

                    let mut idx: usize = 0;
                    let mut new_string = String::from("");
                    let mut mem = memory.1.lock().unwrap();
                    while idx < mem.serial_buffer.len() {
                        new_string.push(mem.serial_buffer[idx] as char);
                        idx += 1;
                    }

                    info!("Serial:  {} ", new_string);
                    mem.serial_buffer = Vec::new();
                }
                else {
                    let mut mem = memory.1.lock().unwrap();
                    mem.serial_buffer.push(value);
                }
            }
            // According to the docs, writing any value to DIV (FF04) ot LY (FF44) from the CPU
            // resets the value back to 0, so check if it's either of those before writing.
            else if address == 0xFF04 || address == 0xFF44 {
                let mut mem = memory.1.lock().unwrap();
                mem.io_regs[(address - 0xFF00) as usize] = 0;
            }
            else {
                if address == 0xFF47 {
                    let mut mem = memory.2.lock().unwrap();
                    mem.tile_palette_dirty = true;
                }
                if address == 0xFF48 || address == 0xFF49 {
                    let mut mem = memory.2.lock().unwrap();
                    mem.sprite_palettes_dirty = true;
                }
                let mut mem = memory.1.lock().unwrap();
                mem.io_regs[(address - 0xFF00) as usize] = value;
            }
        }
        
    }
    else if address >= 0xFF80 && address <= 0xFFFE 
    {
        let mut mem = memory.1.lock().unwrap();
        mem.hram[(address - 0xFF80) as usize] = value;
    }
    else if address == 0xFFFF
    {
        let mut mem = memory.1.lock().unwrap();
        mem.interrupts = value;
    }
    else
    {
        panic!("Invalid or unimplemented write at {}", format!("{:#X}", address));
    }
}

pub fn timer_read(address: u16, memory: &Arc<Mutex<CpuMemory>>) -> u8 {

    if address >= 0xFF00 && address <= 0xFF7F
    {
        let mem = memory.lock().unwrap();
        mem.io_regs[(address - 0xFF00) as usize]
    }
    else {
        info!("Memory: Timer tried to read at address {}, returning 0", format!("{:#X}", address));
        0
    }
}

pub fn timer_write(address: u16, value: u8, memory: &Arc<Mutex<CpuMemory>>) {

    if address >= 0xFF00 && address <= 0xFF7F
    {
        let mut mem = memory.lock().unwrap();
        mem.io_regs[(address - 0xFF00) as usize] = value;
    }
    else {
        info!("Memory: Timer tried to write value {} at address {}", format!("{:#X}", value), format!("{:#X}", address));
    }
}

pub fn gpu_read(address: u16, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) -> u8 {

    if address >= 0x8000 && address <= 0x97FF
    {
        let mem = memory.1.lock().unwrap();
        mem.char_ram[(address - 0x8000) as usize]
    }
    else if address >= 0x9800 && address <= 0x9FFF
    {
        let mem = memory.1.lock().unwrap();
        mem.bg_map[(address - 0x9800) as usize]
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        let mem = memory.1.lock().unwrap();
        mem.oam_mem[(address - 0xFE00) as usize]
    }
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        let mem = memory.0.lock().unwrap();
        mem.io_regs[(address - 0xFF00) as usize]
    }
    else 
    {
        info!("Memory: GPU tried to read at {}, returning 0", format!("{:#X}", address));
        0
    }
}

pub fn gpu_write(address: u16, value: u8, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    if address >= 0xFF00 && address <= 0xFF7F
    {
        let mut mem = memory.0.lock().unwrap();
        mem.io_regs[(address - 0xFF00) as usize] = value;
    }
    else {
        info!("Memory: GPU tried to write value {} at address {}", format!("{:#X}", value), format!("{:#X}", address));
    }
}

fn do_dma_transfer(value: u8, memory: &(Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let start_addr: u16 = (value as u16) << 8;
    let end_addr: u16 = start_addr + 0x009F;

    let mut current_addr = (start_addr, 0xFE00);

    while current_addr.0 < end_addr {
        let value = cpu_read(current_addr.0, memory);
        cpu_write(current_addr.1, value, memory);
        current_addr.0 += 1;
        current_addr.1 += 1;
    }
}