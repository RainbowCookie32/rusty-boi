use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::convert::TryInto;

use super::emulator::Cart;

use log::{info, warn};

pub struct Memory {

    pub loaded_bootrom: Vec<u8>,
    pub loaded_rom: Cart,
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

pub enum MemoryOp {
    
    Read,
    Write,
    BootromFinished,
}

pub struct MemoryAccess {

    pub operation: MemoryOp,
    pub address: u16,
    pub value: u8,
}

pub struct GpuResponse {

    pub tiles_dirty: bool,
    pub background_dirty: bool,
    pub read_value: u8,
}

// Not really *that* necessary, but it's a cleaner
// approach when passing around all the receiver and transmitters.
pub struct ThreadComms {

    pub cpu: ((Sender<MemoryAccess>, Receiver<u8>)),
    pub gpu: (Sender<MemoryAccess>, Receiver<GpuResponse>, Sender<bool>),
    pub timer: ((Sender<MemoryAccess>, Receiver<u8>)),
}

pub fn start_memory(data: (Vec<u8>, Cart), sender: Sender<ThreadComms>) {

    let (cpu_req_tx, cpu_req_rx) = mpsc::channel();
    let (cpu_res_tx, cpu_res_rx) = mpsc::channel();
    
    let (timer_req_tx, timer_req_rx) = mpsc::channel();
    let (timer_res_tx, timer_res_rx) = mpsc::channel();

    let (gpu_req_tx, gpu_req_rx) = mpsc::channel();
    let (gpu_res_tx, gpu_res_rx) = mpsc::channel();
    let (gpu_cache_tx, gpu_cache_rx) = mpsc::channel();

    let care_package = ThreadComms {
        cpu: (cpu_req_tx, cpu_res_rx),
        gpu: (gpu_req_tx, gpu_res_rx, gpu_cache_tx),
        timer: (timer_req_tx, timer_res_rx),
    };

    let initial_memory = Memory {

        loaded_bootrom: data.0,
        loaded_rom: data.1,
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

    let mut current_memory = initial_memory;
    sender.send(care_package).unwrap();

    loop {

        let cpu_request = cpu_req_rx.try_recv();
        let gpu_request = gpu_req_rx.try_recv();
        let gpu_cache = gpu_cache_rx.try_recv();

        let timer_request = timer_req_rx.try_recv();

        match cpu_request {
            Ok(request) => handle_cpu_request(&request, &cpu_res_tx, &mut current_memory),
            Err(_error) => {},
        };

        match gpu_request {
            Ok(request) => handle_gpu_request(&request, &gpu_res_tx, &mut current_memory),
            Err(_error) => {},
        };

        match timer_request {
            Ok(request) => handle_timer_request(&request, &timer_res_tx, &mut current_memory),
            Err(_error) => {},
        };

        match gpu_cache {
            Ok(status) => {
                current_memory.background_dirty = status;
                current_memory.tiles_dirty = status;
            },
            Err(_error) => {},
        }
    }
}

fn handle_cpu_request(request: &MemoryAccess, tx: &Sender<u8>, current_memory: &mut Memory) {

    let result_value: u8;
    
    match request.operation {
        MemoryOp::Read => {
            result_value = memory_read(&request.address, current_memory);
            tx.send(result_value).unwrap();
        },
        MemoryOp::Write => {

            if request.address == 0xFF04 || request.address == 0xFF44 { 
                memory_write(request.address, 0, current_memory);
            }
            else {
                memory_write(request.address, request.value, current_memory);
            }
        },
        MemoryOp::BootromFinished => {
            current_memory.bootrom_finished = true;
        },
    }
}

fn handle_gpu_request(request: &MemoryAccess, tx: &Sender<GpuResponse>, current_memory: &mut Memory) {

    let result_value: u8;
    
    match request.operation {
        MemoryOp::Read => {

            result_value = memory_read(&request.address, current_memory);
            
            let response = GpuResponse {
                tiles_dirty: current_memory.tiles_dirty,
                background_dirty: current_memory.background_dirty,
                read_value: result_value,
            };

            tx.send(response).unwrap();
        },
        MemoryOp::Write => memory_write(request.address, request.value, current_memory),
        MemoryOp::BootromFinished => {
            warn!("Memory: GPU triggered a BootromFinished event for some reason");
        },
    }
}

fn handle_timer_request(request: &MemoryAccess, tx: &Sender<u8>, current_memory: &mut Memory) {

    let result_value: u8;
    
    match request.operation {
        MemoryOp::Read => {
            result_value = memory_read(&request.address, current_memory);
            tx.send(result_value).unwrap();
        },
        MemoryOp::Write => {
            memory_write(request.address, request.value, current_memory);
        },
        MemoryOp::BootromFinished => {
            warn!("Memory: Timer triggered a BootromFinished event");
        },
    }
}

pub fn memory_read(addr: &u16, memory: &Memory) -> u8 {

    let address: u16 = *addr;

    if address < 0x0100 
    {
        let memory_addr: usize = address.try_into().unwrap();
        if memory.bootrom_finished {
            memory.loaded_rom.rom_banks[0][memory_addr]
        }
        else {
            memory.loaded_bootrom[memory_addr]
        }
    }
    else if address >= 0x0100 && address <= 0x3FFF
    {
        let memory_addr: usize = address.try_into().unwrap();
        memory.loaded_rom.rom_banks[0][memory_addr]
    }
    else if address >= 0x4000 && address <= 0x7FFF
    {
        let memory_addr: usize = (addr - 0x4000).try_into().unwrap();
        memory.loaded_rom.rom_banks[memory.selected_bank as usize][memory_addr]
    }
    else if address >= 0x8000 && address <= 0x97FF
    {
        let memory_addr: usize = (addr - 0x8000).try_into().unwrap();
        memory.char_ram[memory_addr]
    }
    else if address >= 0x9800 && address <= 0x9FFF
    {
        let memory_addr: usize = (addr - 0x9800).try_into().unwrap();
        memory.bg_map[memory_addr]
    }
    else if address >= 0xA000 && address <= 0xBFFF 
    {
        if memory.loaded_rom.has_ram {
            let memory_addr: usize = (addr - 0xA000).try_into().unwrap();
            memory.loaded_rom.cart_ram[memory_addr]
        }
        else {
            info!("Memory: Cart has no external RAM, returning 0.");
            0
        }
    }
    else if address >= 0xC000 && address <= 0xDFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();
        memory.ram[memory_addr]
    }
    else if address >= 0xE000 && address <= 0xFDFF 
    {
        let memory_addr: usize = (address - 0xE000).try_into().unwrap();
        memory.echo_ram[memory_addr]
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        let memory_addr: usize = (address - 0xFE00).try_into().unwrap();
        memory.oam_mem[memory_addr]
    }
    else if address >= 0xFEA0 && address <= 0xFEFF
    {
        warn!("Memory: Read to unusable memory at address {}. Returning 0", format!("{:#X}", address));
        0
    }
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        let memory_addr: usize = (address - 0xFF00).try_into().unwrap();
        memory.io_regs[memory_addr]
    }
    else if address >= 0xFF80 && address <= 0xFFFE
    {
        let memory_addr: usize = (address - 0xFF80).try_into().unwrap();
        memory.hram[memory_addr]
    }
    else if address == 0xFFFF
    {
        memory.interrupts
    }
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", addr));
    }
}

