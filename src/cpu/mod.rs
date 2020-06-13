use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;

use log::{info};
use byteorder::{ByteOrder, LittleEndian};

use super::InputEvent;
use super::timer::Timer;
use super::memory::EmulatedMemory;

const Z_FLAG: u8 = 7;
const N_FLAG: u8 = 6;
const H_FLAG: u8 = 5;
const C_FLAG: u8 = 4;

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    NotReady,
    Running { paused: bool, breakpoint: bool, step: bool, error: bool },
}

pub enum Condition {
    ZSet,
    ZNotSet,
    CSet,
    CNotSet,
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

impl InterruptState {
    pub fn default() -> InterruptState {
        InterruptState {
            can_interrupt: false, 
            vblank_enabled: false,
            lcdc_enabled: false,
            timer_enabled: false,
            serial_enabled: false,
            input_enabled: false,
        }
    }
}

#[derive(Clone)]
pub struct CpuRegister {
    value: u16
}

impl CpuRegister {
    pub fn new() -> CpuRegister {
        CpuRegister {
            value: 0,
        }
    }

    pub fn get(&self) -> u16 {
        self.value
    }

    pub fn get_hi(&self) -> u8 {
        (self.value >> 8) as u8
    }

    pub fn get_low(&self) -> u8 {
        (self.value & 0xFF) as u8
    }

    pub fn set(&mut self, value: u16) {
        self.value = value
    }

    pub fn set_hi(&mut self, value: u8) {
        self.value = (self.value & 0x00FF) | (value as u16) << 8;
    }

    pub fn set_low(&mut self, value: u8) {
        self.value = (self.value & 0xFF00) | value as u16;
    }
}

pub struct Cpu {
    // In order: AF, BC, DE, HL, SP
    registers: Vec<CpuRegister>,

    pc: u16,

    pub halted: bool,
    stopped: bool,
    
    pub ui: Arc<Mutex<UiObject>>,
    pub cpu_status: Status,

    input_rx: mpsc::Receiver<InputEvent>,

    timer: Timer,
    memory: Arc<EmulatedMemory>,
    interrupts: InterruptState,
}

impl Cpu {
    pub fn new(ui: Arc<Mutex<UiObject>>, memory: Arc<EmulatedMemory>, rx: mpsc::Receiver<InputEvent>) -> Cpu {
        let timer = Timer::new(memory.clone());

        Cpu {
            registers: vec![CpuRegister::new(); 5],

            pc: 0,

            halted: false,
            stopped: false,
            
            ui: ui,
            cpu_status: Status::NotReady,

            input_rx: rx,

            timer: timer,
            memory: memory,
            interrupts: InterruptState::default(),
        }
    }

    pub fn step(&mut self) {
        self.update_input();
        self.handle_interrupts();

        if !self.halted {
            if self.pc == 0x100 && self.memory.get_bootrom_state() {
                self.memory.disable_bootrom();
                info!("CPU: Bootrom finished, running loaded ROM.");
            }

            self.run_instruction();
        }
        else {
            self.instruction_finished(0, 4);
        }

        self.timer.step(super::GLOBAL_CYCLE_COUNTER.load(Ordering::Relaxed));
    }

    pub fn update_ui_object(&mut self) {
        let lock = self.ui.lock();

        if lock.is_err() {
            return;
        }

        let mut lock = lock.unwrap();

        lock.registers[0] = self.registers[0].get();
        lock.registers[1] = self.registers[1].get();
        lock.registers[2] = self.registers[2].get();
        lock.registers[3] = self.registers[3].get();
        lock.registers[4] = self.registers[4].get();

        lock.pc = self.pc;
        lock.opcode = self.memory.read(self.pc);

        lock.halted = self.halted;
        lock.cpu_status = self.cpu_status;

        if let Status::Running{paused, breakpoint, step, error} = self.cpu_status {
            self.cpu_status = Status::Running {
                paused: if lock.cpu_paused.is_some() { lock.cpu_paused.clone().unwrap() } else { paused },
                breakpoint: breakpoint,
                step: if lock.cpu_step.is_some() { lock.cpu_step.clone().unwrap() } else { false },
                error: error
            };

            lock.cpu_paused = None;
            lock.cpu_step = None;

            for address in &lock.breakpoints {
                if *address == self.pc {
                    self.cpu_status = Status::Running{ paused: true, breakpoint: true, step: step, error: error };
                    break;
                }
            }
        }
        else {
            if lock.cpu_paused.is_some() || lock.cpu_step.is_some() {
                self.cpu_status = Status::Running {
                    paused: if lock.cpu_paused.is_some() { lock.cpu_paused.clone().unwrap() } else { true },
                    breakpoint: false,
                    step: if lock.cpu_step.is_some() { lock.cpu_step.clone().unwrap() } else { false },
                    error: false
                };
    
                lock.cpu_paused = None;
                lock.cpu_step = None;
            }
        }
    }

    fn handle_interrupts(&mut self) {
        self.update_interrupts();

        let mut if_value = self.memory.read(0xFF0F);

        let vblank_requested = (if_value & 1) != 0;
        let lcdc_requested = (if_value & (1 << 1)) != 0;
        let timer_requested = (if_value & (1 << 2)) != 0;
        let serial_requested = (if_value & (1 << 3)) != 0;
        let input_requested = (if_value & (1 << 4)) != 0;

        if vblank_requested && self.interrupts.vblank_enabled {
            if self.interrupts.can_interrupt {
                if_value &= !1;
                self.stack_write(self.pc);
                self.pc = 0x0040;
                self.interrupts.can_interrupt = false;
            }
            
            self.halted = false;
        }
        else if lcdc_requested && self.interrupts.lcdc_enabled {
            if self.interrupts.can_interrupt {
                if_value &= !(1 << 1);
                self.stack_write(self.pc);
                self.pc = 0x0048;
                self.interrupts.can_interrupt = false;
            }
            
            self.halted = false;
        }
        else if timer_requested && self.interrupts.timer_enabled {
            if self.interrupts.can_interrupt {
                if_value &= !(1 << 2);
                self.stack_write(self.pc);
                self.pc = 0x0050;
                self.interrupts.can_interrupt = false;
            }
            
            self.halted = false;
        }
        else if serial_requested && self.interrupts.serial_enabled {
            if self.interrupts.can_interrupt {
                if_value &= !(1 << 3);
                self.stack_write(self.pc);
                self.pc = 0x0058;
                self.interrupts.can_interrupt = false;
            }
            
            self.halted = false;
        }
        else if input_requested && self.interrupts.input_enabled {
            if self.interrupts.can_interrupt {
                if_value &= !(1 << 4);
                self.stack_write(self.pc);
                self.pc = 0x0060;
                self.interrupts.can_interrupt = false;
            }
            
            self.halted = false;
        }

        self.memory.write(0xFF0F, if_value, true);
    }

    fn update_interrupts(&mut self) {
        let ie_value = self.memory.read(0xFFFF);

        self.interrupts.vblank_enabled = (ie_value & 1) != 0;
        self.interrupts.lcdc_enabled = (ie_value & (1 << 1)) != 0;
        self.interrupts.timer_enabled = (ie_value & (1 << 2)) != 0;
        self.interrupts.serial_enabled = (ie_value & (1 << 3)) != 0;
        self.interrupts.input_enabled = (ie_value & (1 << 4)) != 0;
    }

    fn update_input(&mut self) {
        let input_reg = self.memory.read(0xFF00);
        if (input_reg & 0x30) == 0 {
            return;
        }

        let mut result = 0b1111;
        let mut input_received = false;

        let dpad = (input_reg & 0x10) == 0;
        let buttons = (input_reg & 0x20) == 0;

        let received_events = self.input_rx.try_iter();
        let if_value = self.memory.read(0xFF0F);

        for event in received_events {
            input_received = true;
            match event {
                // Direction keys.
                InputEvent::Right => {
                    if dpad {
                        result &= 0x0E;
                    }
                },
                InputEvent::Left => {
                    if dpad {
                        result &= 0x0D;
                    }
                },
                InputEvent::Up => {
                    if dpad {
                        result &= 0x0B;
                    }
                },
                InputEvent::Down => {
                    if dpad {
                        result &= 0x07;
                    }
                },

                // Button keys.
                InputEvent::A => {
                    if buttons {
                        result &= 0x0E;
                    }
                },
                InputEvent::B => {
                    if buttons {
                        result &= 0x0D;
                    }
                },
                InputEvent::Select => {
                    if buttons {
                        result &= 0x0B;
                    }
                },
                InputEvent::Start => {
                    if buttons {
                        result &= 0x07;
                    }
                }
            }
        }

        self.memory.write(0xFF00, result | 0xC0, true);
        if input_received {
            self.memory.write(0xFF0F, if_value | (1 << 4), true);
        }
    }

