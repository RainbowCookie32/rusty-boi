use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

use log::{info, error};
use byteorder::{ByteOrder, LittleEndian};

use super::memory;
use super::memory::{CpuMemory, IoRegisters, GpuMemory};

use super::emulator::InputEvent;
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

impl CpuState {

    pub fn new(bootrom: bool) -> CpuState {
    
        CpuState {
            af: CpuReg{value: if bootrom {0x0000} else {0x01B0}},
            bc: CpuReg{value: if bootrom {0x0000} else {0x0013}},
            de: CpuReg{value: if bootrom {0x0000} else {0x00D8}},
            hl: CpuReg{value: if bootrom {0x0000} else {0x014D}},
            sp: CpuReg{value: if bootrom {0x0000} else {0xFFFE}},

            pc: Pc{value: if bootrom {0x00} else {0x0100}},
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
        }
    }
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

pub fn start_cpu(cycles: Arc<Mutex<u16>>, memory: (CpuMemory, Arc<Mutex<IoRegisters>>, Arc<Mutex<GpuMemory>>), input: Receiver<InputEvent>) {

    let mut current_state = CpuState::new(!memory.0.bootrom_finished);
    let mut timer_state = timer::init_timer();

    let mut cpu_memory = memory.0;
    let shared_memory = (memory.1, memory.2);

    loop {
        
        let input_value = memory::cpu_read(0xFF00, &mut cpu_memory, &shared_memory);
        if input_value == 0x30 || input_value == 0x20 || input_value == 0x10 {
            if update_inputs(&input, &mut cpu_memory, &shared_memory) {break}
        }
        handle_interrupts(&mut current_state, &mut cpu_memory, &shared_memory);
        let mut opcode = memory::cpu_read(current_state.pc.get(), &mut cpu_memory, &shared_memory);

        if !current_state.halted {
            
            if current_state.pc.get() == 0x0100 {
                info!("CPU: Bootrom execution finished, starting loaded ROM.");
                cpu_memory.bootrom_finished = true;
            }
        
            if opcode == 0xCB {
                opcode = read_immediate(current_state.pc.get(), &mut cpu_memory, &shared_memory);
                current_state.last_result = opcodes_prefixed::run_opcode(&mut current_state, opcode, &mut cpu_memory, &shared_memory);
            }
            else {
                current_state.last_result = opcodes::run_opcode(&mut current_state, opcode, &mut cpu_memory, &shared_memory);
                if opcode == 0x00 {current_state.nops += 1;}
                else {current_state.nops = 0;}
                if current_state.nops >= 5 { current_state.last_result = CycleResult::NopFlood }
            }

            if current_state.last_result == CycleResult::Halt {
                current_state.halted = true;
            }
            else if current_state.last_result == CycleResult::Stop {
                current_state.stopped = true;
                // TODO: Since the GPU implementation depends on the display to be enabled
                // to work, disabling it should do the job as well. Should get a more
                // elegant solution eventually.
                memory::cpu_write(0xFF40, 0, &mut cpu_memory, &shared_memory);
            }
        }

        if current_state.last_result == CycleResult::InvalidOp || current_state.last_result == CycleResult::NopFlood {
            error!("CPU: Breaking execution, last state was {:#?}", current_state.last_result);
            break;
        }

        let mut cyc_mut = cycles.lock().unwrap();
        *cyc_mut = cyc_mut.overflowing_add(current_state.cycles.get()).0;
        timer::timer_cycle(&mut timer_state, current_state.cycles.get(), &shared_memory.0);
    }
}

fn update_inputs(input_rx: &Receiver<InputEvent>, cpu_mem: &mut CpuMemory, shared_mem: &(Arc<Mutex<IoRegisters>>, Arc<Mutex<GpuMemory>>)) -> bool {

    let received_input: bool;
    let input_event = input_rx.try_recv();

    let mut should_break = false;
    let mut received_message = InputEvent::APressed;
    // Read the value of the input register, and default all input bits to 1.
    // The lower 4 bits are set when there's no input, and reset when there's a button press.
    let mut input_value = memory::cpu_read(0xFF00, cpu_mem, shared_mem) | 0xCF;

    match input_event {
        Ok(message) => {
            received_input = true;
            received_message = message;
        }
        Err(_error) => {
            received_input = false;
        }
    }

    if received_input {

        if received_message == InputEvent::Quit {
            should_break = true;
        }
        else if input_value == 0xFF {

            match received_message {
                InputEvent::RightPressed => { input_value = 0xFE },
                InputEvent::LeftPressed => { input_value = 0xFD },
                InputEvent::UpPressed => { input_value = 0xFB },
                InputEvent::DownPressed => { input_value = 0xF7 },
                InputEvent::APressed => { input_value = 0xFE },
                InputEvent::BPressed => { input_value = 0xFD },
                InputEvent::SelectPressed => { input_value = 0xFB },
                InputEvent::StartPressed => { input_value = 0xF7 },
                _ => {}
            }
        }
        else if input_value == 0xEF {

            match received_message {
                InputEvent::RightPressed => { input_value = 0xEE },
                InputEvent::LeftPressed => { input_value = 0xED },
                InputEvent::UpPressed => { input_value = 0xEB },
                InputEvent::DownPressed => { input_value = 0xE7 },
                _ => {}
            }

        }
        else if input_value == 0xDF {

            match received_message {
                InputEvent::APressed => { input_value = 0xDE },
                InputEvent::BPressed => { input_value = 0xDD },
                InputEvent::SelectPressed => { input_value = 0xDB },
                InputEvent::StartPressed => { input_value = 0xD7 },
                _ => {}
            }
        }

        memory::cpu_write(0xFF00, input_value, cpu_mem, shared_mem);
        let current_if = memory::cpu_read(0xFF0F, cpu_mem, shared_mem);
        memory::cpu_write(0xFF0F, utils::set_bit(current_if, 4), cpu_mem, shared_mem);
    }
    else {
        memory::cpu_write(0xFF00, input_value, cpu_mem, shared_mem);
    }

    should_break
}

fn handle_interrupts(current_state: &mut CpuState, cpu_mem: &mut CpuMemory, shared_mem: &(Arc<Mutex<IoRegisters>>, Arc<Mutex<GpuMemory>>)) {

    let ie_value = memory::cpu_read(0xFFFF, cpu_mem, shared_mem);
    update_interrupts(ie_value, &mut current_state.interrupts);
    let mut if_value = memory::cpu_read(0xFF0F, cpu_mem, shared_mem);

    let vblank_interrupt = utils::check_bit(if_value, 0) && current_state.interrupts.vblank_enabled;
    let lcdc_interrupt = utils::check_bit(if_value, 1) && current_state.interrupts.lcdc_enabled;
    let timer_interrupt = utils::check_bit(if_value, 2) && current_state.interrupts.timer_enabled;
    let serial_interrupt = utils::check_bit(if_value, 3) && current_state.interrupts.serial_enabled;
    let input_interrupt = utils::check_bit(if_value, 4) && current_state.interrupts.input_enabled;

    if vblank_interrupt {

        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit(if_value, 0);
            memory::cpu_write(0xFF0F, if_value, cpu_mem, shared_mem);
            stack_write(&mut current_state.sp, current_state.pc.get(), cpu_mem, shared_mem);
            current_state.pc.set(0x0040);
            current_state.interrupts.can_interrupt = false;
        }
        current_state.halted = false;
    }
    else if lcdc_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit(if_value, 1);
            memory::cpu_write(0xFF0F, if_value, cpu_mem, shared_mem);
            stack_write(&mut current_state.sp, current_state.pc.get(), cpu_mem, shared_mem);
            current_state.pc.set(0x0048);
            current_state.interrupts.can_interrupt = false;
        }
        current_state.halted = false;
    }
    else if timer_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit(if_value, 2);
            memory::cpu_write(0xFF0F, if_value, cpu_mem, shared_mem);
            stack_write(&mut current_state.sp, current_state.pc.get(), cpu_mem, shared_mem);
            current_state.pc.set(0x0050);
            current_state.interrupts.can_interrupt = false;
        }
        current_state.halted = false;
    }
    else if serial_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit(if_value, 3);
            memory::cpu_write(0xFF0F, if_value, cpu_mem, shared_mem);
            stack_write(&mut current_state.sp, current_state.pc.get(), cpu_mem, shared_mem);
            current_state.pc.set(0x0058);
            current_state.interrupts.can_interrupt = false;
        }
        current_state.halted = false;
    }
    else if input_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit(if_value, 4);
            memory::cpu_write(0xFF0F, if_value, cpu_mem, shared_mem);
            stack_write(&mut current_state.sp, current_state.pc.get(), cpu_mem, shared_mem);
            current_state.pc.set(0x0060);
            current_state.interrupts.can_interrupt = false;
        }
        current_state.halted = false;
    }
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

