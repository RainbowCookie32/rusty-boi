use std::sync::mpsc;

use log::info;

use super::utils;
use super::opcodes;
use super::opcodes_prefixed;

use super::emulator::Interrupt;

use super::memory::MemoryOp;
use super::memory::MemoryAccess;

use byteorder::{ByteOrder, LittleEndian};

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

pub fn exec_loop(state: &mut CpuState, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), interrupt_state: &Interrupt) -> CycleResult {

    let mut current_state = state;
    let current_memory = memory;
    let mut result = current_state.last_result;
    let mut opcode = memory_read_u8(&current_state.pc.get(), memory);

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

    if current_state.interrupts_flag && interrupt_state.interrupt {

        stack_write(&mut current_state.sp, current_state.pc.get(), &memory.0);

        /*match interrupt_state.1 {
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
        }*/
    }

    if !current_state.halted {

        if current_state.pc.get() == 0x0100 {
            info!("CPU: Bootrom execution finished, starting loaded ROM.");
            //current_memory.bootrom_finished = true;
        }

        if current_state.pc.get() == 0x02A6 {
            info!("CPU: Tetris checkpoint.");
        }

        if current_state.pc.get() == 0x0358 {
            info!("CPU: Tetris checkpoint.");
        }
        
        if opcode == 0xCB {
            opcode = memory_read_u8(&(current_state.pc.get() + 1), memory);
            result = opcodes_prefixed::run_prefixed_instruction(&mut current_state, &current_memory, opcode);
        }
        else {
            result = opcodes::run_instruction(&mut current_state, &current_memory, opcode);
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

pub fn memory_read_u8(addr: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> u8 {

    let mem_request = MemoryAccess {
        operation: MemoryOp::Read,
        address: *addr,
        value: 0,
    };
    
    memory.0.send(mem_request).unwrap();
    memory.1.recv().unwrap()
}

pub fn memory_read_u16(addr: u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> u16 {

    let mut bytes: Vec<u8> = vec![0; 2];
    let mut target_addr = addr;
    let mut mem_request = MemoryAccess {
        operation: MemoryOp::Read,
        address: target_addr,
        value: 0,
    };

    let read_value: u16;
    
    memory.0.send(mem_request).unwrap();
    bytes[0] = memory.1.recv().unwrap();

    target_addr += 1;

    mem_request = MemoryAccess {
        operation: MemoryOp::Read,
        address: target_addr,
        value: 0,
    };

    memory.0.send(mem_request).unwrap();
    bytes[1] = memory.1.recv().unwrap();

    read_value = LittleEndian::read_u16(&bytes);
    read_value
}

pub fn memory_write(addr: &u16, val: u8, memory: &mpsc::Sender<MemoryAccess>) {

    let mem_request = MemoryAccess {
        operation: MemoryOp::Write,
        address: *addr,
        value: val,
    };
    
    memory.send(mem_request).unwrap();
}

pub fn stack_read(sp: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> u16 {

    let final_value: u16;
    let mut values: Vec<u8> = vec![0; 2];

    let mut mem_request = MemoryAccess {
        operation: MemoryOp::Read,
        address: sp.get_register(),
        value: 0,
    };
    
    memory.0.send(mem_request).unwrap();
    values[0] = memory.1.recv().unwrap();
    sp.increment();

    mem_request = MemoryAccess {
        operation: MemoryOp::Read,
        address: sp.get_register(),
        value: 0,
    };

    memory.0.send(mem_request).unwrap();
    values[1] = memory.1.recv().unwrap();
    sp.increment();

    final_value = LittleEndian::read_u16(&values);
    final_value
}

pub fn stack_write(sp: &mut CpuReg, value: u16, memory: &mpsc::Sender<MemoryAccess>) {

    sp.decrement();
    
    let mut mem_request = MemoryAccess {
        operation: MemoryOp::Write,
        address: sp.get_register(),
        value: utils::get_lb(value),
    };
    
    memory.send(mem_request).unwrap();
    sp.decrement();

    mem_request = MemoryAccess {
        operation: MemoryOp::Write,
        address: sp.get_register(),
        value: utils::get_rb(value),
    };

    mem_request.value = utils::get_rb(value);
    memory.send(mem_request).unwrap();
}

pub fn toggle_interrupts(state: &mut CpuState, value: u8) {

    state.ie_pending = true;
    state.new_ie_countdown = 2;
    state.new_ie_value = value;
}