    fn run_instruction(&mut self) {
        let opcode = self.memory.read(self.pc);

        if opcode == 0xCB {
            self.run_instruction_prefixed();
        }
        else {
            match opcode {
                0x00 => self.nop(),
                0x01 => self.load_immediate_to_full(1),
                0x02 => self.save_a_to_full(1),
                0x03 => self.increment_full(1),
                0x04 => self.increment_hi(1),
                0x05 => self.decrement_hi(1),
                0x06 => self.load_immediate_to_hi(1),
                0x07 => self.rlca(),
                0x08 => self.save_sp_to_immediate(),
                0x09 => self.add_full_to_hl(1),
                0x0A => self.load_a_from_full(1),
                0x0B => self.decrement_full(1),
                0x0C => self.increment_low(1),
                0x0D => self.decrement_low(1),
                0x0E => self.load_immediate_to_low(1),
                0x0F => self.rrca(),

                0x10 => self.stop(),
                0x11 => self.load_immediate_to_full(2),
                0x12 => self.save_a_to_full(2),
                0x13 => self.increment_full(2),
                0x14 => self.increment_hi(2),
                0x15 => self.decrement_hi(2),
                0x16 => self.load_immediate_to_hi(2),
                0x17 => self.rla(),
                0x18 => self.jump_relative(),
                0x19 => self.add_full_to_hl(2),
                0x1A => self.load_a_from_full(2),
                0x1B => self.decrement_full(2),
                0x1C => self.increment_low(2),
                0x1D => self.decrement_low(2),
                0x1E => self.load_immediate_to_low(2),
                0x1F => self.rra(),

                0x20 => self.jump_relative_conditional(Condition::ZNotSet),
                0x21 => self.load_immediate_to_full(3),
                0x22 => self.save_a_to_hl_inc(),
                0x23 => self.increment_full(3),
                0x24 => self.increment_hi(3),
                0x25 => self.decrement_hi(3),
                0x26 => self.load_immediate_to_hi(3),
                0x27 => self.daa(),
                0x28 => self.jump_relative_conditional(Condition::ZSet),
                0x29 => self.add_full_to_hl(3),
                0x2A => self.load_a_from_hl_inc(),
                0x2B => self.decrement_full(3),
                0x2C => self.increment_low(3),
                0x2D => self.decrement_low(3),
                0x2E => self.load_immediate_to_low(3),
                0x2F => self.cpl(),

                0x30 => self.jump_relative_conditional(Condition::CNotSet),
                0x31 => self.load_immediate_to_full(4),
                0x32 => self.save_a_to_hl_dec(),
                0x33 => self.increment_full(4),
                0x34 => self.increment_at_hl(),
                0x35 => self.decrement_at_hl(),
                0x36 => self.save_immediate_to_hl(),
                0x37 => self.scf(),
                0x38 => self.jump_relative_conditional(Condition::CSet),
                0x39 => self.add_full_to_hl(4),
                0x3A => self.load_a_from_hl_dec(),
                0x3B => self.decrement_full(4),
                0x3C => self.increment_hi(0),
                0x3D => self.decrement_hi(0),
                0x3E => self.load_immediate_to_hi(0),
                0x3F => self.ccf(),

                0x40 => self.load_hi_to_hi(1, 1),
                0x41 => self.load_low_to_hi(1, 1),
                0x42 => self.load_hi_to_hi(1, 2),
                0x43 => self.load_low_to_hi(1, 2),
                0x44 => self.load_hi_to_hi(1, 3),
                0x45 => self.load_low_to_hi(1, 3),
                0x46 => self.load_hl_to_hi(1),
                0x47 => self.load_hi_to_hi(1, 0),
                0x48 => self.load_hi_to_low(1, 1),
                0x49 => self.load_low_to_low(1, 1),
                0x4A => self.load_hi_to_low(1, 2),
                0x4B => self.load_low_to_low(1, 2),
                0x4C => self.load_hi_to_low(1, 3),
                0x4D => self.load_low_to_low(1, 3),
                0x4E => self.load_hl_to_low(1),
                0x4F => self.load_hi_to_low(1, 0),

                0x50 => self.load_hi_to_hi(2, 1),
                0x51 => self.load_low_to_hi(2, 1),
                0x52 => self.load_hi_to_hi(2, 2),
                0x53 => self.load_low_to_hi(2, 2),
                0x54 => self.load_hi_to_hi(2, 3),
                0x55 => self.load_low_to_hi(2, 3),
                0x56 => self.load_hl_to_hi(2),
                0x57 => self.load_hi_to_hi(2, 0),
                0x58 => self.load_hi_to_low(2, 1),
                0x59 => self.load_low_to_low(2, 1),
                0x5A => self.load_hi_to_low(2, 2),
                0x5B => self.load_low_to_low(2, 2),
                0x5C => self.load_hi_to_low(2, 3),
                0x5D => self.load_low_to_low(2, 3),
                0x5E => self.load_hl_to_low(2),
                0x5F => self.load_hi_to_low(2, 0),

                0x60 => self.load_hi_to_hi(3, 1),
                0x61 => self.load_low_to_hi(3, 1),
                0x62 => self.load_hi_to_hi(3, 2),
                0x63 => self.load_low_to_hi(3, 2),
                0x64 => self.load_hi_to_hi(3, 3),
                0x65 => self.load_low_to_hi(3, 3),
                0x66 => self.load_hl_to_hi(3),
                0x67 => self.load_hi_to_hi(3, 0),
                0x68 => self.load_hi_to_low(3, 1),
                0x69 => self.load_low_to_low(3, 1),
                0x6A => self.load_hi_to_low(3, 2),
                0x6B => self.load_low_to_low(3, 2),
                0x6C => self.load_hi_to_low(3, 3),
                0x6D => self.load_low_to_low(3, 3),
                0x6E => self.load_hl_to_low(3),
                0x6F => self.load_hi_to_low(3, 0),

                0x70 => self.load_hi_to_hl(1),
                0x71 => self.load_low_to_hl(1),
                0x72 => self.load_hi_to_hl(2),
                0x73 => self.load_low_to_hl(2),
                0x74 => self.load_hi_to_hl(3),
                0x75 => self.load_low_to_hl(3),
                0x76 => self.halt(),
                0x77 => self.load_hi_to_hl(0),
                0x78 => self.load_hi_to_hi(0, 1),
                0x79 => self.load_low_to_hi(0, 1),
                0x7A => self.load_hi_to_hi(0, 2),
                0x7B => self.load_low_to_hi(0, 2),
                0x7C => self.load_hi_to_hi(0, 3),
                0x7D => self.load_low_to_hi(0, 3),
                0x7E => self.load_hl_to_hi(0),
                0x7F => self.load_hi_to_hi(0, 0),

                0x80 => self.add_hi(1),
                0x81 => self.add_low(1),
                0x82 => self.add_hi(2),
                0x83 => self.add_low(2),
                0x84 => self.add_hi(3),
                0x85 => self.add_low(3),
                0x86 => self.add_hl(),
                0x87 => self.add_hi(0),
                0x88 => self.adc_hi(1),
                0x89 => self.adc_low(1),
                0x8A => self.adc_hi(2),
                0x8B => self.adc_low(2),
                0x8C => self.adc_hi(3),
                0x8D => self.adc_low(3),
                0x8E => self.adc_hl(),
                0x8F => self.adc_hi(0),

                0x90 => self.sub_hi(1),
                0x91 => self.sub_low(1),
                0x92 => self.sub_hi(2),
                0x93 => self.sub_low(2),
                0x94 => self.sub_hi(3),
                0x95 => self.sub_low(3),
                0x96 => self.sub_hl(),
                0x97 => self.sub_hi(0),
                0x98 => self.sbc_hi(1),
                0x99 => self.sbc_low(1),
                0x9A => self.sbc_hi(2),
                0x9B => self.sbc_low(2),
                0x9C => self.sbc_hi(3),
                0x9D => self.sbc_low(3),
                0x9E => self.sbc_hl(),
                0x9F => self.sbc_hi(0),

                0xA0 => self.and_hi(1),
                0xA1 => self.and_low(1),
                0xA2 => self.and_hi(2),
                0xA3 => self.and_low(2),
                0xA4 => self.and_hi(3),
                0xA5 => self.and_low(3),
                0xA6 => self.and_hl(),
                0xA7 => self.and_hi(0),
                0xA8 => self.xor_hi(1),
                0xA9 => self.xor_low(1),
                0xAA => self.xor_hi(2),
                0xAB => self.xor_low(2),
                0xAC => self.xor_hi(3),
                0xAD => self.xor_low(3),
                0xAE => self.xor_hl(),
                0xAF => self.xor_hi(0),

                0xB0 => self.or_hi(1),
                0xB1 => self.or_low(1),
                0xB2 => self.or_hi(2),
                0xB3 => self.or_low(2),
                0xB4 => self.or_hi(3),
                0xB5 => self.or_low(3),
                0xB6 => self.or_hl(),
                0xB7 => self.or_hi(0),
                0xB8 => self.cp_hi(1),
                0xB9 => self.cp_low(1),
                0xBA => self.cp_hi(2),
                0xBB => self.cp_low(2),
                0xBC => self.cp_hi(3),
                0xBD => self.cp_low(3),
                0xBE => self.cp_hl(),
                0xBF => self.cp_hi(0),

                0xC0 => self.return_conditional(Condition::ZNotSet),
                0xC1 => self.pop_register(1),
                0xC2 => self.jump_conditional(Condition::ZNotSet),
                0xC3 => self.jump(),
                0xC4 => self.call_conditional(Condition::ZNotSet),
                0xC5 => self.push_register(1),
                0xC6 => self.add_immediate(),
                0xC7 => self.rst(0),
                0xC8 => self.return_conditional(Condition::ZSet),
                0xC9 => self.ret(),
                0xCA => self.jump_conditional(Condition::ZSet),
                0xCB => self.invalid_opcode(opcode),
                0xCC => self.call_conditional(Condition::ZSet),
                0xCD => self.call(),
                0xCE => self.adc_immediate(),
                0xCF => self.rst(0x0008),

                0xD0 => self.return_conditional(Condition::CNotSet),
                0xD1 => self.pop_register(2),
                0xD2 => self.jump_conditional(Condition::CNotSet),
                0xD3 => self.invalid_opcode(opcode),
                0xD4 => self.call_conditional(Condition::CNotSet),
                0xD5 => self.push_register(2),
                0xD6 => self.sub_immediate(),
                0xD7 => self.rst(0x0010),
                0xD8 => self.return_conditional(Condition::CSet),
                0xD9 => self.reti(),
                0xDA => self.jump_conditional(Condition::CSet),
                0xDB => self.invalid_opcode(opcode),
                0xDC => self.call_conditional(Condition::CSet),
                0xDD => self.invalid_opcode(opcode),
                0xDE => self.sbc_immediate(),
                0xDF => self.rst(0x0018),

                0xE0 => self.save_a_to_ff_immediate(),
                0xE1 => self.pop_register(3),
                0xE2 => self.save_a_to_ff_c(),
                0xE3 => self.invalid_opcode(opcode),
                0xE4 => self.invalid_opcode(opcode),
                0xE5 => self.push_register(3),
                0xE6 => self.and_immediate(),
                0xE7 => self.rst(0x0020),
                0xE8 => self.add_signed_immediate_to_sp(),
                0xE9 => self.jump_hl(),
                0xEA => self.save_a_to_immediate(),
                0xEB => self.invalid_opcode(opcode),
                0xEC => self.invalid_opcode(opcode),
                0xED => self.invalid_opcode(opcode),
                0xEE => self.xor_immediate(),
                0xEF => self.rst(0x0028),

                0xF0 => self.load_a_from_ff_immediate(),
                0xF1 => self.pop_register(0),
                0xF2 => self.load_a_from_ff_c(),
                0xF3 => self.di(),
                0xF4 => self.invalid_opcode(opcode),
                0xF5 => self.push_register(0),
                0xF6 => self.or_immediate(),
                0xF7 => self.rst(0x0030),
                0xF8 => self.load_sp_plus_signed_to_hl(),
                0xF9 => self.load_hl_to_sp(),
                0xFA => self.load_a_from_immediate(),
                0xFB => self.ei(),
                0xFC => self.invalid_opcode(opcode),
                0xFD => self.invalid_opcode(opcode),
                0xFE => self.cp_immediate(),
                0xFF => self.rst(0x0038),
            }
        }
    } 

