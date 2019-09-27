use log::info;
use std::sync::mpsc::{Sender, Receiver};
use byteorder::{ByteOrder, LittleEndian};

use super::memory::{MemoryOp, MemoryAccess};
use super::{utils, opcodes, opcodes_prefixed};
use super::register::{CpuReg, Register, Pc, PcTrait, Cycles, CycleCounter};


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

    pub enabled_interrupts: u8,
    pub interrupts_flag: bool,
    pub will_toggle_ie: bool,
    pub new_ie_value: bool,
    pub countdown_to_ie: u8,
        
    pub nops: u8,
}

pub struct Interrupt {
    pub interrupt: bool,
    pub interrupt_type: InterruptType,
}

#[derive(PartialEq, Debug)]
pub enum InterruptType {

    Vblank,
    LcdcStat,
    Timer,
    Serial,
    ButtonPress,
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

        enabled_interrupts: 0,
        interrupts_flag: false,
        will_toggle_ie: false,
        new_ie_value: false,
        countdown_to_ie: 2,

        nops: 0,
    };

    info!("CPU: CPU initialized");

    initial_state
}

pub fn exec_loop(cycles_tx: Sender<u32>, memory: (Sender<MemoryAccess>, Receiver<u8>), interrupts: Receiver<Interrupt>) {

    let current_memory = memory;
    let mut current_state = init_cpu();
    let mut interrupt_state = Interrupt { 
        interrupt: false,
        interrupt_type: InterruptType::Vblank,
    };

    loop {

        let mut opcode = memory_read_u8(&current_state.pc.get(), &current_memory);
        let received_interrupt = interrupts.try_recv();

        match received_interrupt {
            Ok(result) => {
                interrupt_state = result;
            },
            Err(_error) => {},
        };

        if current_state.will_toggle_ie {

            if current_state.countdown_to_ie == 0 {
            
                current_state.interrupts_flag = current_state.new_ie_value;
                current_state.will_toggle_ie = false;
                current_state.countdown_to_ie = 2;
            }
            else {
                current_state.countdown_to_ie -= 1;
            }
        }

        if current_state.interrupts_flag && interrupt_state.interrupt {

            stack_write(&mut current_state.sp, current_state.pc.get(), &current_memory.0);

            match interrupt_state.interrupt_type {
                InterruptType::Vblank => if utils::check_bit(current_state.enabled_interrupts, 0) {
                    current_state.pc.set(0x40);
                    current_state.interrupts_flag = false;
                    current_state.halted = false;
                },
                InterruptType::LcdcStat => if utils::check_bit(current_state.enabled_interrupts, 1) {
                    current_state.pc.set(0x48);
                    current_state.interrupts_flag = false;
                    current_state.halted = false;
                },
                InterruptType::Timer => if utils::check_bit(current_state.enabled_interrupts, 2) {
                    current_state.pc.set(0x50);
                    current_state.interrupts_flag = false;
                    current_state.halted = false;
                },
                InterruptType::Serial => if utils::check_bit(current_state.enabled_interrupts, 3) {
                    current_state.pc.set(0x58);
                    current_state.interrupts_flag = false;
                    current_state.halted = false;
                },
                InterruptType::ButtonPress => if utils::check_bit(current_state.enabled_interrupts, 4) {
                    current_state.pc.set(0x60);
                    current_state.interrupts_flag = false;
                    current_state.halted = false;
                },
            }
        }

        if !current_state.halted {

            if current_state.pc.get() == 0x0100 {
                info!("CPU: Bootrom execution finished, starting loaded ROM.");
                current_memory.0.send(MemoryAccess{ operation: MemoryOp::BootromFinished, address: 0, value: 0 }).unwrap();
            }
        
            if opcode == 0xCB {
                opcode = memory_read_u8(&(current_state.pc.get() + 1), &current_memory);
                current_state.last_result = opcodes_prefixed::run_prefixed_instruction(&mut current_state, &current_memory, opcode);
            }
            else {
                current_state.last_result = opcodes::run_instruction(&mut current_state, &current_memory, opcode);
                if opcode == 0x00 {current_state.nops += 1;}
                else {current_state.nops = 0;}
                if current_state.nops >= 5 { current_state.last_result = CycleResult::NopFlood }
            }

            if current_state.last_result == CycleResult::Halt {
                current_state.halted = true;
            }
        }

        if current_state.last_result == CycleResult::InvalidOp || current_state.last_result == CycleResult::UnimplementedOp {
            break;
        }

        cycles_tx.send(current_state.cycles.get()).unwrap();
    }
}

pub fn memory_read_u8(addr: &u16, memory: &(Sender<MemoryAccess>, Receiver<u8>)) -> u8 {

    let mem_request = MemoryAccess {
        operation: MemoryOp::Read,
        address: *addr,
        value: 0,
    };
    
    memory.0.send(mem_request).unwrap();
    memory.1.recv().unwrap()
}

pub fn memory_read_u16(addr: u16, memory: &(Sender<MemoryAccess>, Receiver<u8>)) -> u16 {

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

pub fn memory_write(addr: &u16, val: u8, memory: &Sender<MemoryAccess>) {

    let mem_request = MemoryAccess {
        operation: MemoryOp::Write,
        address: *addr,
        value: val,
    };
    
    memory.send(mem_request).unwrap();
}

pub fn stack_read(sp: &mut CpuReg, memory: &(Sender<MemoryAccess>, Receiver<u8>)) -> u16 {

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

pub fn stack_write(sp: &mut CpuReg, value: u16, memory: &Sender<MemoryAccess>) {

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

    state.will_toggle_ie = true;
    state.countdown_to_ie = 2;
    state.new_ie_value = if value == 1 { true } else { false };
}