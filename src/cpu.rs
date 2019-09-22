use std::convert::TryInto;
use byteorder::{ByteOrder, LittleEndian};

use log::trace;
use log::info;
use log::error;

use super::utils;
use super::opcodes;
use super::opcodes_prefixed;

use super::emulator::Interrupt;

use super::register::{CpuReg, Register, Pc, PcTrait, Cycles};

pub struct CpuState {
    
    pub af: CpuReg,
    pub bc: CpuReg,
    pub de: CpuReg,
    pub hl: CpuReg,
    pub sp: CpuReg,
    
    pub pc: Pc,
    pub cycles: Cycles,

    pub halted: bool,
    pub last_result: CycleResult,

    pub interrupts_flag: bool,
    pub ie_pending: bool,
    pub new_ie_value: u8,
    pub new_ie_countdown: u8,
        
    pub nops: u8,
}

pub struct Memory {

    pub loaded_bootrom: Vec<u8>,
    pub loaded_rom: Vec<u8>,

    pub ram: Vec<u8>,
    pub io_regs: Vec<u8>,
    pub hram: Vec<u8>,
    pub interrupts: u8,

    pub char_ram: Vec<u8>,
    pub bg_map: Vec<u8>,
    pub oam_mem: Vec<u8>,

    pub tiles_dirty: bool,
    pub background_dirty: bool,
    pub bootrom_finished: bool,
}

#[derive(PartialEq, Debug, Copy, Clone)]
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
        af: CpuReg{value: 0x0000},
        bc: CpuReg{value: 0x0000},
        de: CpuReg{value: 0x0000},
        hl: CpuReg{value: 0x0000},
        sp: CpuReg{value: 0x0000},

        pc: Pc{value: 0x0}, // 0x0100 is the start PC for ROMs, 0x00 is for the bootrom
        cycles: Cycles{value: 0},

        halted: false,
        last_result: CycleResult::Success,

        interrupts_flag: false,
        ie_pending: false,
        new_ie_value: 0,
        new_ie_countdown: 2,

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
        hram: vec![0; 127],
        interrupts: 0,

        char_ram: vec![0; 6144],
        bg_map: vec![0; 2048],
        oam_mem: vec![0; 160],

        tiles_dirty: false,
        background_dirty: false,
        bootrom_finished: false,
    };

    info!("CPU: Memory initialized");

    initial_memory
}

