use std::convert::TryInto;
use byteorder::{ByteOrder, LittleEndian};

use log::trace;
use log::info;
use log::error;

use super::opcodes;
use super::opcodes_prefixed;

use super::register;
use super::register::PcTrait;

pub struct CpuState {
    
    pub af: register::CpuReg,
    pub bc: register::CpuReg,
    pub de: register::CpuReg,
    pub hl: register::CpuReg,
    pub sp: register::CpuReg,
    
    pub pc: register::Pc,
    pub cycles: register::Cycles,
    
    pub stack: Vec<u8>, 

    pub nops: u8,
}

pub struct Memory {

    pub loaded_bootrom: Vec<u8>,
    pub loaded_rom: Vec<u8>,

    pub ram: Vec<u8>,
    pub io_regs: Vec<u8>,

    pub char_ram: Vec<u8>,
    pub bg_map: Vec<u8>,
    pub oam_mem: Vec<u8>,

    pub tiles_dirty: bool,
    pub background_dirty: bool,
}

#[derive(PartialEq, Debug)]
pub enum CycleResult {

    UnimplementedOp,
    NopFlood,
    InvalidOp,
    Stop,
    Halt,
    Success,
}

pub fn init_cpu() -> CpuState {

    let initial_state = CpuState {
        af: register::CpuReg{value: 0x0000},
        bc: register::CpuReg{value: 0x0000},
        de: register::CpuReg{value: 0x0000},
        hl: register::CpuReg{value: 0x0000},
        sp: register::CpuReg{value: 0x0000},

        pc: register::Pc{value: 0x0}, // 0x0100 is the start PC for ROMs, 0x00 is for the bootrom
        cycles: register::Cycles{value: 0},

        stack: Vec::new(),

        nops: 0,
    };

    info!("CPU: CPU initialized");

    initial_state
}

pub fn init_memory(bootrom: Vec<u8>, rom: Vec<u8>) -> Memory {

    let initial_memory = Memory {

        loaded_bootrom: bootrom,
        loaded_rom: rom,

        ram: vec![0; 8192],
        io_regs: vec![0; 256],

        char_ram: vec![0; 6144],
        bg_map: vec![0; 2048],
        oam_mem: vec![0; 160],

        tiles_dirty: false,
        background_dirty: false,
    };

    info!("CPU: Memory initialized");

    initial_memory
}

pub fn exec_loop(state: &mut CpuState, memory: &mut Memory) -> CycleResult {

    let mut current_state = state;
    let mut current_memory = memory;
    let mut result: CycleResult;
    let mut opcode = memory_read_u8(&current_state.pc.get(), &current_memory);

    if current_state.pc.get() == 0x0100 {
        info!("CPU: Bootrom execution finished, starting loaded ROM.");
    }
        
    if opcode == 0xCB {
        opcode = memory_read_u8(&(current_state.pc.get() + 1), &current_memory);
        result = opcodes_prefixed::run_prefixed_instruction(&mut current_state, &mut current_memory, opcode);
    }
    else {
        result = opcodes::run_instruction(&mut current_state, &mut current_memory, opcode);
        if opcode == 0x00 {current_state.nops += 1;}
        else {current_state.nops = 0;}
        if current_state.nops >= 5 { result = CycleResult::NopFlood }
    }

    result
}

pub fn memory_read_u8(addr: &u16, memory: &Memory) -> u8 {

    let address: u16 = *addr;

    if address < 0x0100 
    {
        let memory_addr: usize = address.try_into().unwrap();
        memory.loaded_bootrom[memory_addr]
    }
    else if address >= 0x0100 && address <= 0x3FFF
    {
        let memory_addr: usize = address.try_into().unwrap();
        memory.loaded_rom[memory_addr]
    }
    else if address >= 0x4000 && address <= 0x7FFF
    {
        let memory_addr: usize = address.try_into().unwrap();
        memory.loaded_rom[memory_addr]
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
    else if address >= 0xC000 && address <= 0xCFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();
        memory.ram[memory_addr]
    }
    else if address >= 0xD000 && address <= 0xDFFF
    {
        let memory_addr: usize = (address - 0xD000).try_into().unwrap();
        memory.ram[memory_addr]
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        let memory_addr: usize = (address - 0xFE00).try_into().unwrap();
        memory.oam_mem[memory_addr]
    }
    else if address >= 0xFEA0 && address <= 0xFEFF
    {
        error!("CPU: Read to unusable memory at address {}. Returning 0", format!("{:#X}", address));
        0
    }
    else if address >= 0xFF00
    {
        let memory_addr: usize = (address - 0xFF00).try_into().unwrap();
        memory.io_regs[memory_addr]
    }
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", addr));
    }
}