    fn run_instruction_prefixed(&mut self) {
        let opcode = self.memory.read(self.pc + 1);

        match opcode {
            0x00 => self.rlc_hi(1),
            0x01 => self.rlc_low(1),
            0x02 => self.rlc_hi(2),
            0x03 => self.rlc_low(2),
            0x04 => self.rlc_hi(3),
            0x05 => self.rlc_low(3),
            0x06 => self.rlc_hl(),
            0x07 => self.rlc_hi(0),
            0x08 => self.rrc_hi(1),
            0x09 => self.rrc_low(1),
            0x0A => self.rrc_hi(2),
            0x0B => self.rrc_low(2),
            0x0C => self.rrc_hi(3),
            0x0D => self.rrc_low(3),
            0x0E => self.rrc_hl(),
            0x0F => self.rrc_hi(0),

            0x10 => self.rl_hi(1),
            0x11 => self.rl_low(1),
            0x12 => self.rl_hi(2),
            0x13 => self.rl_low(2),
            0x14 => self.rl_hi(3),
            0x15 => self.rl_low(3),
            0x16 => self.rl_hl(),
            0x17 => self.rl_hi(0),
            0x18 => self.rr_hi(1),
            0x19 => self.rr_low(1),
            0x1A => self.rr_hi(2),
            0x1B => self.rr_low(2),
            0x1C => self.rr_hi(3),
            0x1D => self.rr_low(3),
            0x1E => self.rr_hl(),
            0x1F => self.rr_hi(0),

            0x20 => self.sla_hi(1),
            0x21 => self.sla_low(1),
            0x22 => self.sla_hi(2),
            0x23 => self.sla_low(2),
            0x24 => self.sla_hi(3),
            0x25 => self.sla_low(3),
            0x26 => self.sla_hl(),
            0x27 => self.sla_hi(0),
            0x28 => self.sra_hi(1),
            0x29 => self.sra_low(1),
            0x2A => self.sra_hi(2),
            0x2B => self.sra_low(2),
            0x2C => self.sra_hi(3),
            0x2D => self.sra_low(3),
            0x2E => self.sra_hl(),
            0x2F => self.sra_hi(0),

            0x30 => self.swap_hi(1),
            0x31 => self.swap_low(1),
            0x32 => self.swap_hi(2),
            0x33 => self.swap_low(2),
            0x34 => self.swap_hi(3),
            0x35 => self.swap_low(3),
            0x36 => self.swap_hl(),
            0x37 => self.swap_hi(0),
            0x38 => self.srl_hi(1),
            0x39 => self.srl_low(1),
            0x3A => self.srl_hi(2),
            0x3B => self.srl_low(2),
            0x3C => self.srl_hi(3),
            0x3D => self.srl_low(3),
            0x3E => self.srl_hl(),
            0x3F => self.srl_hi(0),

            0x40 => self.bit_hi(1, 0),
            0x41 => self.bit_low(1, 0),
            0x42 => self.bit_hi(2, 0),
            0x43 => self.bit_low(2, 0),
            0x44 => self.bit_hi(3, 0),
            0x45 => self.bit_low(3, 0),
            0x46 => self.bit_hl(0),
            0x47 => self.bit_hi(0, 0),
            0x48 => self.bit_hi(1, 1),
            0x49 => self.bit_low(1, 1),
            0x4A => self.bit_hi(2, 1),
            0x4B => self.bit_low(2, 1),
            0x4C => self.bit_hi(3, 1),
            0x4D => self.bit_low(3, 1),
            0x4E => self.bit_hl(1),
            0x4F => self.bit_hi(0, 1),

            0x50 => self.bit_hi(1, 2),
            0x51 => self.bit_low(1, 2),
            0x52 => self.bit_hi(2, 2),
            0x53 => self.bit_low(2, 2),
            0x54 => self.bit_hi(3, 2),
            0x55 => self.bit_low(3, 2),
            0x56 => self.bit_hl(2),
            0x57 => self.bit_hi(0, 2),
            0x58 => self.bit_hi(1, 3),
            0x59 => self.bit_low(1, 3),
            0x5A => self.bit_hi(2, 3),
            0x5B => self.bit_low(2, 3),
            0x5C => self.bit_hi(3, 3),
            0x5D => self.bit_low(3, 3),
            0x5E => self.bit_hl(3),
            0x5F => self.bit_hi(0, 3),

            0x60 => self.bit_hi(1, 4),
            0x61 => self.bit_low(1, 4),
            0x62 => self.bit_hi(2, 4),
            0x63 => self.bit_low(2, 4),
            0x64 => self.bit_hi(3, 4),
            0x65 => self.bit_low(3, 4),
            0x66 => self.bit_hl(4),
            0x67 => self.bit_hi(0, 4),
            0x68 => self.bit_hi(1, 5),
            0x69 => self.bit_low(1, 5),
            0x6A => self.bit_hi(2, 5),
            0x6B => self.bit_low(2, 5),
            0x6C => self.bit_hi(3, 5),
            0x6D => self.bit_low(3, 5),
            0x6E => self.bit_hl(5),
            0x6F => self.bit_hi(0, 5),

            0x70 => self.bit_hi(1, 6),
            0x71 => self.bit_low(1, 6),
            0x72 => self.bit_hi(2, 6),
            0x73 => self.bit_low(2, 6),
            0x74 => self.bit_hi(3, 6),
            0x75 => self.bit_low(3, 6),
            0x76 => self.bit_hl(6),
            0x77 => self.bit_hi(0, 6),
            0x78 => self.bit_hi(1, 7),
            0x79 => self.bit_low(1, 7),
            0x7A => self.bit_hi(2, 7),
            0x7B => self.bit_low(2, 7),
            0x7C => self.bit_hi(3, 7),
            0x7D => self.bit_low(3, 7),
            0x7E => self.bit_hl(7),
            0x7F => self.bit_hi(0, 7),

            0x80 => self.res_hi(1, 0),
            0x81 => self.res_low(1, 0),
            0x82 => self.res_hi(2, 0),
            0x83 => self.res_low(2, 0),
            0x84 => self.res_hi(3, 0),
            0x85 => self.res_low(3, 0),
            0x86 => self.res_hl(0),
            0x87 => self.res_hi(0, 0),
            0x88 => self.res_hi(1, 1),
            0x89 => self.res_low(1, 1),
            0x8A => self.res_hi(2, 1),
            0x8B => self.res_low(2, 1),
            0x8C => self.res_hi(3, 1),
            0x8D => self.res_low(3, 1),
            0x8E => self.res_hl(1),
            0x8F => self.res_hi(0, 1),

            0x90 => self.res_hi(1, 2),
            0x91 => self.res_low(1, 2),
            0x92 => self.res_hi(2, 2),
            0x93 => self.res_low(2, 2),
            0x94 => self.res_hi(3, 2),
            0x95 => self.res_low(3, 2),
            0x96 => self.res_hl(2),
            0x97 => self.res_hi(0, 2),
            0x98 => self.res_hi(1, 3),
            0x99 => self.res_low(1, 3),
            0x9A => self.res_hi(2, 3),
            0x9B => self.res_low(2, 3),
            0x9C => self.res_hi(3, 3),
            0x9D => self.res_low(3, 3),
            0x9E => self.res_hl(3),
            0x9F => self.res_hi(0, 3),

            0xA0 => self.res_hi(1, 4),
            0xA1 => self.res_low(1, 4),
            0xA2 => self.res_hi(2, 4),
            0xA3 => self.res_low(2, 4),
            0xA4 => self.res_hi(3, 4),
            0xA5 => self.res_low(3, 4),
            0xA6 => self.res_hl(4),
            0xA7 => self.res_hi(0, 4),
            0xA8 => self.res_hi(1, 5),
            0xA9 => self.res_low(1, 5),
            0xAA => self.res_hi(2, 5),
            0xAB => self.res_low(2, 5),
            0xAC => self.res_hi(3, 5),
            0xAD => self.res_low(3, 5),
            0xAE => self.res_hl(5),
            0xAF => self.res_hi(0, 5),

            0xB0 => self.res_hi(1, 6),
            0xB1 => self.res_low(1, 6),
            0xB2 => self.res_hi(2, 6),
            0xB3 => self.res_low(2, 6),
            0xB4 => self.res_hi(3, 6),
            0xB5 => self.res_low(3, 6),
            0xB6 => self.res_hl(6),
            0xB7 => self.res_hi(0, 6),
            0xB8 => self.res_hi(1, 7),
            0xB9 => self.res_low(1, 7),
            0xBA => self.res_hi(2, 7),
            0xBB => self.res_low(2, 7),
            0xBC => self.res_hi(3, 7),
            0xBD => self.res_low(3, 7),
            0xBE => self.res_hl(7),
            0xBF => self.res_hi(0, 7),

            0xC0 => self.set_hi(1, 0),
            0xC1 => self.set_low(1, 0),
            0xC2 => self.set_hi(2, 0),
            0xC3 => self.set_low(2, 0),
            0xC4 => self.set_hi(3, 0),
            0xC5 => self.set_low(3, 0),
            0xC6 => self.set_hl(0),
            0xC7 => self.set_hi(0, 0),
            0xC8 => self.set_hi(1, 1),
            0xC9 => self.set_low(1, 1),
            0xCA => self.set_hi(2, 1),
            0xCB => self.set_low(2, 1),
            0xCC => self.set_hi(3, 1),
            0xCD => self.set_low(3, 1),
            0xCE => self.set_hl(1),
            0xCF => self.set_hi(0, 1),

            0xD0 => self.set_hi(1, 2),
            0xD1 => self.set_low(1, 2),
            0xD2 => self.set_hi(2, 2),
            0xD3 => self.set_low(2, 2),
            0xD4 => self.set_hi(3, 2),
            0xD5 => self.set_low(3, 2),
            0xD6 => self.set_hl(2),
            0xD7 => self.set_hi(0, 2),
            0xD8 => self.set_hi(1, 3),
            0xD9 => self.set_low(1, 3),
            0xDA => self.set_hi(2, 3),
            0xDB => self.set_low(2, 3),
            0xDC => self.set_hi(3, 3),
            0xDD => self.set_low(3, 3),
            0xDE => self.set_hl(3),
            0xDF => self.set_hi(0, 3),

            0xE0 => self.set_hi(1, 4),
            0xE1 => self.set_low(1, 4),
            0xE2 => self.set_hi(2, 4),
            0xE3 => self.set_low(2, 4),
            0xE4 => self.set_hi(3, 4),
            0xE5 => self.set_low(3, 4),
            0xE6 => self.set_hl(4),
            0xE7 => self.set_hi(0, 4),
            0xE8 => self.set_hi(1, 5),
            0xE9 => self.set_low(1, 5),
            0xEA => self.set_hi(2, 5),
            0xEB => self.set_low(2, 5),
            0xEC => self.set_hi(3, 5),
            0xED => self.set_low(3, 5),
            0xEE => self.set_hl(5),
            0xEF => self.set_hi(0, 5),

            0xF0 => self.set_hi(1, 6),
            0xF1 => self.set_low(1, 6),
            0xF2 => self.set_hi(2, 6),
            0xF3 => self.set_low(2, 6),
            0xF4 => self.set_hi(3, 6),
            0xF5 => self.set_low(3, 6),
            0xF6 => self.set_hl(6),
            0xF7 => self.set_hi(0, 6),
            0xF8 => self.set_hi(1, 7),
            0xF9 => self.set_low(1, 7),
            0xFA => self.set_hi(2, 7),
            0xFB => self.set_low(2, 7),
            0xFC => self.set_hi(3, 7),
            0xFD => self.set_low(3, 7),
            0xFE => self.set_hl(7),
            0xFF => self.set_hi(0, 7),
        }
    }

