use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::sync::atomic::{AtomicU16, Ordering};

use log::error;
use byteorder::{ByteOrder, LittleEndian};

use super::timer::TimerModule;
use super::emulator::{KeyType, InputEvent};
use super::memory::Memory;

const ZF_BIT: u8 = 7;
const NF_BIT: u8 = 6;
const HF_BIT: u8 = 5;
const CF_BIT: u8 = 4;


#[derive(Clone)]
pub struct Register {
    value: u8,
}

impl Register {
    pub fn new() -> Register {
        Register {
            value: 0,
        }
    }

    pub fn get(&self) -> u8 {
        self.value
    }

    pub fn set(&mut self, value: u8) {
        self.value = value;
    }
}

pub struct FlagsRegister {
    value: u8,
}

impl FlagsRegister {
    pub fn new() -> FlagsRegister {
        FlagsRegister {
            value: 0,
        }
    }

    pub fn get_value(&self) -> u8 {
        self.value
    }

    pub fn set_value(&mut self, value: u8) {
        self.value = value;
    }


    pub fn get_zf(&self) -> u8 {
        (self.value >> ZF_BIT) & 1
    }

    pub fn get_cf(&self) -> u8 {
        (self.value >> CF_BIT) & 1
    }


    pub fn set_zf(&mut self, value: bool) {
        if value {
            self.value |= 1 << ZF_BIT;
        }
        else {
            self.value &= !(1 << ZF_BIT);
        }
    }

    pub fn set_nf(&mut self, value: bool) {
        if value {
            self.value |= 1 << NF_BIT;
        }
        else {
            self.value &= !(1 << NF_BIT);
        }
    }

    pub fn set_hf(&mut self, value: bool) {
        if value {
            self.value |= 1 << HF_BIT;
        }
        else {
            self.value &= !(1 << HF_BIT);
        }
    }

    pub fn set_cf(&mut self, value: bool) {
        if value {
            self.value |= 1 << CF_BIT;
        }
        else {
            self.value &= !(1 << CF_BIT);
        }
    }
}

pub struct Instruction {
    pub x: u8,
    pub y: u8,
    pub z: u8,

    pub p: u8,
    pub q: u8,
}

impl Instruction {
    pub fn new(value: u8) -> Instruction {
        Instruction {
            x: (value >> 6),
            y: (value >> 3) & 7,
            z: value & 7,

            p: (value >> 4) & 3,
            q: (value >> 3) & 1,
        }
    }
}

pub struct Cpu {

    // B, C, D, E, H, L, (HL), A.
    // (HL) is added to the Vec for now, but it's not used.
    registers: Vec<Register>,
    cpu_flags: FlagsRegister,

    pc: u16,
    sp: u16,
    cycles: Arc<AtomicU16>,

    halted: bool,
    stopped: bool,
    interrupts_enabled: bool,

    timer: TimerModule,

    memory: Arc<Memory>,

    input_queue: Vec<InputEvent>,
    input_receiver: Receiver<InputEvent>,
}

impl Cpu {
    pub fn new(memory: Arc<Memory>, cycles: Arc<AtomicU16>, input: Receiver<InputEvent>, run_bootrom: bool) -> Cpu {
        
        let timer_cycles = Arc::clone(&cycles);

        memory.write(0xFF00, 0xCF, true);
        
        Cpu {
            registers: vec![Register::new(); 8],
            cpu_flags: FlagsRegister::new(),
            
            pc: if run_bootrom {0x0} else {0x100},
            sp: 0,
            cycles: cycles,

            halted: false,
            stopped: false,
            interrupts_enabled: false,

            timer: TimerModule::new(timer_cycles, Arc::clone(&memory)),

            memory: memory,

            input_queue: Vec::new(),
            input_receiver: input,
        }
    }

    pub fn get_rp(&mut self, index: u8) -> u16 {
        match index {
            0 => {
                return ((self.registers[0].get() as u16) << 8) | (self.registers[1].get() as u16);
            },
            1 => {
                return ((self.registers[2].get() as u16) << 8) | (self.registers[3].get() as u16);
            },
            2 => {
                return ((self.registers[4].get() as u16) << 8) | (self.registers[5].get() as u16);
            }
            3 => {
                return self.sp;
            }
            _ => panic!("Invalid register pair index {}", index),
        }
    }

    pub fn get_rp2(&mut self, index: u8) -> u16 {
        match index {
            0 => {
                return ((self.registers[0].get() as u16) << 8) | (self.registers[1].get() as u16);
            },
            1 => {
                return ((self.registers[2].get() as u16) << 8) | (self.registers[3].get() as u16);
            },
            2 => {
                return ((self.registers[4].get() as u16) << 8) | (self.registers[5].get() as u16);
            }
            3 => {
                return ((self.registers[7].get() as u16) << 8) | (self.cpu_flags.get_value() as u16);
            }
            _ => panic!("Invalid register pair index {}", index),
        }
    }