pub fn memory_write(address: u16, value: u8, memory: &mut Memory) {

    if address <= 0x7FFF
    {
        info!("Memory: Switching ROM Bank to {}", value);
        memory.selected_bank = value;
    }
    else if address >= 0x8000 && address <= 0x97FF
    {
        let memory_addr: usize = (address - 0x8000).try_into().unwrap();
        // A simple check that avoids marking tiles as dirty if the old value is the same as the new one.
        // The best example here is the bootrom's first loop that zeroes VRAM. Both the initial value and the new one are 0.
        // Regenerating caches there is useless.
        memory.tiles_dirty = check_write(&memory.char_ram[memory_addr], &value);
        memory.char_ram[memory_addr] = value;
    }
    else if address >= 0x9800 && address <= 0x9FFF
    {
        let memory_addr: usize = (address - 0x9800).try_into().unwrap();
        // A simple check that avoids marking the background as dirty if the old value is the same as the new one.
        memory.background_dirty = check_write(&memory.bg_map[memory_addr], &value);
        memory.bg_map[memory_addr] = value;
    }
    else if address >= 0xA000 && address <= 0xBFFF 
    {
        if memory.loaded_rom.has_ram {
            let memory_addr: usize = (address - 0xA000).try_into().unwrap();
            memory.loaded_rom.cart_ram[memory_addr] = value;
        }
        else {
            info!("Memory: Cart has no external RAM, ignoring write.");
        }
    }
    else if address >= 0xC000 && address <= 0xDFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();
        memory.ram[memory_addr] = value;
        memory.echo_ram[memory_addr] = value;
    }
    else if address >= 0xE000 && address <= 0xFDFF 
    {
        warn!("Memory: Write to echo ram. Address {}, value {}.", format!("{:#X}", address), format!("{:#X}", value));
        let memory_addr: usize = (address - 0xE000).try_into().unwrap();
        memory.ram[memory_addr] = value;
        memory.echo_ram[memory_addr] = value;
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        let memory_addr: usize = (address - 0xFE00).try_into().unwrap();
        memory.oam_mem[memory_addr] = value;
    }
    else if address >= 0xFEA0 && address <= 0xFEFF
    {
        warn!("Memory: Write to unusable memory at address {}, value {}. Ignoring...", format!("{:#X}", address), format!("{:#X}", value));
    }
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        let memory_addr: usize = (address - 0xFF00).try_into().unwrap();
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
        memory.io_regs[memory_addr] = value;
    }
    else if address >= 0xFF80 && address <= 0xFFFE 
    {
        let memory_addr: usize = (address - 0xFF80).try_into().unwrap();
        memory.hram[memory_addr] = value;
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