    fn invalid_opcode(&mut self, opcode: u8) {
        self.cpu_status = Status::Running{paused: true, breakpoint: false, step: false, error: true};
        self.ui.lock().unwrap().cpu_status = self.cpu_status;
        log::error!("Tried to execute invalid opcode 0x{:02X}", opcode);
    }

    fn instruction_finished(&mut self, pc: u16, cycles: u16) {
        self.pc += pc;
        super::GLOBAL_CYCLE_COUNTER.fetch_add(cycles, Ordering::Relaxed);
    }

    fn update_flags(&mut self, z: Option<bool>, n: Option<bool>, h: Option<bool>, c: Option<bool>) {
        if z.is_some() {
            let result = Cpu::set_bit(self.registers[0].get_low(), Z_FLAG, z.unwrap());
            self.registers[0].set_low(result);
        }

        if n.is_some() {
            let result = Cpu::set_bit(self.registers[0].get_low(), N_FLAG, n.unwrap());
            self.registers[0].set_low(result);
        }

        if h.is_some() {
            let result = Cpu::set_bit(self.registers[0].get_low(), H_FLAG, h.unwrap());
            self.registers[0].set_low(result);
        }

        if c.is_some() {
            let result = Cpu::set_bit(self.registers[0].get_low(), C_FLAG, c.unwrap());
            self.registers[0].set_low(result);
        }
    }