    pub fn set_rp(&mut self, index: u8, value: u16) {
        let hi = (value >> 8) as u8;
        let low = value as u8;

        match index {
            0 => {
                self.registers[0].set(hi);
                self.registers[1].set(low);
            },
            1 => {
                self.registers[2].set(hi);
                self.registers[3].set(low);
            },
            2 => {
                self.registers[4].set(hi);
                self.registers[5].set(low);
            },
            3 => {
                self.sp = value;
            },
            _ => panic!("Invalid index for register pair"),
        }
    }

    pub fn set_rp2(&mut self, index: u8, value: u16) {
        let hi = (value >> 8) as u8;
        let low = value as u8;

        match index {
            0 => {
                self.registers[0].set(hi);
                self.registers[1].set(low);
            },
            1 => {
                self.registers[2].set(hi);
                self.registers[3].set(low);
            },
            2 => {
                self.registers[4].set(hi);
                self.registers[5].set(low);
            },
            3 => {
                self.registers[7].set(hi);
                self.cpu_flags.set_value(low);
            },
            _ => panic!("Invalid index for register pair"),
        }
    }

    pub fn get_register(&mut self, index: u8) -> u8 {
        if index == 6 {
            let address = self.get_rp(2);
            return self.memory.read(address);
        }

        self.registers[index as usize].get()
    }

    pub fn set_register(&mut self, index: u8, value: u8) {
        if index == 6 {
            let address = self.get_rp(2);
            self.memory.write(address, value, true);
            return;
        }

        self.registers[index as usize].set(value);
    }

    fn stack_read(&mut self) -> u16 {
        let sp = self.get_rp(3);
        let bytes = vec![self.memory.read(sp), self.memory.read(sp + 1)];

        self.set_rp(3, sp + 2);
        LittleEndian::read_u16(&bytes)
    }

    fn stack_write(&mut self, value: u16) {
        let hi = (value >> 8) as u8;
        let low = value as u8;
        let sp = self.get_rp(3);

        self.memory.write(sp - 1, hi, true);
        self.memory.write(sp - 2, low, true);
        self.set_rp(3, sp - 2);
    }

    pub fn execution_loop(&mut self) {

        loop {
            if self.update_input_queue() {break}
            self.check_interrupts();
            if !self.halted {self.run_instruction()}
            self.timer.timer_cycle();
        }
    }

    fn update_input_queue(&mut self) -> bool {

        for event in self.input_receiver.try_iter() {
            if event.get_event() == KeyType::QuitEvent {
                return true;
            }
            if event.should_keep() {
                self.input_queue.push(event);
            }
        }

        let input_reg = self.memory.read(0xFF00);

        if self.input_queue.len() > 0 {
            if ((input_reg >> 4) & 3) != 0 || input_reg == 0xCF {
                self.process_input();
            }
        }

        false
    }

    fn process_input(&mut self) {
        let event = self.input_queue.remove(0);
        let mut input_reg = self.memory.read(0xFF00) | 0xCF;

        // Start, Select, A, and B.
        if input_reg == 0xEF {
            match event.get_event() {
                KeyType::Start => input_reg &= 0xE7,
                KeyType::Select => input_reg &= 0xEB,
                KeyType::B => input_reg &= 0xED,
                KeyType::A => input_reg &= 0xEE,
                _ => {},
            }
        }
        // Directional pad
        else if input_reg == 0xDF {
            match event.get_event() {
                KeyType::Down => input_reg &= 0xD7,
                KeyType::Up => input_reg &= 0xDB,
                KeyType::Left => input_reg &= 0xDD,
                KeyType::Right => input_reg &= 0xDE,
                _ => {},
            }
        }
        // Anything goes
        else if input_reg == 0xFF {
            match event.get_event() {
                KeyType::Down => input_reg &= 0xD7,
                KeyType::Up => input_reg &= 0xDB,
                KeyType::Left => input_reg &= 0xDD,
                KeyType::Right => input_reg &= 0xDE,
                KeyType::Start => input_reg &= 0xE7,
                KeyType::Select => input_reg &= 0xEB,
                KeyType::B => input_reg &= 0xED,
                KeyType::A => input_reg &= 0xEE,
                _ => {},
            }
        }

        let if_flag = self.memory.read(0xFF0F);

        self.memory.write(0xFF00, input_reg, true);
        self.memory.write(0xFF0F, if_flag | (1 << 4), true);
    }