pub fn memory_read_u16(addr: &u16, memory: &Memory) -> u16 {

    let address: u16 = *addr;
    let mut target: Vec<u8> = vec![0; 2];
    let target_addr: u16;

    if address < 0x0100
    {
        let memory_addr: usize = address.try_into().unwrap();
        target[0] = memory.loaded_bootrom[memory_addr];
        target[1] = memory.loaded_bootrom[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0x0100 && address <= 0x3FFF
    {
        let memory_addr: usize = address.try_into().unwrap();
        target[0] = memory.loaded_rom[memory_addr];
        target[1] = memory.loaded_rom[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0x4000 && address <= 0x7FFF
    {
        let memory_addr: usize = address.try_into().unwrap();
        target[0] = memory.loaded_rom[memory_addr];
        target[1] = memory.loaded_rom[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0x8000 && address <= 0x97FF
    {
        let memory_addr: usize = address.try_into().unwrap();
        target[0] = memory.char_ram[memory_addr];
        target[1] = memory.char_ram[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0x9800 && address <= 0x9FFF
    {
        let memory_addr: usize = address.try_into().unwrap();
        target[0] = memory.bg_map[memory_addr];
        target[1] = memory.bg_map[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0xC000 && address <= 0xCFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();
        target[0] = memory.ram[memory_addr];
        target[1] = memory.ram[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0xD000 && address <= 0xDFFF
    {
        let memory_addr: usize = (address - 0xD000).try_into().unwrap();
        target[0] = memory.ram[memory_addr];
        target[1] = memory.ram[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        let memory_addr: usize = (address - 0xFE00).try_into().unwrap();
        target[0] = memory.oam_mem[memory_addr];
        target[1] = memory.oam_mem[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0xFEA0 && address <= 0xFEFF
    {
        error!("CPU: Read to unusable memory at address {}. Returning 0", format!("{:#X}", addr));
        0
    }
    else if address >= 0xFF00
    {
        let memory_addr: usize = (address - 0xFF00).try_into().unwrap();
        target[0] = memory.io_regs[memory_addr];
        target[1] = memory.io_regs[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", addr));
    }
}

pub fn memory_write(address: u16, value: u8, memory: &mut Memory) {

    if address <= 0x3FFF
    {
        error!("CPU: Tried to write to cart, illegal write");
    }
    else if address >= 0x4000 && address <= 0x7FFF
    {
        error!("CPU: Tried to write to cart, illegal write");
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
    else if address >= 0xC000 && address <= 0xCFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();
        memory.ram[memory_addr] = value;
    }
    else if address >= 0xD000 && address <= 0xDFFF
    {
        let memory_addr: usize = (address - 0xD000).try_into().unwrap();
        memory.ram[memory_addr] = value;
    }
    else if address >= 0xFE00 && address <= 0xFE9F 
    {
        let memory_addr: usize = (address - 0xFE00).try_into().unwrap();
        memory.oam_mem[memory_addr] = value;
    }
    else if address >= 0xFEA0 && address <= 0xFEFF
    {
        error!("CPU: Write to unusable memory at address {}. Ignoring...", format!("{:#X}", address));
    }
    else if address >= 0xFF00
    {
        let memory_addr: usize = (address - 0xFF00).try_into().unwrap();
        memory.io_regs[memory_addr] = value;
    }
    else
    {
        panic!("Invalid or unimplemented write at {}", format!("{:#X}", address));
    }
}

fn check_write(old_value: &u8, new_value: &u8) -> bool {

    if old_value == new_value {
        trace!("CPU: Old value in memory ({}) is the same as ({}), not marking as dirty", format!("{:#X}", old_value), format!("{:#X}", new_value));
        false
    }
    else {
        true
    }
}