use log::{info, error};
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
    pub stopped: bool,
    pub last_result: CycleResult,

    pub interrupts: InterruptState,
    pub interrupts_toggled: bool,
    pub new_interrupts_value: bool,
    pub instructions_since_toggle: u8,
        
    pub nops: u8,
}

#[derive(PartialEq, Debug)]
pub enum InterruptType {

    Vblank,
    LcdcStat,
    Timer,
    Serial,
    ButtonPress,
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
        stopped: false,
        last_result: CycleResult::Success,

        interrupts: InterruptState {
            can_interrupt: false, 
            vblank_enabled: false,
            lcdc_enabled: false,
            timer_enabled: false,
            serial_enabled: false,
            input_enabled: false,
        },
        interrupts_toggled: false,
        new_interrupts_value: false,
        instructions_since_toggle: 0,

        nops: 0,
    };

    info!("CPU: CPU initialized");

    initial_state
}

pub fn exec_loop(interrupts: (Receiver<(bool, InterruptType)>, Sender<InterruptState>), cycles_tx: Sender<u32>, memory: (Sender<MemoryAccess>, Receiver<u8>, Receiver<u8>)) {

    let current_memory = (memory.0, memory.1);
    let mut current_state = init_cpu();

    loop {

        let updated_interrupts = memory.2.try_recv();

        match updated_interrupts {
            Ok(value) => update_interrupts(value, &mut current_state.interrupts),
            Err(_error) => {},
        };

        interrupts.1.send(current_state.interrupts).unwrap();

        let received_interrupt = interrupts.0.try_recv();
        let current_interrupt = match received_interrupt {
            Ok(message) => message,
            Err(_err) => { (false, InterruptType::Vblank) },
        };

        if current_state.interrupts_toggled {

            if current_state.instructions_since_toggle == 2 {
                current_state.interrupts.can_interrupt = true;
                current_state.interrupts_toggled = false;
                current_state.instructions_since_toggle = 0;
            }
            else {
                current_state.instructions_since_toggle += 1;
            }
        }

        if current_state.interrupts.can_interrupt && current_interrupt.0 {

            match current_interrupt.1 {
                InterruptType::Vblank => {
                    if current_state.interrupts.vblank_enabled {
                        stack_write(&mut current_state.sp, current_state.pc.get(), &current_memory.0);
                        current_state.halted = false;
                        current_state.interrupts.can_interrupt = false;
                        current_state.pc.set(0x0040);
                    }
                },
                InterruptType::LcdcStat => {
                    if current_state.interrupts.lcdc_enabled {
                        stack_write(&mut current_state.sp, current_state.pc.get(), &current_memory.0);
                        current_state.halted = false;
                        current_state.interrupts.can_interrupt = false;
                        current_state.pc.set(0x0048);
                    }
                },
                InterruptType::Timer => {
                    if current_state.interrupts.timer_enabled {
                        stack_write(&mut current_state.sp, current_state.pc.get(), &current_memory.0);
                        current_state.halted = false;
                        current_state.interrupts.can_interrupt = false;
                        current_state.pc.set(0x0050);
                    }
                },
                InterruptType::Serial => {
                    if current_state.interrupts.serial_enabled {
                        stack_write(&mut current_state.sp, current_state.pc.get(), &current_memory.0);
                        current_state.halted = false;
                        current_state.interrupts.can_interrupt = false;
                        current_state.pc.set(0x0058);
                    }
                },
                InterruptType::ButtonPress => {
                    if current_state.interrupts.input_enabled {
                        stack_write(&mut current_state.sp, current_state.pc.get(), &current_memory.0);
                        current_state.halted = false;
                        current_state.interrupts.can_interrupt = false;
                        current_state.pc.set(0x0060);
                        if current_state.stopped {
                            memory_write(&0xFF40, 1, &current_memory.0);
                            current_state.stopped = false;
                        }
                    }
                },
            }
        }

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
                // Since the GPU implementation depends on the display to be enabled
                // to work, disabling it should do the job as well. Should get a more
                // elegant solution eventually. TODO
                memory_write(&0xFF40, 0, &current_memory.0);
            }
        }

        if current_state.last_result == CycleResult::InvalidOp || current_state.last_result == CycleResult::UnimplementedOp {
            error!("CPU: Breaking execution, last state was {:#?}", current_state.last_result);
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

    state.interrupts_toggled = true;
    state.new_interrupts_value = if value == 1 {true} else {false};
    state.instructions_since_toggle = 0;
}

fn update_interrupts(new_value: u8, interrupts: &mut InterruptState) {

    interrupts.vblank_enabled = utils::check_bit(new_value, 0);
    interrupts.lcdc_enabled = utils::check_bit(new_value, 1);
    interrupts.timer_enabled = utils::check_bit(new_value, 2);
    interrupts.serial_enabled = utils::check_bit(new_value, 3);
    interrupts.input_enabled = utils::check_bit(new_value, 4);
}