    fn check_interrupts(&mut self) {
        let if_value = self.memory.read(0xFF0F);
        let ie_value = self.memory.read(0xFFFF);

        let vblank_int = (if_value & 1) == 1;
        let lcdc_int = ((if_value >> 1) & 1) == 1;
        let timer_int = ((if_value >> 2) & 1) == 1;
        let serial_int = ((if_value >> 3) & 1) == 1;
        let input_int = ((if_value >> 4) & 1) == 1;

        // Vblank interrupt.
        if vblank_int {
            if self.interrupts_enabled && (ie_value & 1) == 1 {
                self.memory.write(0xFF0F, if_value & !(1), true);
                self.stack_write(self.pc);
                self.pc = 0x0040;
                self.interrupts_enabled = false;
            }
            self.halted = false;
        }
        // LCDC interrupt.
        else if lcdc_int {
            if self.interrupts_enabled && ((ie_value >> 1) & 1) == 1 {
                self.memory.write(0xFF0F, if_value & !(1 << 1), true);
                self.stack_write(self.pc);
                self.pc = 0x0048;
                self.interrupts_enabled = false;

            }
            self.halted = false;
        }
        // Timer interrupt.
        else if timer_int {
            if self.interrupts_enabled && ((ie_value >> 2) & 1) == 1 {
                self.memory.write(0xFF0F, if_value & !(1 << 2), true);
                self.stack_write(self.pc);
                self.pc = 0x0050;
                self.interrupts_enabled = false;
            }
            self.halted = false;
        }
        // Serial transfer interrupt.
        else if serial_int {
            if self.interrupts_enabled && ((ie_value >> 3) & 1) == 1 {
                self.memory.write(0xFF0F, if_value & !(1 << 3), true);
                self.stack_write(self.pc);
                self.pc = 0x0058;
                self.interrupts_enabled = false;
            }
            self.halted = false;
        }
        // Input interrupt.
        else if input_int {
            if self.interrupts_enabled && ((ie_value >> 4) & 1) == 1 {
                self.memory.write(0xFF0F, if_value & !(1 << 4), true);
                self.stack_write(self.pc);
                self.pc = 0x0060;
                self.interrupts_enabled = false;
            }
            self.halted = false;
        }
    }

    fn run_instruction(&mut self) {
        
        if self.pc == 0x0100 {
            log::info!("CPU: Bootrom execution finished, executing loaded ROM.");
            self.memory.bootrom_finished();
        }
        
        let opcode = self.memory.read(self.pc);

        if opcode == 0xCB {
            let opcode = self.memory.read(self.pc + 1);
            let instruction = Instruction::new(opcode);

            if instruction.x == 0 {
                match instruction.y {
                    0 => self.rlc(instruction.z),
                    1 => self.rrc(instruction.z),
                    2 => self.rl(instruction.z),
                    3 => self.rr(instruction.z),
                    4 => self.sla(instruction.z),
                    5 => self.sra(instruction.z),
                    6 => self.swap(instruction.z),
                    7 => self.srl(instruction.z),
                    _ => panic!("Invalid operation"),
                }
            }
            else if instruction.x == 1 {
                self.bit(instruction.z, instruction.y);
            }
            else if instruction.x == 2 {
                self.res(instruction.z, instruction.y);
                return;
            }
            else if instruction.x == 3 {
                self.set(instruction.z, instruction.y);
            }
        }
        else {
            let instruction = Instruction::new(opcode);

            if instruction.x == 0 {

                if instruction.z == 0 {
                    match instruction.y {
                        0 => self.nop(),
                        1 => self.save_sp_to_imm(),
                        2 => self.stop(),
                        3 => self.jr(),
                        4 | 5 | 6 | 7 => self.jr_cc(instruction.y - 4),
                        _ => panic!("Invalid operation"),
                    }
                }
                else if instruction.z == 1 {
                    match instruction.q {
                        0 => self.load_imm_to_rp(instruction.p),
                        1 => self.add_rp_to_hl(instruction.p),
                        _ => panic!("Invalid operation"),
                    }
                }
                else if instruction.z == 2 {
                    if instruction.q == 0 {
                        match instruction.p {
                            0 => self.save_a_to_rp(0),
                            1 => self.save_a_to_rp(1),
                            2 => self.save_a_to_hl_inc(),
                            3 => self.save_a_to_hl_dec(),
                            _ => panic!("Invalid operation"),
                        }
                    }
                    else if instruction.q == 1 {
                        match instruction.p {
                            0 => self.load_a_from_rp(0),
                            1 => self.load_a_from_rp(1),
                            2 => self.load_a_from_hl_inc(),
                            3 => self.load_a_from_hl_dec(),
                            _ => panic!("Invalid operation"),
                        }
                    }
                }
                else if instruction.z == 3 {
                    if instruction.q == 0 {
                        self.inc_rp(instruction.p);
                    }
                    else if instruction.q == 1 {
                        self.dec_rp(instruction.p);
                    }
                }
                else if instruction.z == 4 {
                    self.inc_reg(instruction.y);
                }
                else if instruction.z == 5 {
                    self.dec_reg(instruction.y);
                }
                else if instruction.z == 6 {
                    self.load_imm_into_reg(instruction.y);
                }
                else if instruction.z == 7 {
                    match instruction.y {
                        0 => self.rlca(),
                        1 => self.rrca(),
                        2 => self.rla(),
                        3 => self.rra(),
                        4 => self.daa(),
                        5 => self.cpl(),
                        6 => self.scf(),
                        7 => self.ccf(),
                        _ => panic!("Invalid operation"),
                    }
                }
            }
            else if instruction.x == 1 {
                if instruction.z == 6 && instruction.y == 6 {
                    self.halt();
                }
                else {
                    self.load_reg_into_reg(instruction.y, instruction.z);
                }
            }
            else if instruction.x == 2 {
                match instruction.y {
                    0 => self.add(instruction.z),
                    1 => self.adc(instruction.z),
                    2 => self.sub(instruction.z),
                    3 => self.sbc(instruction.z),
                    4 => self.and(instruction.z),
                    5 => self.xor(instruction.z),
                    6 => self.or(instruction.z),
                    7 => self.cp(instruction.z),
                    _ => panic!("Invalid operation")
                }
            }
            else if instruction.x == 3 {
                if instruction.z == 0 {
                    match instruction.y {
                        0 | 1 | 2 | 3 => self.ret_cc(instruction.y),
                        4 => self.save_a_to_ff_imm(),
                        5 => self.add_imm_to_sp(),
                        6 => self.load_a_from_ff_imm(),
                        7 => self.load_sp_imm_to_hl(),
                        _ => panic!("Invalid operation"),
                    }
                }
                else if instruction.z == 1 {
                    if instruction.q == 0 {
                        self.pop(instruction.p);
                    }
                    else if instruction.q == 1 {
                        match instruction.p {
                            0 => self.ret(),
                            1 => self.reti(),
                            2 => self.jp_hl(),
                            3 => self.load_hl_to_sp(),
                            _ => panic!("Invalid operation"),
                        }
                    }
                }
                else if instruction.z == 2 {
                    match instruction.y {
                        0|1|2|3 => self.jp_cc(instruction.y),
                        4 => self.save_a_to_ff_c(),
                        5 => self.save_a_to_imm(),
                        6 => self.load_a_from_ff_c(),
                        7 => self.load_a_from_imm(),
                        _ => panic!("Invalid operation"),
                    }
                }
                else if instruction.z == 3 {
                    match instruction.y {
                        0 => self.jp(),
                        1 => panic!("Prefixed opcode in unprefixed codepath"),
                        6 => self.di(),
                        7 => self.ei(),
                        _ => panic!("Invalid operation"),
                    }
                }
                else if instruction.z == 4 {
                    if instruction.y < 4 {
                        self.call_cc(instruction.y);
                        return;
                    }
                    panic!("Invalid operation");
                }
                else if instruction.z == 5 {
                    if instruction.q == 0 {
                        self.push(instruction.p);
                    }
                    else if instruction.q == 1 && instruction.p == 0 {
                        self.call();
                    }
                    else {
                        panic!("Invalid operation");
                    }
                }
                else if instruction.z == 6 {
                    match instruction.y {
                        0 => self.add_imm(),
                        1 => self.adc_imm(),
                        2 => self.sub_imm(),
                        3 => self.sbc_imm(),
                        4 => self.and_imm(),
                        5 => self.xor_imm(),
                        6 => self.or_imm(),
                        7 => self.cp_imm(),
                        _ => panic!("Invalid operation"),
                    }
                }
                else if instruction.z == 7 {
                    self.rst(instruction.y * 8);
                }
            }
        }
    }