pub fn exec_loop(state: &mut CpuState, memory: &mut Memory, interrupt_state: &mut (bool, Interrupt)) -> CycleResult {

    let mut current_state = state;
    let mut current_memory = memory;
    let mut result = current_state.last_result;
    let mut opcode = memory_read_u8(&current_state.pc.get(), &current_memory);

    if current_state.ie_pending {

        if current_state.new_ie_countdown == 0 {
            
            current_state.interrupts_flag = true;
            current_state.ie_pending = false;
            current_state.new_ie_countdown = 2;
            current_state.new_ie_value = 0;
        }
        else {
            current_state.new_ie_countdown -= 1;
        }
    }

    if current_state.interrupts_flag && interrupt_state.0 {

        stack_write(&mut current_state.sp, current_state.pc.get(), &mut current_memory);

        match interrupt_state.1 {
            Interrupt::Vblank => if utils::check_bit(current_memory.interrupts, 0) {
                current_state.pc.set(0x40);
                current_state.interrupts_flag = false;
                current_state.halted = false;
            },
            Interrupt::LcdcStat => if utils::check_bit(current_memory.interrupts, 1) {
                current_state.pc.set(0x48);
                current_state.interrupts_flag = false;
                current_state.halted = false;
            },
            Interrupt::Timer => if utils::check_bit(current_memory.interrupts, 2) {
                current_state.pc.set(0x50);
                current_state.interrupts_flag = false;
                current_state.halted = false;
            },
            Interrupt::Serial => if utils::check_bit(current_memory.interrupts, 3) {
                current_state.pc.set(0x58);
                current_state.interrupts_flag = false;
                current_state.halted = false;
            },
            Interrupt::ButtonPress => if utils::check_bit(current_memory.interrupts, 4) {
                current_state.pc.set(0x60);
                current_state.interrupts_flag = false;
                current_state.halted = false;
            },
        }
    }

    if !current_state.halted {

        if current_state.pc.get() == 0x0100 {
            info!("CPU: Bootrom execution finished, starting loaded ROM.");
            current_memory.bootrom_finished = true;
        }

        if current_state.pc.get() == 0x02A6 {
            info!("CPU: Tetris checkpoint.");
        }

        if current_state.pc.get() == 0x0358 {
            info!("CPU: Tetris checkpoint.");
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

        if result == CycleResult::Halt {
            current_state.halted = true;
        }
    }

    result
}

pub fn memory_read_u8(addr: &u16, memory: &Memory) -> u8 {

    let address: u16 = *addr;

    if address < 0x0100 
    {
        let memory_addr: usize = address.try_into().unwrap();
        if memory.bootrom_finished {
            memory.loaded_rom[memory_addr]
        }
        else {
            memory.loaded_bootrom[memory_addr]
        }
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
    else if address >= 0xA000 && address <= 0xBFFF 
    {
        error!("CPU: Unimplemented read at {}, returning 0", format!("{:#X}", address));
        0
    }
    else if address >= 0xC000 && address <= 0xDFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();
        memory.ram[memory_addr]
    }
    else if address >= 0xE000 && address <= 0xFDFF 
    {
        error!("CPU: Unimplemented read at {}, returning 0", format!("{:#X}", address));
        0
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

pub fn memory_read_u16(addr: &u16, memory: &Memory) -> u16 {

    let address: u16 = *addr;
    let mut target: Vec<u8> = vec![0; 2];
    let target_addr: u16;

    if address < 0x0100
    {
        let memory_addr: usize = address.try_into().unwrap();

        if memory.bootrom_finished {
            target[0] = memory.loaded_rom[memory_addr];
            target[1] = memory.loaded_rom[memory_addr + 1];
            target_addr = LittleEndian::read_u16(&target);
            target_addr
        }
        else {
            target[0] = memory.loaded_bootrom[memory_addr];
            target[1] = memory.loaded_bootrom[memory_addr + 1];
            target_addr = LittleEndian::read_u16(&target);
            target_addr
        }
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
    else if address >= 0xA000 && address <= 0xBFFF 
    {
        error!("CPU: Unimplemented read at {}, returning 0", format!("{:#X}", address));
        0
    }
    else if address >= 0xC000 && address <= 0xDFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();
        target[0] = memory.ram[memory_addr];
        target[1] = memory.ram[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0xE000 && address <= 0xFDFF 
    {
        error!("CPU: Unimplemented read at {}, returning 0", format!("{:#X}", address));
        0
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
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        let memory_addr: usize = (address - 0xFF00).try_into().unwrap();
        target[0] = memory.io_regs[memory_addr];
        target[1] = memory.io_regs[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0xFF80 && address <= 0xFFFE 
    {
        let memory_addr: usize = (address - 0xFF80).try_into().unwrap();
        target[0] = memory.hram[memory_addr];
        target[1] = memory.hram[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", addr));
    }
}

pub fn memory_write(address: u16, value: u8, memory: &mut Memory) {

    if address <= 0x7FFF
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
    else if address >= 0xA000 && address <= 0xBFFF 
    {
        error!("CPU: Write to unimplemented area at address {}. Ignoring...", format!("{:#X}", address));
    }
    else if address >= 0xC000 && address <= 0xDFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();
        memory.ram[memory_addr] = value;
    }
    else if address >= 0xE000 && address <= 0xFDFF 
    {
        error!("CPU: Write to unimplemented area at address {}. Ignoring...", format!("{:#X}", address));
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
    else if address >= 0xFF00 && address <= 0xFF7F
    {
        let memory_addr: usize = (address - 0xFF00).try_into().unwrap();

        if address == 0xFF44 {
            memory.io_regs[memory_addr] = 0;
        }
        else {
            memory.io_regs[memory_addr] = value;
        }   
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

pub fn stack_read(sp: &mut CpuReg, memory: &mut Memory) -> u16 {

    let final_value: u16;
    let mut values: Vec<u8> = vec![0; 2];
    
    values[0] = memory_read_u8(&sp.get_register(), memory);
    sp.increment();
    values[1] = memory_read_u8(&sp.get_register(), memory);
    sp.increment();
    final_value = LittleEndian::read_u16(&values);
    final_value
}

pub fn stack_write(sp: &mut CpuReg, value: u16, memory: &mut Memory) {

    sp.decrement();
    memory_write(sp.get_register(), utils::get_lb(value), memory);
    sp.decrement();
    memory_write(sp.get_register(), utils::get_rb(value), memory);
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

pub fn toggle_interrupts(state: &mut CpuState, value: u8) {

    state.ie_pending = true;
    state.new_ie_countdown = 2;
    state.new_ie_value = value;
}