pub fn read_immediate(address: u16, cpu_mem: &mut CpuMemory, shared_mem: &(Arc<Mutex<IoRegisters>>, Arc<Mutex<GpuMemory>>)) -> u8 {

    memory::cpu_read(address + 1, cpu_mem, shared_mem)
}

pub fn read_u16(addr: u16, cpu_mem: &mut CpuMemory, shared_mem: &(Arc<Mutex<IoRegisters>>, Arc<Mutex<GpuMemory>>)) -> u16 {

    let mut bytes: Vec<u8> = vec![0; 2];
    let read_value: u16;
    
    bytes[0] = memory::cpu_read(addr, cpu_mem, shared_mem);
    bytes[1] = memory::cpu_read(addr + 1, cpu_mem, shared_mem);

    read_value = LittleEndian::read_u16(&bytes);
    read_value
}

pub fn stack_read(sp: &mut CpuReg, cpu_mem: &mut CpuMemory, shared_mem: &(Arc<Mutex<IoRegisters>>, Arc<Mutex<GpuMemory>>)) -> u16 {

    let final_value: u16;
    let mut values: Vec<u8> = vec![0; 2];
    
    values[0] = memory::cpu_read(sp.get_register(), cpu_mem, shared_mem);
    sp.increment();
    values[1] = memory::cpu_read(sp.get_register(), cpu_mem, shared_mem);
    sp.increment();

    final_value = LittleEndian::read_u16(&values);
    final_value
}

pub fn stack_write(sp: &mut CpuReg, value: u16, cpu_mem: &mut CpuMemory, shared_mem: &(Arc<Mutex<IoRegisters>>, Arc<Mutex<GpuMemory>>)) {

    sp.decrement();
    memory::cpu_write(sp.get_register(), utils::get_lb(value), cpu_mem, shared_mem);
    sp.decrement();
    memory::cpu_write(sp.get_register(), utils::get_rb(value), cpu_mem, shared_mem);
}