    fn instruction_finished(&mut self, pc: u16, cycles: u16) {
        self.pc += pc;
        self.cycles.fetch_add(cycles, Ordering::Relaxed);
    }

    fn nop(&mut self) {
        self.instruction_finished(1, 4);
    }

    fn save_sp_to_imm(&mut self) {
        let value = self.get_rp(3);
        let bytes = vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)];
        let hi = (value >> 8) as u8;
        let low = value as u8;
        let address = LittleEndian::read_u16(&bytes);

        self.memory.write(address, low, true);
        self.memory.write(address + 1, hi, true);
        self.instruction_finished(3, 20);
    }

    fn stop(&mut self) {
        error!("Unimplemented opcode STOP. Execution will continue, but things may break");
        self.instruction_finished(2, 4);
    }

    fn jr(&mut self) {
        let value = self.memory.read(self.pc + 1) as i8;
        self.pc = self.pc.wrapping_add(value as u16) + 2;
        self.instruction_finished(0, 12);
    }

    fn jr_cc(&mut self, condition: u8) {
        let jump = match condition {
            0 => self.cpu_flags.get_zf() == 0,
            1 => self.cpu_flags.get_zf() == 1,
            2 => self.cpu_flags.get_cf() == 0,
            3 => self.cpu_flags.get_cf() == 1,
            _ => panic!("Invalid jump condition"),
        };

        if jump {
            self.jr();
        }
        else {
            self.instruction_finished(2, 8);
        }
    }

    // Load 16-bit immediate value to a register pair (BC, DE, HL, SP).
    fn load_imm_to_rp(&mut self, index: u8) {
        let bytes = vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)];
        
        self.set_rp(index, LittleEndian::read_u16(&bytes));
        self.instruction_finished(3, 12);
    }

    fn add_rp_to_hl(&mut self, index: u8) {
        let hl = self.get_rp(2) as u32;
        let value = self.get_rp(index) as u32;
        let hf = ((hl & 0xFFF) + (value & 0xFFF)) > 0xFFF;
        let result = hl + value;

        self.set_rp(2, result as u16);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(hf);
        self.cpu_flags.set_cf((result & 0x10000) != 0);
        self.instruction_finished(1, 8);
    }

    fn load_a_from_rp(&mut self, index: u8) {
        let address = self.get_rp(index);
        let value = self.memory.read(address);

        self.set_register(7, value);
        self.instruction_finished(1, 8);
    }

    fn load_a_from_hl_inc(&mut self) {
        let address = self.get_rp(2);
        let value = self.memory.read(address);

        self.set_register(7, value);
        self.set_rp(2, address.wrapping_add(1));
        self.instruction_finished(1, 8);
    }

    fn load_a_from_hl_dec(&mut self) {
        let address = self.get_rp(2);
        let value = self.memory.read(address);

        self.set_register(7, value);
        self.set_rp(2, address.wrapping_sub(1));
        self.instruction_finished(1, 8);
    }

    fn save_a_to_rp(&mut self, index: u8) {
        let address = self.get_rp(index);
        let value = self.get_register(7);

        self.memory.write(address, value, true);
        self.instruction_finished(1, 8);
    }

    fn save_a_to_hl_inc(&mut self) {
        let address = self.get_rp(2);
        let value = self.get_register(7);

        self.memory.write(address, value, true);
        self.set_rp(2, address.wrapping_add(1));
        self.instruction_finished(1, 8);
    }

    fn save_a_to_hl_dec(&mut self) {
        let address = self.get_rp(2);
        let value = self.get_register(7);

        self.memory.write(address, value, true);
        self.set_rp(2, address.wrapping_sub(1));
        self.instruction_finished(1, 8);
    }

    fn inc_rp(&mut self, index: u8) {
        let result = self.get_rp(index).wrapping_add(1);
        self.set_rp(index, result);
        self.instruction_finished(1, 8);
    }

    fn dec_rp(&mut self, index: u8) {
        let result = self.get_rp(index).wrapping_sub(1);
        self.set_rp(index, result);
        self.instruction_finished(1, 8);
    }

    fn inc_reg(&mut self, index: u8) {
        let result = self.get_register(index).wrapping_add(1);

        self.set_register(index, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf((result & 0x0F) == 0);
        self.instruction_finished(1, if index == 6 {12} else {4});
    }

    fn dec_reg(&mut self, index: u8) {
        let result = self.get_register(index).wrapping_sub(1);

        self.set_register(index, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf((result & 0x0F) == 0x0F);
        self.instruction_finished(1, if index == 6 {12} else {4});
    }

    // Load immediate 8-bit value into a register.
    fn load_imm_into_reg(&mut self, index: u8) {
        let value = self.memory.read(self.pc + 1);
        self.set_register(index, value);
        self.instruction_finished(2, if index == 6 {12} else {8});
    }

    fn rlca(&mut self) {
        let value = self.get_register(7);
        let carry = ((value >> 7) & 1) == 1;
        let result = value.rotate_left(1);

        self.set_register(7, result);
        self.cpu_flags.set_zf(false);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(carry);
        self.instruction_finished(1, 4);
    }

    fn rrca(&mut self) {
        let value = self.get_register(7);
        let carry = (value & 1) == 1;
        let result = value.rotate_right(1);

        self.set_register(7, result);
        self.cpu_flags.set_zf(false);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(carry);
        self.instruction_finished(1, 4);
    }

    fn rla(&mut self) {
        let value = self.get_register(7);
        let carry = self.cpu_flags.get_cf();
        let will_carry = ((value >> 7) & 1) == 1;
        let result = (value << 1) | carry;

        self.set_register(7, result);
        self.cpu_flags.set_zf(false);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(will_carry);
        self.instruction_finished(1, 4);
    }

    fn rra(&mut self) {
        let value = self.get_register(7);
        let carry = self.cpu_flags.get_cf();
        let will_carry = (value & 1) == 1;
        let result = (value >> 1) | (carry << 7);

        self.set_register(7, result);
        self.cpu_flags.set_zf(false);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(will_carry);
        self.instruction_finished(1, 4);
    }

    fn daa(&mut self) {
        // I'll implement this whenever I find a ROM (that's not a test) that needs it.
        error!("Unimplemented opcode DAA. Execution will continue, but things may break");
        self.instruction_finished(1, 4);
    }

    fn cpl(&mut self) {
        let result = !(self.get_register(7));
        self.set_register(7, result);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(true);
        self.instruction_finished(1, 4);
    }

    fn scf(&mut self) {
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(true);
        self.instruction_finished(1, 4);
    }

    fn ccf(&mut self) {
        let carry = if self.cpu_flags.get_cf() == 1 {false} else {true};
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(carry);
        self.instruction_finished(1, 4);
    }

    // Load an 8-bit register into another.
    fn load_reg_into_reg(&mut self, target: u8, source: u8) {
        let value = self.get_register(source);
        self.set_register(target, value);
        self.instruction_finished(1, if source == 6 || target == 6 {8} else {4});
    }

    fn halt(&mut self) {
        self.halted = true;
        self.instruction_finished(1, 4);
    }

    fn add(&mut self, index: u8) {
        let hf = (((self.get_register(7) & 0xF) + (self.get_register(index) & 0xF)) & 0x10) == 0x10;
        let result = self.get_register(7) as u16 + self.get_register(index) as u16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(hf);
        self.cpu_flags.set_cf(result > 0xFF);
        self.instruction_finished(1, if index == 6 {8} else {4});
    }

    fn adc(&mut self, index: u8) {
        let hf = (((self.get_register(7) & 0xF) + (self.get_register(index) & 0xF) + (self.cpu_flags.get_cf())) & 0x10) == 0x10;
        let result = self.get_register(7) as u16 + self.get_register(index) as u16 + self.cpu_flags.get_cf() as u16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(hf);
        self.cpu_flags.set_cf(result > 0xFF);
        self.instruction_finished(1, if index == 6 {8} else {4});
    }

    fn sub(&mut self, index: u8) {
        let hf = ((self.get_register(7) as i16 & 0xF) - (self.get_register(index) as i16 & 0xF)) < 0;
        let result = self.get_register(7) as i16 - self.get_register(index) as i16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(hf);
        self.cpu_flags.set_cf(result < 0);
        self.instruction_finished(1, if index == 6 {8} else {4});
    }

    fn sbc(&mut self, index: u8) {
        let hf = ((self.get_register(7) as i16 & 0xF) - (self.get_register(index) as i16 & 0xF) - self.cpu_flags.get_cf() as i16) < 0;
        let result = self.get_register(7) as i16 - self.get_register(index) as i16 - self.cpu_flags.get_cf() as i16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(hf);
        self.cpu_flags.set_cf(result < 0);
        self.instruction_finished(1, if index == 6 {8} else {4});
    }

    fn and(&mut self, index: u8) {
        let result = self.get_register(7) & self.get_register(index);

        self.set_register(7, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(true);
        self.cpu_flags.set_cf(false);
        self.instruction_finished(1, if index == 6 {8} else {4});
    }

    fn xor(&mut self, index: u8) {
        let result = self.get_register(7) ^ self.get_register(index);

        self.set_register(7, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(false);
        self.instruction_finished(1, if index == 6 {8} else {4});
    }

    fn or(&mut self, index: u8) {
        let result = self.get_register(7) | self.get_register(index);

        self.set_register(7, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(false);
        self.instruction_finished(1, if index == 6 {8} else {4});
    }

    fn cp(&mut self, index: u8) {
        let hf = ((self.get_register(7) as i16 & 0xF) - (self.get_register(index) as i16 & 0xF)) < 0;
        let values = (self.get_register(7), self.get_register(index));

        self.cpu_flags.set_zf(values.0 == values.1);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(hf);
        self.cpu_flags.set_cf(values.0 < values.1);
        self.instruction_finished(1, if index == 6 {8} else {4});
    }

    fn ret_cc(&mut self, condition: u8) {
        let ret = match condition {
            0 => self.cpu_flags.get_zf() == 0,
            1 => self.cpu_flags.get_zf() == 1,
            2 => self.cpu_flags.get_cf() == 0,
            3 => self.cpu_flags.get_cf() == 1,
            _ => panic!("Invalid return condition"),
        };

        if ret {
            self.ret();
        }
        else {
            self.instruction_finished(1, 8);
        }
    }

    fn save_a_to_ff_imm(&mut self) {
        let address = 0xFF00 + self.memory.read(self.pc + 1) as u16;
        let value = self.get_register(7);

        self.memory.write(address, value, true);
        self.instruction_finished(2, 12);
    }

    fn add_imm_to_sp(&mut self) {
        let value = self.memory.read(self.pc + 1) as i8;
        let result = self.get_rp(3).wrapping_add(value as u16);

        self.set_rp(3, result);
        self.cpu_flags.set_zf(false);
        self.cpu_flags.set_nf(false);
        // TODO: Proper flags
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(false);
        self.instruction_finished(2, 16);
    }

    // Load the value pointed by 0xFF00 + immediate value into A.
    fn load_a_from_ff_imm(&mut self) {
        let address = 0xFF00 + self.memory.read(self.pc + 1) as u16;
        let value = self.memory.read(address);

        self.set_register(7, value);
        self.instruction_finished(2, 12);
    }

    fn load_sp_imm_to_hl(&mut self) {
        let imm = self.memory.read(self.pc + 1) as i8;
        let result = self.get_rp(3).wrapping_add(imm as u16);

        self.set_rp(2, result);
        self.cpu_flags.set_zf(false);
        self.cpu_flags.set_nf(false);
        // TODO: Proper flags
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(false);
        self.instruction_finished(2, 12);
    }

    fn pop(&mut self, index: u8) {
        let value = self.stack_read();
        self.set_rp2(index, value);
        self.instruction_finished(1, 12);
    }

    fn ret(&mut self) {
        self.pc = self.stack_read();
        self.instruction_finished(0, 16);
    }

    fn reti(&mut self) {
        self.interrupts_enabled = true;
        self.ret();
    }

    fn jp_hl(&mut self) {
        self.pc = self.get_rp(2);
        self.instruction_finished(0, 4);
    }

    fn load_hl_to_sp(&mut self) {
        let hl = self.get_rp(2);
        self.set_rp(3, hl);
        self.instruction_finished(1, 8);
    }

    fn jp_cc(&mut self, condition: u8) {
        let jump = match condition {
            0 => self.cpu_flags.get_zf() == 0,
            1 => self.cpu_flags.get_zf() == 1,
            2 => self.cpu_flags.get_cf() == 0,
            3 => self.cpu_flags.get_cf() == 1,
            _ => panic!("Invalid jump condition"),
        };

        if jump {
            self.jp();
        }
        else {
            self.instruction_finished(3, 12);
        }
    }

    // Save the value of A into 0xFF00 + the value of C
    fn save_a_to_ff_c(&mut self) {
        let address = 0xFF00 + self.get_register(1) as u16;
        let value = self.get_register(7);

        self.memory.write(address, value, true);
        self.instruction_finished(1, 8);
    }

    // Save the value of A into address at immediate value.
    fn save_a_to_imm(&mut self) {
        let bytes = vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)];
        let value = self.get_register(7);

        self.memory.write(LittleEndian::read_u16(&bytes), value, true);
        self.instruction_finished(3, 16);
    }

    // Read address 0xFF00 + the value of C, and load the value into A.
    fn load_a_from_ff_c(&mut self) {
        let address = 0xFF00 + self.get_register(1) as u16;
        let value = self.memory.read(address);

        self.set_register(7, value);
        self.instruction_finished(1, 8);
    }

    // Load value from address at immediate value into A.
    fn load_a_from_imm(&mut self) {
        let bytes = vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)];
        let value = self.memory.read(LittleEndian::read_u16(&bytes));

        self.set_register(7, value);
        self.instruction_finished(3, 16);
    }

    fn jp(&mut self) {
        let bytes = vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)];
        let address = LittleEndian::read_u16(&bytes);

        self.pc = address;
        self.instruction_finished(0, 16);
    }

    fn di(&mut self) {
        self.interrupts_enabled = false;
        self.instruction_finished(1, 4);
    }

    fn ei(&mut self) {
        self.interrupts_enabled = true;
        self.instruction_finished(1, 4);
    }

    fn call_cc(&mut self, condition: u8) {
        let call = match condition {
            0 => self.cpu_flags.get_zf() == 0,
            1 => self.cpu_flags.get_zf() == 1,
            2 => self.cpu_flags.get_cf() == 0,
            3 => self.cpu_flags.get_cf() == 1,
            _ => panic!("Invalid call condition"),
        };

        if call {
            self.call();
        }
        else {
            self.instruction_finished(3, 12)
        }
    }

    fn push(&mut self, index: u8) {
        let reg = self.get_rp2(index);
        self.stack_write(reg);
        self.instruction_finished(1, 16);
    }

    fn call(&mut self) {
        let bytes = vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)];
        let target_address = LittleEndian::read_u16(&bytes);
        let ret_address = self.pc + 3;
        
        self.stack_write(ret_address);
        self.pc = target_address;
        self.instruction_finished(0, 24);
    }

    fn add_imm(&mut self) {
        let hf = (((self.get_register(7) & 0xF) + (self.memory.read(self.pc + 1) & 0xF)) & 0x10) == 0x10;
        let result = self.get_register(7) as u16 + self.memory.read(self.pc + 1) as u16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(hf);
        self.cpu_flags.set_cf(result > 0xFF);
        self.instruction_finished(2, 8);
    }

    fn adc_imm(&mut self) {
        let hf = (((self.get_register(7) & 0xF) + (self.memory.read(self.pc + 1) & 0xF) + (self.cpu_flags.get_cf())) & 0x10) == 0x10;
        let result = self.get_register(7) as u16 + self.memory.read(self.pc + 1) as u16 + self.cpu_flags.get_cf() as u16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(hf);
        self.cpu_flags.set_cf(result > 0xFF);
        self.instruction_finished(2, 8);
    }

    fn sub_imm(&mut self) {
        let hf = ((self.get_register(7) as i16 & 0xF) - (self.memory.read(self.pc + 1) as i16 & 0xF)) < 0;
        let result = self.get_register(7) as i16 - self.memory.read(self.pc + 1) as i16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(hf);
        self.cpu_flags.set_cf(result < 0);
        self.instruction_finished(2, 8);
    }

    fn sbc_imm(&mut self) {
        let hf = ((self.get_register(7) as i16 & 0xF) - (self.memory.read(self.pc + 1) as i16 & 0xF) - self.cpu_flags.get_cf() as i16) < 0;
        let result = self.get_register(7) as i16 - self.memory.read(self.pc + 1) as i16 - self.cpu_flags.get_cf() as i16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(hf);
        self.cpu_flags.set_cf(result < 0);
        self.instruction_finished(2, 8);
    }

    fn and_imm(&mut self) {
        let result = self.get_register(7) & self.memory.read(self.pc + 1);

        self.set_register(7, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(true);
        self.cpu_flags.set_cf(false);
        self.instruction_finished(2, 8);
    }

    fn xor_imm(&mut self) {
        let result = self.get_register(7) ^ self.memory.read(self.pc + 1);

        self.set_register(7, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(false);
        self.instruction_finished(2, 8);
    }

    fn or_imm(&mut self) {
        let result = self.get_register(7) | self.memory.read(self.pc + 1);

        self.set_register(7, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(false);
        self.instruction_finished(2, 8);
    }

    fn cp_imm(&mut self) {
        let hf = ((self.get_register(7) as i16 & 0xF) - (self.memory.read(self.pc + 1) as i16 & 0xF)) < 0;
        let values = (self.get_register(7), self.memory.read(self.pc + 1));

        self.cpu_flags.set_zf(values.0 == values.1);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(hf);
        self.cpu_flags.set_cf(values.0 < values.1);
        self.instruction_finished(2, 8);
    }

    fn rst(&mut self, offset: u8) {
        let target_address = 0x0000 + offset as u16;
        let ret_address = self.pc + 1;
        
        self.stack_write(ret_address);
        self.pc = target_address;
        self.instruction_finished(0, 16);
    }


    // Prefixed opcodes

    fn rlc(&mut self, index: u8) {
        let value = self.get_register(index);
        let carry = ((value >> 7) & 1) == 1;
        let result = value.rotate_left(1);
        
        self.set_register(index, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(carry);
        self.instruction_finished(2, if index == 6 {16} else {8});
    }

    fn rrc(&mut self, index: u8) {
        let value = self.get_register(index);
        let carry = (value & 1) == 1;
        let result = value.rotate_right(1);
        
        self.set_register(index, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(carry);
        self.instruction_finished(2, if index == 6 {16} else {8});
    }

    fn rl(&mut self, index: u8) {
        let value = self.get_register(index);
        let will_carry = ((value >> 7) & 1) == 1;
        let carry_value = self.cpu_flags.get_cf();
        let result = (value << 1) | carry_value;

        self.set_register(index, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(will_carry);
        self.instruction_finished(2, if index == 6 {16} else {8});
    }

    fn rr(&mut self, index: u8) {
        let value = self.get_register(index);
        let will_carry = (value & 1) == 1;
        let carry_value = self.cpu_flags.get_cf();
        let result = (value >> 1) | (carry_value << 7);

        self.set_register(index, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(will_carry);
        self.instruction_finished(2, if index == 6 {16} else {8});
    }

    fn sla(&mut self, index: u8) {
        let value = self.get_register(index);
        let shifted_bit = ((value >> 7) & 1) == 1;
        let result = value << 1;
        
        self.set_register(index, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(shifted_bit);
        self.instruction_finished(2, if index == 6 {16} else {8});
    }

    fn sra(&mut self, index: u8) {
        let value = self.get_register(index);
        let will_carry = (value & 1) == 1;
        let msb = (value >> 7) & 1;
        let result = (value >> 1) | (msb << 7);

        self.set_register(index, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(will_carry);
        self.instruction_finished(2, if index == 6 {16} else {8});
    }

    fn swap(&mut self, index: u8) {
        let value = self.get_register(index);
        let result = ((value & 0xF0) >> 4) | ((value & 0xF) << 4);

        self.set_register(index, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(false);
        self.instruction_finished(2, if index == 6 {16} else {8});
    }

    fn srl(&mut self, index: u8) {
        let value = self.get_register(index);
        let will_carry = (value & 1) == 1;
        let result = value >> 1;

        self.set_register(index, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(will_carry);
        self.instruction_finished(2, if index == 6 {16} else {8});
    }

    fn bit(&mut self, index: u8, bit: u8) {
        let value = self.get_register(index);
        let result = ((value >> bit) & 1) == 1;

        self.cpu_flags.set_zf(!result);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(true);
        self.instruction_finished(2, if index == 6 {16} else {8});
    }

    fn res(&mut self, index: u8, bit: u8) {
        let value = self.get_register(index);
        self.set_register(index, value & !(1 << bit));
        self.instruction_finished(2, if index == 6 {16} else {8});
    }

    fn set(&mut self, index: u8, bit: u8) {
        let value = self.get_register(index);
        self.set_register(index, value | (1 << bit));
        self.instruction_finished(2, if index == 6 {16} else {8});
    }
}