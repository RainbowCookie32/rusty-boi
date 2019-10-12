use std::sync::{Arc, Mutex};

use log::{info, warn};

use super::emulator::Cart;


pub struct Memory {

    pub loaded_bootrom: Vec<u8>,
    pub loaded_cart: Cart,
    pub selected_bank: u8,

    pub ram: Vec<u8>,
    pub echo_ram: Vec<u8>,
    pub io_regs: Vec<u8>,
    pub hram: Vec<u8>,
    pub interrupts: u8,

    pub char_ram: Vec<u8>,
    pub bg_map: Vec<u8>,
    pub oam_mem: Vec<u8>,

    pub tiles_dirty: bool,
    pub background_dirty: bool,
    pub bootrom_finished: bool,

    pub serial_buffer: Vec<u8>,
}

pub fn init_memory(data: (Vec<u8>, Cart)) -> Arc<Mutex<Memory>> {

    let initial_memory = Memory {

        loaded_bootrom: data.0,
        loaded_cart: data.1,
        selected_bank: 1,

        ram: vec![0; 8192],
        echo_ram: vec![0; 8192],
        io_regs: vec![0; 256],
        hram: vec![0; 127],
        interrupts: 0,

        char_ram: vec![0; 6144],
        bg_map: vec![0; 2048],
        oam_mem: vec![0; 160],

        tiles_dirty: false,
        background_dirty: false,
        bootrom_finished: false,

        serial_buffer: Vec::new(),
    };

    Arc::new(Mutex::new(initial_memory))
}

pub fn read(address: u16, memory: &Memory) -> u8 {

    if address < 0x0100 
    {
        if memory.bootrom_finished {
            memory.loaded_cart.rom_banks[0][address as usize]
        }
        else {
            memory.loaded_bootrom[address as usize]
        }
    }
    else if address >= 0x0100 && address <= 0x3FFF
    {
        memory.loaded_cart.rom_banks[0][address as usize]
    }
    else if address >= 0x4000 && address <= 0x7FFF
    {
        memory.loaded_cart.rom_banks[memory.selected_bank as usize][(address - 0x4000) as usize]
    }
    else if address >= 0x8000 && address <= 0x97FF
    {
        memory.char_ram[(address - 0x8000) as usize]
    }
    else if address >= 0x9800 && address <= 0x9FFF
    {
        memory.bg_map[(address - 0x9800) as usize]
    }
    else if address >= 0xA000 && address <= 0xBFFF 
    {
        if memory.loaded_cart.has_ram {
            memory.loaded_cart.cart_ram[(address - 0xA000) as usize]
        }
        else {
            info!("Memory: Cart has no external RAM, returning 0.");
            0
        }
    }
    else if address >= 0xC000 && address <= 0xDFFF
    {
        memory.ram[(address - 0xC000) as usize]
    }
    else if address >= 0xE000 && address <= 0xFDFF 
    {
        memory.echo_ram[(address - 0xE000) as usize]
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        memory.oam_mem[(address - 0xFE00) as usize]
    }
    else if address >= 0xFEA0 && address <= 0xFEFF
    {
        warn!("Memory: Read to unusable memory at address {}. Returning 0", format!("{:#X}", address));
        0
    }
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        memory.io_regs[(address - 0xFF00) as usize]
    }
    else if address >= 0xFF80 && address <= 0xFFFE
    {
        memory.hram[(address - 0xFF80) as usize]
    }
    else if address == 0xFFFF
    {
        memory.interrupts
    }
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", address));
    }
}

pub fn write(address: u16, value: u8, memory: &mut Memory) {

    if address <= 0x7FFF
    {
        info!("Memory: Switching ROM Bank to {}", value);
        memory.selected_bank = value;
    }
    else if address >= 0x8000 && address <= 0x97FF
    {
        memory.tiles_dirty = check_write(&memory.char_ram[(address - 0x8000) as usize], &value);
        memory.char_ram[(address - 0x8000) as usize] = value;
    }
    else if address >= 0x9800 && address <= 0x9FFF
    {
        memory.background_dirty = check_write(&memory.bg_map[(address - 0x9800) as usize], &value);
        memory.bg_map[(address - 0x9800) as usize] = value;
    }
    else if address >= 0xA000 && address <= 0xBFFF 
    {
        if memory.loaded_cart.has_ram {
            memory.loaded_cart.cart_ram[(address - 0xA000) as usize] = value;
        }
        else {
            info!("Memory: Cart has no external RAM, ignoring write.");
        }
    }
    else if address >= 0xC000 && address <= 0xDFFF
    {
        memory.ram[(address - 0xC000) as usize] = value;
        memory.echo_ram[(address - 0xC000) as usize] = value;
    }
    else if address >= 0xE000 && address <= 0xFDFF 
    {
        warn!("Memory: Write to echo ram. Address {}, value {}.", format!("{:#X}", address), format!("{:#X}", value));
        memory.ram[(address - 0xE000) as usize] = value;
        memory.echo_ram[(address - 0xE000) as usize] = value;
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        memory.oam_mem[(address - 0xFE00) as usize] = value;
    }
    else if address >= 0xFEA0 && address <= 0xFEFF
    {
        warn!("Memory: Write to unusable memory at address {}, value {}. Ignoring...", format!("{:#X}", address), format!("{:#X}", value));
    }
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        // Basically here to print the output of blargg's tests.
        // Holds the values stored in FF01 until a line break, then prints them.
        if address == 0xFF01 {
            if value == 0xA {

                let mut idx: usize = 0;
                let mut new_string = String::from("");
                while idx < memory.serial_buffer.len() {
                    new_string.push(memory.serial_buffer[idx] as char);
                    idx += 1;
                }

                info!("Serial:  {} ", new_string);
                memory.serial_buffer = Vec::new();
            }
            else {
                memory.serial_buffer.push(value);
            }
        }
        memory.io_regs[(address - 0xFF00) as usize] = value;
    }
    else if address >= 0xFF80 && address <= 0xFFFE 
    {
        memory.hram[(address - 0xFF80) as usize] = value;
    }
    else if address == 0xFFFF
    {
        memory.interrupts = value;
    }
    else
    {
        panic!("Invalid or unimplemented write at {}", format!("{:#X}", address));
    }
}

fn check_write(old_value: &u8, new_value: &u8) -> bool {

    if old_value == new_value {
        false
    }
    else {
        true
    }
}