    fn stack_read(&mut self) -> u16 {
        let mut sp = self.registers[4].get();
        let mut values = vec![0; 2];

        values[0] = self.memory.read(sp);
        sp += 1;
        values[1] = self.memory.read(sp);
        sp += 1;
        
        self.registers[4].set(sp);
        LittleEndian::read_u16(&values)
    }

    fn stack_write(&mut self, value: u16) {
        let mut sp = self.registers[4].get();

        sp -= 1;
        self.memory.write(sp, (value >> 8) as u8, true);
        sp -= 1;
        self.memory.write(sp, value as u8, true);

        self.registers[4].set(sp);
    }

    
    // Regular Instructions.

    
    // Special things.
    fn nop(&mut self) {
        self.instruction_finished(1, 4);
    }

    fn daa(&mut self) {
        //todo!("DAA aka the weird one");
        self.instruction_finished(1, 4);
    }

    fn halt(&mut self) {
        self.halted = true;
        self.instruction_finished(1, 4);
    }

    fn stop(&mut self) {
        self.stopped = true;
        self.instruction_finished(2, 4);
    }

    
    // Jumps.
    fn jump(&mut self) {
        self.pc = LittleEndian::read_u16(&vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)]);
        self.instruction_finished(0, 16);
    }

    fn jump_hl(&mut self) {
        self.pc = self.registers[3].get();
        self.instruction_finished(0, 4);
    }

    fn jump_conditional(&mut self, condition: Condition) {
        let condition_met = match condition {
            Condition::ZNotSet => !Cpu::check_bit(self.registers[0].get_low(), Z_FLAG),
            Condition::ZSet => Cpu::check_bit(self.registers[0].get_low(), Z_FLAG),
            Condition::CNotSet => !Cpu::check_bit(self.registers[0].get_low(), C_FLAG),
            Condition::CSet => Cpu::check_bit(self.registers[0].get_low(), C_FLAG),
        };

        if condition_met {
            self.jump();
        }
        else {
            self.instruction_finished(3, 12);
        }
    }

    fn jump_relative(&mut self) {
        let target = self.memory.read(self.pc + 1) as i8;

        self.pc = self.pc.wrapping_add(target as u16);
        self.instruction_finished(2, 12);
    }

    fn jump_relative_conditional(&mut self, condition: Condition) {
        let condition_met = match condition {
            Condition::ZNotSet => !Cpu::check_bit(self.registers[0].get_low(), Z_FLAG),
            Condition::ZSet => Cpu::check_bit(self.registers[0].get_low(), Z_FLAG),
            Condition::CNotSet => !Cpu::check_bit(self.registers[0].get_low(), C_FLAG),
            Condition::CSet => Cpu::check_bit(self.registers[0].get_low(), C_FLAG),
        };

        if condition_met {
            self.jump_relative();
        }
        else {
            self.instruction_finished(2, 8);
        }
    }


    // Calls and Returns.
    fn call(&mut self) {
        let target = LittleEndian::read_u16(&vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)]);
        let ret_addr = self.pc + 3;

        self.pc = target;
        self.stack_write(ret_addr);
        self.instruction_finished(0, 24);
    }

    fn call_conditional(&mut self, condition: Condition) {
        let condition_met = match condition {
            Condition::ZNotSet => !Cpu::check_bit(self.registers[0].get_low(), Z_FLAG),
            Condition::ZSet => Cpu::check_bit(self.registers[0].get_low(), Z_FLAG),
            Condition::CNotSet => !Cpu::check_bit(self.registers[0].get_low(), C_FLAG),
            Condition::CSet => Cpu::check_bit(self.registers[0].get_low(), C_FLAG),
        };

        if condition_met {
            self.call();
        }
        else {
            self.instruction_finished(3, 12);
        }
    }

    fn ret(&mut self) {
        self.pc = self.stack_read();
        self.instruction_finished(0, 16);
    }

    fn reti(&mut self) {
        self.interrupts.can_interrupt = true;
        self.ret();
    }

    fn return_conditional(&mut self, condition: Condition) {
        let condition_met = match condition {
            Condition::ZNotSet => !Cpu::check_bit(self.registers[0].get_low(), Z_FLAG),
            Condition::ZSet => Cpu::check_bit(self.registers[0].get_low(), Z_FLAG),
            Condition::CNotSet => !Cpu::check_bit(self.registers[0].get_low(), C_FLAG),
            Condition::CSet => Cpu::check_bit(self.registers[0].get_low(), C_FLAG),
        };

        if condition_met {
            self.ret();
        }
        else {
            self.instruction_finished(1, 8);
        }
    }


    // Register loads.
    fn load_hi_to_hi(&mut self, target: usize, source: usize) {
        let value = self.registers[source].get_hi();
        self.registers[target].set_hi(value);
        self.instruction_finished(1, 4);
    }

    fn load_hi_to_low(&mut self, target: usize, source: usize) {
        let value = self.registers[source].get_hi();
        self.registers[target].set_low(value);
        self.instruction_finished(1, 4);
    }

    fn load_hi_to_hl(&mut self, source: usize) {
        self.memory.write(self.registers[3].get(), self.registers[source].get_hi(), true);
        self.instruction_finished(1, 8);
    }

    fn load_low_to_hi(&mut self, target: usize, source: usize) {
        let value = self.registers[source].get_low();
        self.registers[target].set_hi(value);
        self.instruction_finished(1, 4);
    }

    fn load_low_to_low(&mut self, target: usize, source: usize) {
        let value = self.registers[source].get_low();
        self.registers[target].set_low(value);
        self.instruction_finished(1, 4);
    }

    fn load_low_to_hl(&mut self, source: usize) {
        self.memory.write(self.registers[3].get(), self.registers[source].get_low(), true);
        self.instruction_finished(1, 8);
    }


    // Register immediate loads.
    fn load_immediate_to_hi(&mut self, register: usize) {
        let value = self.memory.read(self.pc + 1);
        self.registers[register].set_hi(value);
        self.instruction_finished(2, 8);
    }

    fn load_immediate_to_low(&mut self, register: usize) {
        let value = self.memory.read(self.pc + 1);
        self.registers[register].set_low(value);
        self.instruction_finished(2, 8);
    }

    fn load_immediate_to_full(&mut self, register: usize) {
        let value = LittleEndian::read_u16(&vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)]);
        self.registers[register].set(value);
        self.instruction_finished(3, 12);
    }


    // Register loads from self.memory.
    fn load_a_from_full(&mut self, register: usize) {
        let value = self.memory.read(self.registers[register].get());

        self.registers[0].set_hi(value);
        self.instruction_finished(1, 8);
    }

    fn load_a_from_hl_inc(&mut self) {
        let target = self.registers[3].get();
        self.registers[0].set_hi(self.memory.read(target));
        self.registers[3].set(target.wrapping_add(1));
        self.instruction_finished(1, 8);
    }

    fn load_a_from_hl_dec(&mut self) {
        let target = self.registers[3].get();
        let value = self.memory.read(target);
        
        self.registers[0].set_hi(value);
        self.registers[3].set(target.wrapping_sub(1));
        self.instruction_finished(1, 8);
    }

    fn load_a_from_ff_immediate(&mut self) {
        let address = 0xFF00 + self.memory.read(self.pc + 1) as u16;
        self.registers[0].set_hi(self.memory.read(address));
        self.instruction_finished(2, 12);
    }

    fn load_a_from_ff_c(&mut self) {
        let address = 0xFF00 + self.registers[1].get_low() as u16;
        self.registers[0].set_hi(self.memory.read(address));
        self.instruction_finished(1, 8);
    }

    fn load_a_from_immediate(&mut self) {
        let address = LittleEndian::read_u16(&vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)]);
        let value = self.memory.read(address);

        self.registers[0].set_hi(value);
        self.instruction_finished(3, 16);
    }

    fn load_hl_to_hi(&mut self, target: usize) {
        let value = self.memory.read(self.registers[3].get());
        self.registers[target].set_hi(value);
        self.instruction_finished(1, 8);
    }

    fn load_hl_to_low(&mut self, target: usize) {
        let value = self.memory.read(self.registers[3].get());
        self.registers[target].set_low(value);
        self.instruction_finished(1, 8);
    }


    // Load the value of HL into SP.
    fn load_hl_to_sp(&mut self) {
        let value = self.registers[3].get();

        self.registers[4].set(value);
        self.instruction_finished(1, 8);
    }


    // Increments.
    fn increment(&mut self, value: u8) -> u8 {
        let hf = Cpu::check_hf_u8((value, 1));
        let result = value.wrapping_add(1);
        self.update_flags(Some(result == 0), Some(false), Some(hf), None);
        result
    }

    fn increment_hi(&mut self, register: usize) {
        let result = self.increment(self.registers[register].get_hi());
        self.registers[register].set_hi(result);
        self.instruction_finished(1, 4);
    }

    fn increment_low(&mut self, register: usize) {
        let result = self.increment(self.registers[register].get_low());
        self.registers[register].set_low(result);
        self.instruction_finished(1, 4);
    }

    fn increment_at_hl(&mut self) {
        let result = self.increment(self.memory.read(self.registers[3].get()));
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(1, 12);
    }

    fn increment_full(&mut self, register: usize) {
        let result = self.registers[register].get().wrapping_add(1);
        self.registers[register].set(result);
        self.instruction_finished(1, 8);
    }


    // Decrements.
    fn decrement(&mut self, value: u8) -> u8 {
        let hf = Cpu::check_borrow_u8((value, 1));
        let result = value.wrapping_sub(1);
        self.update_flags(Some(result == 0), Some(true), Some(hf), None);
        result
    }

    fn decrement_hi(&mut self, register: usize) {
        let result = self.decrement(self.registers[register].get_hi());
        self.registers[register].set_hi(result);
        self.instruction_finished(1, 4);
    }

    fn decrement_low(&mut self, register: usize) {
        let result = self.decrement(self.registers[register].get_low());
        self.registers[register].set_low(result);
        self.instruction_finished(1, 4);
    }

    fn decrement_at_hl(&mut self) {
        let result = self.decrement(self.memory.read(self.registers[3].get()));
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(1, 12);
    }

    fn decrement_full(&mut self, register: usize) {
        let result = self.registers[register].get().wrapping_sub(1);
        self.registers[register].set(result);
        self.instruction_finished(1, 8);
    }


    // ADD.
    fn add(&mut self, value: u8) {
        let hf = Cpu::check_hf_u8((self.registers[0].get_hi(), value));
        let result = self.registers[0].get_hi() as u16 + value as u16;
        self.update_flags(Some(result as u8 == 0), Some(false), Some(hf), Some((result & 0x100) != 0));
        self.registers[0].set_hi(result as u8);
    }

    fn add_hi(&mut self, source: usize) {
        self.add(self.registers[source].get_hi());
        self.instruction_finished(1, 4);
    }

    fn add_low(&mut self, source: usize) {
        self.add(self.registers[source].get_low());
        self.instruction_finished(1, 4);
    }

    fn add_hl(&mut self) {
        self.add(self.memory.read(self.registers[3].get()));
        self.instruction_finished(1, 8);
    }

    fn add_immediate(&mut self) {
        self.add(self.memory.read(self.pc + 1));
        self.instruction_finished(2, 8);
    }


    // SUB.
    fn sub(&mut self, value: u8) {
        let carry = self.registers[0].get_hi() < value;
        let hf = Cpu::check_borrow_u8((self.registers[0].get_hi(), value));
        let result = self.registers[0].get_hi().wrapping_sub(value);

        self.update_flags(Some(result == 0), Some(true), Some(hf), Some(carry));
        self.registers[0].set_hi(result);
    }

    fn sub_hi(&mut self, source: usize) {
        self.sub(self.registers[source].get_hi());
        self.instruction_finished(1, 4);
    }

    fn sub_low(&mut self, source: usize) {
        self.sub(self.registers[source].get_low());
        self.instruction_finished(1, 4);
    }

    fn sub_hl(&mut self) {
        self.sub(self.memory.read(self.registers[3].get()));
        self.instruction_finished(1, 8);
    }

    fn sub_immediate(&mut self) {
        self.sub(self.memory.read(self.pc + 1));
        self.instruction_finished(2, 8);
    }


    // ADC.
    fn adc(&mut self, value: u8) {
        let carry = Cpu::check_bit(self.registers[0].get_low(), C_FLAG) as u8;
        let register = self.registers[0].get_hi();
        let result = register as u16 + value as u16 + carry as u16;

        let hf = ((register & 0x0F) + (value & 0x0F) + carry) > 0x0F;
        let carry = result > 0xFF;

        self.update_flags(Some(result == 0), Some(false), Some(hf), Some(carry));
        self.registers[0].set_hi(result as u8);
    }

    fn adc_hi(&mut self, source: usize) {
        self.adc(self.registers[source].get_hi());
        self.instruction_finished(1, 4);
    }

    fn adc_low(&mut self, source: usize) {
        self.adc(self.registers[source].get_low());
        self.instruction_finished(1, 4);
    }

    fn adc_hl(&mut self) {
        self.adc(self.memory.read(self.registers[3].get()));
        self.instruction_finished(1, 8);
    }

    fn adc_immediate(&mut self) {
        self.adc(self.memory.read(self.pc + 1));
        self.instruction_finished(2, 8);
    }


    // SBC.
    fn sbc(&mut self, value: u8) {
        let register = self.registers[0].get_hi();
        let carry = Cpu::check_bit(self.registers[0].get_low(), C_FLAG) as i16;
        let result = register as i16 - value as i16 - carry;

        let hf = ((register as i16 & 0x0F) - ((value as i16) & 0x0F) - carry) < 0;

        self.update_flags(Some(result == 0), Some(true), Some(hf), Some(result < 0));
        self.registers[0].set_hi(result as u8);
    }

    fn sbc_hi(&mut self, source: usize) {
        self.sbc(self.registers[source].get_hi());
        self.instruction_finished(1, 4);
    }

    fn sbc_low(&mut self, source: usize) {
        self.sbc(self.registers[source].get_low());
        self.instruction_finished(1, 4);
    }

    fn sbc_hl(&mut self) {
        self.sbc(self.memory.read(self.registers[3].get()));
        self.instruction_finished(1, 8);
    }

    fn sbc_immediate(&mut self) {
        self.sbc(self.memory.read(self.pc + 1));
        self.instruction_finished(2, 8);
    }


    // AND.
    fn and(&mut self, value: u8) {
        let result = self.registers[0].get_hi() & value;
        self.update_flags(Some(result == 0), Some(false), Some(true), Some(false));
        self.registers[0].set_hi(result);
    }

    fn and_hi(&mut self, source: usize) {
        self.and(self.registers[source].get_hi());
        self.instruction_finished(1, 4);
    }

    fn and_low(&mut self, source: usize) {
        self.and(self.registers[source].get_low());
        self.instruction_finished(1, 4);
    }

    fn and_hl(&mut self) {
        self.and(self.memory.read(self.registers[3].get()));
        self.instruction_finished(1, 8);
    }

    fn and_immediate(&mut self) {
        self.and(self.memory.read(self.pc + 1));
        self.instruction_finished(2, 8);
    }


    // XOR.
    fn xor(&mut self, value: u8) {
        let result = self.registers[0].get_hi() ^ value;
        self.update_flags(Some(result == 0), Some(false), Some(false), Some(false));
        self.registers[0].set_hi(result);
    }

    fn xor_hi(&mut self, source: usize) {
        self.xor(self.registers[source].get_hi());
        self.instruction_finished(1, 4);
    }

    fn xor_low(&mut self, source: usize) {
        self.xor(self.registers[source].get_low());
        self.instruction_finished(1, 4);
    }

    fn xor_hl(&mut self) {
        self.xor(self.memory.read(self.registers[3].get()));
        self.instruction_finished(1, 8);
    }

    fn xor_immediate(&mut self) {
        self.xor(self.memory.read(self.pc + 1));
        self.instruction_finished(2, 8);
    }


    // OR.
    fn or(&mut self, value: u8) {
        let result = self.registers[0].get_hi() | value;
        self.update_flags(Some(result == 0), Some(false), Some(false), Some(false));
        self.registers[0].set_hi(result);
    }

    fn or_hi(&mut self, source: usize) {
        self.or(self.registers[source].get_hi());
        self.instruction_finished(1, 4);
    }

    fn or_low(&mut self, source: usize) {
        self.or(self.registers[source].get_low());
        self.instruction_finished(1, 4);
    }

    fn or_hl(&mut self) {
        self.or(self.memory.read(self.registers[3].get()));
        self.instruction_finished(1, 8);
    }

    fn or_immediate(&mut self) {
        self.or(self.memory.read(self.pc + 1));
        self.instruction_finished(2, 8);
    }


    // CP.
    fn cp(&mut self, value: u8) {
        let hf = Cpu::check_borrow_u8((self.registers[0].get_hi(), value));
        let result = self.registers[0].get_hi().overflowing_sub(value);
        self.update_flags(Some(result.0 == 0), Some(true), Some(hf), Some(result.1));
    }
    fn cp_hi(&mut self, source: usize) {
        self.cp(self.registers[source].get_hi());
        self.instruction_finished(1, 4);
    }

    fn cp_low(&mut self, source: usize) {
        self.cp(self.registers[source].get_low());
        self.instruction_finished(1, 4);
    }

    fn cp_hl(&mut self) {
        let value = self.memory.read(self.registers[3].get());
        self.cp(value);
        self.instruction_finished(1, 8);
    }

    fn cp_immediate(&mut self) {
        let value = self.memory.read(self.pc + 1);
        self.cp(value);
        self.instruction_finished(2, 8);
    }


    // Add 16bit register value to HL.
    fn add_full_to_hl(&mut self, register: usize) {
        let hl = self.registers[3].get();
        let half_carry = Cpu::check_hf_u16((hl, self.registers[register].get()));
        let result = (hl as u32) + (self.registers[register].get() as u32);
        let carry = (result & 0x10000) != 0;

        self.registers[3].set(result as u16);
        self.update_flags(None, Some(false), Some(half_carry), Some(carry));
        
        self.instruction_finished(1, 8);
    }


    // Rotate left.
    fn rla(&mut self) {
        let value = self.registers[0].get_hi() << 1;
        let carry = Cpu::check_bit(self.registers[0].get_low(), C_FLAG);

        self.update_flags(Some(false), Some(false), Some(false), Some(Cpu::check_bit(self.registers[0].get_hi(), 7)));
        self.registers[0].set_hi(value | carry as u8);
        self.instruction_finished(1, 4);
    }

    fn rlca(&mut self) {
        let carry = Cpu::check_bit(self.registers[0].get_hi(), 7);
        let result = self.registers[0].get_hi().rotate_left(1);

        self.registers[0].set_hi(result);
        self.update_flags(Some(false), Some(false), Some(false), Some(carry));
        self.instruction_finished(1, 4);
    }


    // Rotate right.
    fn rra(&mut self) {
        let will_carry = Cpu::check_bit(self.registers[0].get_hi(), 0);
        let current_carry = Cpu::check_bit(self.registers[0].get_low(), C_FLAG) as u8;
        let result = (self.registers[0].get_hi() >> 1) | (current_carry << 7);

        self.update_flags(Some(false), Some(false), Some(false), Some(will_carry));
        self.registers[0].set_hi(result);
        self.instruction_finished(1, 4);
    }

    fn rrca(&mut self) {
        let carry = Cpu::check_bit(self.registers[0].get_hi(), 0);
        let result = self.registers[0].get_hi().rotate_right(1);

        self.registers[0].set_hi(result);
        self.update_flags(Some(false), Some(false), Some(false), Some(carry));
        self.instruction_finished(1, 4);
    }


    // Push and Pop registers.
    fn pop_register(&mut self, target: usize) {
        let value = self.stack_read();
        self.registers[target].set(value);
        self.instruction_finished(1, 12);
    }

    fn push_register(&mut self, target: usize) {
        self.stack_write(self.registers[target].get());
        self.instruction_finished(1, 16);
    }


    // Carry flag manipulation.
    fn scf(&mut self) {
        self.update_flags(None, Some(false), Some(false), Some(true));
        self.instruction_finished(1, 4);
    }

    fn ccf(&mut self) {
        self.update_flags(None, Some(false), Some(false), Some(!Cpu::check_bit(self.registers[0].get_low(), C_FLAG)));
        self.instruction_finished(1, 4);
    }


    // Save A to self.memory.
    fn save_a_to_full(&mut self, register: usize) {
        let value = self.registers[0].get_hi();
        let address = self.registers[register].get();
        self.memory.write(address, value, true);
        self.instruction_finished(1, 8);
    }

    fn save_a_to_hl_inc(&mut self) {
        let target = self.registers[3].get();
        self.memory.write(target, self.registers[0].get_hi(), true);
        self.registers[3].set(target.wrapping_add(1));
        self.instruction_finished(1, 8);
    }

    fn save_a_to_hl_dec(&mut self) {
        let target = self.registers[3].get();
        self.memory.write(target, self.registers[0].get_hi(), true);
        self.registers[3].set(target.wrapping_sub(1));
        self.instruction_finished(1, 8);
    }

    fn save_a_to_immediate(&mut self) {
        let address = LittleEndian::read_u16(&vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)]);
        self.memory.write(address, self.registers[0].get_hi(), true);
        self.instruction_finished(3, 16);
    }

    fn save_a_to_ff_immediate(&mut self) {
        let target = 0xFF00 + self.memory.read(self.pc + 1) as u16;
        self.memory.write(target, self.registers[0].get_hi(), true);
        self.instruction_finished(2, 12);
    }

    fn save_a_to_ff_c(&mut self) {
        let target = 0xFF00 + self.registers[1].get_low() as u16;
        self.memory.write(target, self.registers[0].get_hi(), true);
        self.instruction_finished(1, 8);
    }


    // Save SP to address pointed by immediate 16bit value.
    fn save_sp_to_immediate(&mut self) {
        let address = LittleEndian::read_u16(&vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)]);

        self.memory.write(address, self.registers[4].get_low(), true);
        self.memory.write(address + 1, self.registers[4].get_hi(), true);
        self.instruction_finished(3, 20);
    }


    // Save the immediate 8bit value to address pointed by HL.
    fn save_immediate_to_hl(&mut self) {
        let value = self.memory.read(self.pc + 1);
        self.memory.write(self.registers[3].get(), value, true);
        self.instruction_finished(2, 12);
    }


    // Add signed immediate value to SP.
    fn add_signed_immediate_to_sp(&mut self) {
        let register = self.registers[4].get();
        let value = self.memory.read(self.pc + 1) as i8;
        let result = register.wrapping_add(value as u16);

        let hf = ((register ^ value as u16 ^ (result & 0xFFFF)) & 0x10) == 0x10;
        let carry = ((register ^ value as u16 ^ (result & 0xFFFF)) & 0x100) == 0x100;

        self.registers[4].set(result);
        self.update_flags(Some(false), Some(false), Some(hf), Some(carry));
        self.instruction_finished(2, 16);
    }


    // Reset PC to address.
    fn rst(&mut self, address: u16) {
        self.stack_write(self.pc + 1);
        self.pc = address;
        self.instruction_finished(0, 16);
    }


    // Complement A's value.
    fn cpl(&mut self) {
        let result = !self.registers[0].get_hi();
        self.registers[0].set_hi(result);
        self.update_flags(None, Some(true), Some(true), None);
        self.instruction_finished(1, 4);
    }

    
    // Save (SP + signed 8bit immediate) to HL.
    fn load_sp_plus_signed_to_hl(&mut self) {
        let register = self.registers[4].get();
        let value = self.memory.read(self.pc + 1) as i8;
        let result = register.wrapping_add(value as u16);

        let hf = ((register ^ value as u16 ^ (result & 0xFFFF)) & 0x10) == 0x10;
        let carry = ((register ^ value as u16 ^ (result & 0xFFFF)) & 0x100) == 0x100;

        self.registers[3].set(result);
        self.update_flags(Some(false), Some(false), Some(hf), Some(carry));
        self.instruction_finished(2, 12);
    }


    // Disable/Enable Interrupts.
    fn di(&mut self) {
        self.interrupts.can_interrupt = false;
        self.instruction_finished(1, 4);
    }

    fn ei(&mut self) {
        self.interrupts.can_interrupt = true;
        self.instruction_finished(1, 4);
    }



    // Prefixed Instructions.

    fn rlc(&mut self, value: u8) -> u8 {
        let carry = (value >> 7) == 1;
        let result = value.rotate_left(1);

        self.update_flags(Some(result == 0), Some(false), Some(false), Some(carry));
        result
    }

    fn rlc_hi(&mut self, target: usize) {
        let result = self.rlc(self.registers[target].get_hi());
        self.registers[target].set_hi(result);
        self.instruction_finished(2, 8);
    }

    fn rlc_low(&mut self, target: usize) {
        let result = self.rlc(self.registers[target].get_low());
        self.registers[target].set_low(result);
        self.instruction_finished(2, 8);
    }

    fn rlc_hl(&mut self) {
        let result = self.rlc(self.memory.read(self.registers[3].get()));
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(2, 16);
    }

    fn rrc(&mut self, value: u8) -> u8 {
        let carry = (value & 1) == 1;
        let result = value.rotate_right(1);

        self.update_flags(Some(result == 0), Some(false), Some(false), Some(carry));
        result
    }

    fn rrc_hi(&mut self, target: usize) {
        let result = self.rrc(self.registers[target].get_hi());
        self.registers[target].set_hi(result);
        self.instruction_finished(2, 8);
    }

    fn rrc_low(&mut self, target: usize) {
        let result = self.rrc(self.registers[target].get_low());
        self.registers[target].set_low(result);
        self.instruction_finished(2, 8);
    }

    fn rrc_hl(&mut self) {
        let result = self.rrc(self.memory.read(self.registers[3].get()));
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(2, 8);
    }

    fn rl(&mut self, value: u8) -> u8 {
        let new_carry = ((value >> 7) & 1) == 1;
        let current_carry = Cpu::check_bit(self.registers[0].get_low(), C_FLAG);
        let result = (value << 1) | current_carry as u8;

        self.update_flags(Some(result == 0), Some(false), Some(false), Some(new_carry));
        result
    }

    fn rl_hi(&mut self, target: usize) {
        let result = self.rl(self.registers[target].get_hi());
        self.registers[target].set_hi(result);
        self.instruction_finished(2, 8);
    }

    fn rl_low(&mut self, target: usize) {
        let result = self.rl(self.registers[target].get_low());
        self.registers[target].set_low(result);
        self.instruction_finished(2, 8);
    }

    fn rl_hl(&mut self) {
        let result = self.rl(self.memory.read(self.registers[3].get()));
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(2, 16);
    }

    fn rr(&mut self, value: u8) -> u8 {
        let new_carry = (value & 1) == 1;
        let current_carry = Cpu::check_bit(self.registers[0].get_low(), C_FLAG) as u8;
        let result = (value >> 1) | (current_carry << 7);

        self.update_flags(Some(result == 0), Some(false), Some(false), Some(new_carry));
        result
    }

    fn rr_hi(&mut self, target: usize) {
        let result = self.rr(self.registers[target].get_hi());
        self.registers[target].set_hi(result);
        self.instruction_finished(2, 8);
    }

    fn rr_low(&mut self, target: usize) {
        let result = self.rr(self.registers[target].get_low());
        self.registers[target].set_low(result);
        self.instruction_finished(2, 8);
    }

    fn rr_hl(&mut self) {
        let result = self.rr(self.memory.read(self.registers[3].get()));
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(2, 16);
    }

    fn sla(&mut self, value: u8) -> u8 {
        let carry = (value >> 7) == 1;
        let result = value << 1;

        self.update_flags(Some(result == 0), Some(false), Some(false), Some(carry));
        result
    }

    fn sla_hi(&mut self, target: usize) {
        let result = self.sla(self.registers[target].get_hi());
        self.registers[target].set_hi(result);
        self.instruction_finished(2, 8);
    }

    fn sla_low(&mut self, target: usize) {
        let result = self.sla(self.registers[target].get_low());
        self.registers[target].set_low(result);
        self.instruction_finished(2, 8);
    }

    fn sla_hl(&mut self) {
        let result = self.sla(self.memory.read(self.registers[3].get()));
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(2, 16);
    }

    fn sra(&mut self, value: u8) -> u8 {
        let carry = (value & 1) == 1;
        let msb = (value >> 7) == 1;
        
        let mut result = value >> 1;
        if msb {
            result |= 1 << 7;
        }
        else {
            result &= !(1 << 7);
        }

        self.update_flags(Some(result == 0), Some(false), Some(false), Some(carry));
        result
    }

    fn sra_hi(&mut self, target: usize) {
        let result = self.sra(self.registers[target].get_hi());
        self.registers[target].set_hi(result);
        self.instruction_finished(2, 8);
    }

    fn sra_low(&mut self, target: usize) {
        let result = self.sra(self.registers[target].get_low());
        self.registers[target].set_low(result);
        self.instruction_finished(2, 8);
    }

    fn sra_hl(&mut self) {
        let result = self.sra(self.memory.read(self.registers[3].get()));
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(2, 16);
    }

    fn swap(&mut self, value: u8) -> u8 {
        let result = (value >> 4) | (value << 4);
        self.update_flags(Some(result == 0), Some(false), Some(false), Some(false));
        result
    }

    fn swap_hi(&mut self, target: usize) {
        let result = self.swap(self.registers[target].get_hi());
        self.registers[target].set_hi(result);
        self.instruction_finished(2, 8);
    }

    fn swap_low(&mut self, target: usize) {
        let result = self.swap(self.registers[target].get_low());
        self.registers[target].set_low(result);
        self.instruction_finished(2, 8);
    }

    fn swap_hl(&mut self) {
        let result = self.swap(self.memory.read(self.registers[3].get()));
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(2, 16);
    }

    fn srl(&mut self, value: u8) -> u8 {
        let carry = (value & 1) == 1;
        let result = value >> 1;

        self.update_flags(Some(result == 0), Some(false), Some(false), Some(carry));
        result
    }

    fn srl_hi(&mut self, target: usize) {
        let result = self.srl(self.registers[target].get_hi());
        self.registers[target].set_hi(result);
        self.instruction_finished(2, 8);
    }

    fn srl_low(&mut self, target: usize) {
        let result = self.srl(self.registers[target].get_low());
        self.registers[target].set_low(result);
        self.instruction_finished(2, 8);
    }

    fn srl_hl(&mut self) {
        let result = self.srl(self.memory.read(self.registers[3].get()));
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(2, 16);
    }

    fn bit(&mut self, value: u8, bit: u8) {
        let result = ((value >> bit) & 1) == 0;
        self.update_flags(Some(result), Some(false), Some(true), None);
    }

    fn bit_hi(&mut self, target: usize, bit: u8) {
        self.bit(self.registers[target].get_hi(), bit);
        self.instruction_finished(2, 8);
    }

    fn bit_low(&mut self, target: usize, bit: u8) {
        self.bit(self.registers[target].get_low(), bit);
        self.instruction_finished(2, 8);
    }

    fn bit_hl(&mut self, bit: u8) {
        self.bit(self.memory.read(self.registers[3].get()), bit);
        self.instruction_finished(2, 16);
    }

    fn res(&mut self, value: u8, bit: u8) -> u8 {
        value & !(1 << bit)
    }

    fn res_hi(&mut self, target: usize, bit: u8) {
        let result = self.res(self.registers[target].get_hi(), bit);
        self.registers[target].set_hi(result);
        self.instruction_finished(2, 8);
    }

    fn res_low(&mut self, target: usize, bit: u8) {
        let result = self.res(self.registers[target].get_low(), bit);
        self.registers[target].set_low(result);
        self.instruction_finished(2, 8);
    }

    fn res_hl(&mut self, bit: u8) {
        let result = self.res(self.memory.read(self.registers[3].get()), bit);
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(2, 16);
    }

    fn set(&mut self, value: u8, bit: u8) -> u8 {
        value | (1 << bit)
    }

    fn set_hi(&mut self, target: usize, bit: u8) {
        let result = self.set(self.registers[target].get_hi(), bit);
        self.registers[target].set_hi(result);
        self.instruction_finished(2, 8);
    }

    fn set_low(&mut self, target: usize, bit: u8) {
        let result = self.set(self.registers[target].get_low(), bit);
        self.registers[target].set_low(result);
        self.instruction_finished(2, 8);
    }

    fn set_hl(&mut self, bit: u8) {
        let result = self.set(self.memory.read(self.registers[3].get()), bit);
        self.memory.write(self.registers[3].get(), result, true);
        self.instruction_finished(2, 16);
    }



    // Utils

    fn check_bit(value: u8, bit: u8) -> bool {
        ((value >> bit) & 1) == 1
    }

    fn set_bit(value: u8, bit: u8, new_value: bool) -> u8 {
        if new_value {
            value | (1 << bit)
        }
        else {
            value & !(1 << bit)
        }
    }

    fn check_hf_u8(values: (u8, u8)) -> bool {
        (((values.0 & 0x0F) + (values.1 & 0x0F)) & 0x10) == 0x10
    }

    fn check_hf_u16(values: (u16, u16)) -> bool {
        ((((values.0 & 0x0FFF) + (values.1 & 0x0FFF))) & 0x1000) == 0x1000
    }

    fn check_borrow_u8(values: (u8, u8)) -> bool {
        ((values.0 & 0xF) as i8 - (values.1 & 0xF) as i8) < 0
    }
}

pub struct UiObject {
    pub registers: Vec<u16>,
    pub pc: u16,
    pub opcode: u8,

    pub halted: bool,

    pub cpu_status: Status,
    pub cpu_paused: Option<bool>,
    pub cpu_step: Option<bool>,

    pub breakpoints: Vec<u16>,
    pub breakpoint_hit: bool
}

impl UiObject {
    pub fn new() -> UiObject {
        UiObject {
            registers: vec![0; 5],
            pc: 0,
            opcode: 0,

            halted: false,

            cpu_status: Status::NotReady,
            cpu_paused: None,
            cpu_step: None,

            breakpoints: Vec::new(),
            breakpoint_hit: false
        }
    }
}