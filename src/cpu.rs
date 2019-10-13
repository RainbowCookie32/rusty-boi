use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

use log::{info, error};
use byteorder::{ByteOrder, LittleEndian};

use super::memory;
use super::memory::{RomMemory, CpuMemory, GpuMemory};

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

pub fn cpu_loop(cycles: Arc<Mutex<u16>>, input: Receiver<InputEvent>, memory: (Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let mut current_state = init_cpu();
    let mut timer_state = timer::init_timer();

    loop {

        if current_state.cycles.get() > 200 {current_state.cycles.set(0)}
        
        handle_interrupts(&mut current_state, &memory);
        let mut opcode = memory::cpu_read(current_state.pc.get(), &memory);

        if !current_state.halted {
            
            if current_state.pc.get() == 0x0100 {
                let mut mem = memory.0.lock().unwrap();
                info!("CPU: Bootrom execution finished, starting loaded ROM.");
                mem.bootrom_finished = true;
            }

            if current_state.pc.get() == 0xC642 {
                //info!("CPU: Test finished.");
            }
        
            if opcode == 0xCB {
                opcode = read_immediate(current_state.pc.get(), &memory);
                current_state.last_result = opcodes_prefixed::run_opcode(&mut current_state, opcode, &memory);
            }
            else {
                current_state.last_result = opcodes::run_opcode(&mut current_state, opcode, &memory);
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
                memory::cpu_write(0xFF40, 0, &memory);
            }
        }

        if current_state.last_result == CycleResult::InvalidOp || current_state.last_result == CycleResult::NopFlood {
            error!("CPU: Breaking execution, last state was {:#?}", current_state.last_result);
            break;
        }

        let mut cyc_mut = cycles.lock().unwrap();
        *cyc_mut = cyc_mut.overflowing_add(current_state.cycles.get()).0;
        timer::timer_cycle(&mut timer_state, current_state.cycles.get(), &memory.1);
        if update_inputs(&input, &memory) {break}
    }
}

fn update_inputs(input_rx: &Receiver<InputEvent>, memory: &(Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) -> bool {

    let received_input: bool;
    let input_event = input_rx.try_recv();

    let mut should_break = false;
    let mut received_message = InputEvent::APressed;
    let mut input_value = memory::cpu_read(0xFF00, &memory);

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

        // Not fully sure if it should also trigger an interrupt on release.
        let mut should_interrupt = false;

        if received_message == InputEvent::Quit {
            should_break = true;
        }
        else if input_value & 0x20 == 0x20 {

            info!("Input: Pressed button at P15 row");
            match received_message {
                InputEvent::RightPressed => { input_value = utils::set_bit(input_value, 0); should_interrupt = true; },
                InputEvent::RightReleased => { input_value = utils::reset_bit(input_value, 0); },
                InputEvent::LeftPressed => { input_value = utils::set_bit(input_value, 1); should_interrupt = true; },
                InputEvent::LeftReleased => { input_value = utils::reset_bit(input_value, 1); },
                InputEvent::UpPressed => { input_value = utils::set_bit(input_value, 2); should_interrupt = true; },
                InputEvent::UpReleased => { input_value = utils::reset_bit(input_value, 2); },
                InputEvent::DownPressed => { input_value = utils::set_bit(input_value, 3); should_interrupt = true; },
                InputEvent::DownReleased => { input_value = utils::reset_bit(input_value, 3); },
                _ => {}
            }

        }
        else if input_value & 0x10 == 0x10 {

            info!("Input: Pressed button at P14 row");
            match received_message {
                InputEvent::APressed => { input_value = utils::set_bit(input_value, 0); should_interrupt = true; },
                InputEvent::AReleased => { input_value = utils::reset_bit(input_value, 0) },
                InputEvent::BPressed => { input_value = utils::set_bit(input_value, 1); should_interrupt = true; },
                InputEvent::BReleased => { input_value = utils::reset_bit(input_value, 1) },
                InputEvent::SelectPressed => { input_value = utils::set_bit(input_value, 2); should_interrupt = true; },
                InputEvent::SelectReleased => { input_value = utils::reset_bit(input_value, 2) },
                InputEvent::StartPressed => { input_value = utils::set_bit(input_value, 3); should_interrupt = true; },
                InputEvent::StartReleased => { input_value = utils::reset_bit(input_value, 3) },
                _ => {}
            }
        }

        memory::cpu_write(0xFF00, input_value, memory);
        if should_interrupt {
            let current_if = memory::cpu_read(0xFF0F, memory);
            memory::cpu_write(0xFF0F, utils::set_bit(current_if, 4), memory);
        }
    }
    else {
        memory::cpu_write(0xFF00, input_value | 0xF, memory);
    }

    should_break
}

fn handle_interrupts(current_state: &mut CpuState, memory: &(Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let ie_value = memory::cpu_read(0xFFFF, memory);
    update_interrupts(ie_value, &mut current_state.interrupts);
    let mut if_value = memory::cpu_read(0xFF0F, memory);

    let vblank_interrupt = utils::check_bit(if_value, 0) && current_state.interrupts.vblank_enabled;
    let lcdc_interrupt = utils::check_bit(if_value, 1) && current_state.interrupts.lcdc_enabled;
    let timer_interrupt = utils::check_bit(if_value, 2) && current_state.interrupts.timer_enabled;
    let serial_interrupt = utils::check_bit(if_value, 3) && current_state.interrupts.serial_enabled;
    let input_interrupt = utils::check_bit(if_value, 4) && current_state.interrupts.input_enabled;

    if vblank_interrupt {

        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit(if_value, 0);
            memory::cpu_write(0xFF0F, if_value, memory);
            stack_write(&mut current_state.sp, current_state.pc.get(), memory);
            current_state.pc.set(0x0040);
        }
        current_state.halted = false;
    }
    else if lcdc_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit(if_value, 1);
            memory::cpu_write(0xFF0F, if_value, memory);
            stack_write(&mut current_state.sp, current_state.pc.get(), memory);
            current_state.pc.set(0x0048);
        }
        current_state.halted = false;
    }
    else if timer_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit(if_value, 2);
            memory::cpu_write(0xFF0F, if_value, memory);
            stack_write(&mut current_state.sp, current_state.pc.get(), memory);
            current_state.pc.set(0x0050);
        }
        current_state.halted = false;
    }
    else if serial_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit(if_value, 3);
            memory::cpu_write(0xFF0F, if_value, memory);
            stack_write(&mut current_state.sp, current_state.pc.get(), memory);
            current_state.pc.set(0x0058);
        }
        current_state.halted = false;
    }
    else if input_interrupt {
        
        if current_state.interrupts.can_interrupt {
            if_value = utils::reset_bit(if_value, 4);
            memory::cpu_write(0xFF0F, if_value, memory);
            stack_write(&mut current_state.sp, current_state.pc.get(), memory);
            current_state.pc.set(0x0060);
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

pub fn read_immediate(address: u16, memory: &(Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) -> u8 {

    memory::cpu_read(address + 1, memory)
}

pub fn read_u16(addr: u16, memory: &(Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) -> u16 {

    let mut bytes: Vec<u8> = vec![0; 2];
    let read_value: u16;
    
    bytes[0] = memory::cpu_read(addr, memory);
    bytes[1] = memory::cpu_read(addr + 1, memory);

    read_value = LittleEndian::read_u16(&bytes);
    read_value
}

pub fn stack_read(sp: &mut CpuReg, memory: &(Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) -> u16 {

    let final_value: u16;
    let mut values: Vec<u8> = vec![0; 2];
    
    values[0] = memory::cpu_read(sp.get_register(), memory);
    sp.increment();
    values[1] = memory::cpu_read(sp.get_register(), memory);
    sp.increment();

    final_value = LittleEndian::read_u16(&values);
    final_value
}

pub fn stack_write(sp: &mut CpuReg, value: u16, memory: &(Arc<Mutex<RomMemory>>, Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    sp.decrement();
    memory::cpu_write(sp.get_register(), utils::get_lb(value), memory);
    sp.decrement();
    memory::cpu_write(sp.get_register(), utils::get_rb(value), memory);
}