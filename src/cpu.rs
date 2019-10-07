use log::{info, error};
use std::sync::mpsc::{Sender, Receiver};
use byteorder::{ByteOrder, LittleEndian};

use super::memory::{MemoryOp, MemoryAccess};
use super::{timer, utils, opcodes, opcodes_prefixed};
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
    pub stopped: bool,
    pub halt_bug: bool,
    pub last_result: CycleResult,

    pub interrupts: InterruptState,
        
    pub nops: u8,
}

#[derive(Copy, Clone)]
pub struct InterruptState {

    pub can_interrupt: bool,
    pub vblank_enabled: bool, 
    pub lcdc_enabled: bool,
    pub timer_enabled: bool,
    pub serial_enabled: bool,
    pub input_enabled: bool,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum CycleResult {

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
        stopped: false,
        halt_bug: false,
        last_result: CycleResult::Success,

        interrupts: InterruptState {
            can_interrupt: false, 
            vblank_enabled: false,
            lcdc_enabled: false,
            timer_enabled: false,
            serial_enabled: false,
            input_enabled: false,
        },

        nops: 0,
    };

    info!("CPU: CPU initialized");

    initial_state
}

pub fn exec_loop(cycles_tx: Sender<u32>, timer: (Sender<MemoryAccess>, Receiver<u8>), memory: (Sender<MemoryAccess>, Receiver<u8>)) {

    let current_memory = (memory.0, memory.1);
    let mut current_state = init_cpu();
    let mut timer_state = timer::init_timer();

    loop {

        handle_interrupts(&mut current_state, &current_memory);
        let mut opcode = memory_read_u8(&current_state.pc.get(), &current_memory);

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
            else if current_state.last_result == CycleResult::Stop {
                current_state.stopped = true;
                // Ugly, hacky, everything qualifies here probably.
                // TODO: Since the GPU implementation depends on the display to be enabled
                // to work, disabling it should do the job as well. Should get a more
                // elegant solution eventually.
                memory_write(&0xFF40, 0, &current_memory.0);
            }
            current_state.halt_bug = false;
        }
        else {
            current_state.halt_bug = true;
        }

        if current_state.last_result == CycleResult::InvalidOp || current_state.last_result == CycleResult::NopFlood {
            error!("CPU: Breaking execution, last state was {:#?}", current_state.last_result);
            break;
        }

        cycles_tx.send(current_state.cycles.get()).unwrap();
        timer::timer_cycle(&mut timer_state, current_state.cycles.get(), &timer);
    }
}

fn handle_interrupts(current_state: &mut CpuState, memory: &(Sender<MemoryAccess>, Receiver<u8>)) {

    let ie_value = memory_read_u8(&0xFFFF, memory);
    update_interrupts(ie_value, &mut current_state.interrupts);

    let mut if_value = memory_read_u8(&0xFF0F, memory);

    let vblank_interrupt = utils::check_bit(if_value, 0) && current_state.interrupts.vblank_enabled;
    let lcdc_interrupt = utils::check_bit(if_value, 1) && current_state.interrupts.lcdc_enabled;
    let timer_interrupt = utils::check_bit(if_value, 2) && current_state.interrupts.timer_enabled;
    let serial_interrupt = utils::check_bit(if_value, 3) && current_state.interrupts.serial_enabled;
    let input_interrupt = utils::check_bit(if_value, 4) && current_state.interrupts.input_enabled;

    if vblank_interrupt {

        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit_u8(if_value, 0);
            memory_write(&0xFF0F, if_value, &memory.0);
            stack_write(&mut current_state.sp, current_state.pc.get(), &memory.0);
            current_state.pc.set(0x0040);
        }
        current_state.halted = false;
    }
    else if lcdc_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit_u8(if_value, 1);
            memory_write(&0xFF0F, if_value, &memory.0);
            stack_write(&mut current_state.sp, current_state.pc.get(), &memory.0);
            current_state.pc.set(0x0048);
        }
        current_state.halted = false;
    }
    else if timer_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit_u8(if_value, 2);
            memory_write(&0xFF0F, if_value, &memory.0);
            stack_write(&mut current_state.sp, current_state.pc.get(), &memory.0);
            current_state.pc.set(0x0050);
        }
        current_state.halted = false;
    }
    else if serial_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit_u8(if_value, 3);
            memory_write(&0xFF0F, if_value, &memory.0);
            stack_write(&mut current_state.sp, current_state.pc.get(), &memory.0);
            current_state.pc.set(0x0058);
        }
        current_state.halted = false;
    }
    else if input_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit_u8(if_value, 4);
            memory_write(&0xFF0F, if_value, &memory.0);
            stack_write(&mut current_state.sp, current_state.pc.get(), &memory.0);
            current_state.pc.set(0x0060);
        }
        current_state.halted = false;
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

pub fn toggle_interrupts(state: &mut CpuState, value: bool) {

    state.interrupts.can_interrupt = value;
}

fn update_interrupts(new_value: u8, interrupts: &mut InterruptState) {

    interrupts.vblank_enabled = utils::check_bit(new_value, 0);
    interrupts.lcdc_enabled = utils::check_bit(new_value, 1);
    interrupts.timer_enabled = utils::check_bit(new_value, 2);
    interrupts.serial_enabled = utils::check_bit(new_value, 3);
    interrupts.input_enabled = utils::check_bit(new_value, 4);
}