use std::convert::TryInto;
use std::time;
use std::thread;
use byteorder::{ByteOrder, LittleEndian};

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

    pub should_execute: bool,
    pub nops: u8,
}

pub struct Memory {

    pub loaded_bootrom: Vec<u8>,
    pub loaded_rom: Vec<u8>,

    pub ram: Vec<u8>,
    pub vram: Vec<u8>,
    pub io_regs: Vec<u8>,
}

pub fn init_cpu() -> CpuState {

    let initial_state = CpuState {
        af: register::CpuReg{value: 0x01B0},
        bc: register::CpuReg{value: 0x0013},
        de: register::CpuReg{value: 0x00D8},
        hl: register::CpuReg{value: 0x014D},
        sp: register::CpuReg{value: 0xFFFE},

        pc: register::Pc{value: 0x0000}, // 0x0100 is the start value for ROMS, 0x0000 is for the bootrom
        cycles: register::Cycles{value: 0},

        stack: Vec::new(),

        should_execute: true,
        nops: 0,
    };

    println!("CPU initialized");

    initial_state
}

pub fn init_memory(bootrom: Vec<u8>, rom: Vec<u8>) -> Memory {

    let initial_memory = Memory {

        loaded_bootrom: bootrom,
        loaded_rom: rom,

        ram: vec![0; 8192],
        vram: vec![0; 8192],
        io_regs: vec![0; 256],
    };

    println!("Memory initialized");

    initial_memory
}

pub fn exec_loop(state: &mut CpuState, memory: &mut Memory) {

    let mut current_state = state;
    let mut current_memory = memory;
    let slow_mode = true;
    
    let mut opcode = memory_read_u8(&current_state.pc.get(), &current_memory);
        
    if opcode == 0xCB {
        opcode = memory_read_u8(&(current_state.pc.get() + 1), &current_memory);
        opcodes_prefixed::run_prefixed_instruction(&mut current_state, &mut current_memory, opcode);
    }
    else {
        opcodes::run_instruction(&mut current_state, &mut current_memory, opcode);
    }

    if slow_mode {thread::sleep(time::Duration::from_millis(150))};
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
    else if address >= 0x8000 && address <= 0x9FFF
    {
        let memory_addr: usize = (addr - 0x8000).try_into().unwrap();
        memory.vram[memory_addr]
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
    else if address >= 0x8000 && address <= 0x9FFF
    {
        let memory_addr: usize = (address - 0x8000).try_into().unwrap();
        target[0] = memory.vram[memory_addr];
        target[1] = memory.vram[memory_addr + 1];
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
        panic!("Tried to write to cart, illegal write");
    }
    else if address >= 0x4000 && address <= 0x7FFF
    {
        panic!("Tried to write to cart, illegal write");
    }
    else if address >= 0x8000 && address <= 0x9FFF
    {
        let memory_addr: usize = (address - 0x8000).try_into().unwrap();
        memory.vram[memory_addr] = value;
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
    else if address >= 0xFF00
    {
        let memory_addr: usize = (address - 0xFF00).try_into().unwrap();
        memory.io_regs[memory_addr] = value;
    }
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", address));
    }
}