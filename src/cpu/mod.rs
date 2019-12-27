use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::sync::atomic::{AtomicU16, Ordering};

use byteorder::{ByteOrder, LittleEndian};

use super::timer::TimerModule;
use super::emulator::InputEvent;
use super::memory::{CpuMemory, SharedMemory};

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
    pub registers: Vec<Register>,
    pub cpu_flags: FlagsRegister,

    pub pc: u16,
    pub sp: u16,
    pub cycles: Arc<AtomicU16>,

    pub halted: bool,
    pub stopped: bool,
    pub interrupts_enabled: bool,

    pub timer: TimerModule,

    pub memory: CpuMemory,
    pub shared_memory: Arc<SharedMemory>,

    pub input_receiver: Receiver<InputEvent>,
}

impl Cpu {
    pub fn new(cpu_mem: CpuMemory, shared: Arc<SharedMemory>, cycles: Arc<AtomicU16>, input: Receiver<InputEvent>, run_bootrom: bool) -> Cpu {
        
        let timer_cycles = Arc::clone(&cycles);
        
        Cpu {
            registers: vec![Register::new(); 8],
            cpu_flags: FlagsRegister::new(),
            
            pc: if run_bootrom {0x0} else {0x100},
            sp: 0,
            cycles: cycles,

            halted: false,
            stopped: false,
            interrupts_enabled: false,

            timer: TimerModule::new(timer_cycles, Arc::clone(&shared)),

            memory: cpu_mem,
            shared_memory: shared,

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
            self.memory.write(address, value);
            return;
        }

        self.registers[index as usize].set(value);
    }

    pub fn execution_loop(&mut self) {

        loop {
            self.update_input();
            self.check_interrupts();
            self.run_instruction();
            self.timer.timer_cycle();
        }
    }

    fn update_input(&mut self) {

    }

    fn check_interrupts(&mut self) {
        
    }

    fn run_instruction(&mut self) {
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
                        6 => self.di(),
                        7 => self.ei(),
                        _ => panic!("Invalid operation"),
                    }
                }
                else if instruction.z == 4 {
                    match instruction.y {
                        0|1|2|3 => self.call_cc(instruction.y),
                        _ => panic!("Invalid operation"),
                    }
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

        self.memory.write(address, low);
        self.memory.write(address + 1, hi);
        self.instruction_finished(3, 20);
    }

    fn stop(&mut self) {
        panic!("Unimplemented instruction: STOP");
    }

    fn jr(&mut self) {
        let value = self.memory.read(self.pc + 1) as i8;
        self.pc = self.pc.wrapping_add(value as u16 + 2);
        self.cycles.fetch_add(8, Ordering::Relaxed);
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
            self.instruction_finished(2, 12);
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
        let result = hl + value;

        self.set_rp(2, result as u16);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
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

        self.memory.write(address, value);
        self.instruction_finished(1, 8);
    }

    fn save_a_to_hl_inc(&mut self) {
        let address = self.get_rp(2);
        let value = self.get_register(7);

        self.memory.write(address, value);
        self.set_rp(2, address.wrapping_add(1));
        self.instruction_finished(1, 8);
    }

    fn save_a_to_hl_dec(&mut self) {
        let address = self.get_rp(2);
        let value = self.get_register(7);

        self.memory.write(address, value);
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
        self.cpu_flags.set_hf(false);
        self.instruction_finished(1, if index == 6 {12} else {4});
    }

    fn dec_reg(&mut self, index: u8) {
        let result = self.get_register(index).wrapping_sub(1);

        self.set_register(index, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(false);
        self.instruction_finished(1, if index == 6 {12} else {4});
    }

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
        self.instruction_finished(2, 8);
    }

    fn rla(&mut self) {
        let carry = self.cpu_flags.get_cf();
        let value = self.get_register(7);
        let result = (value << 1) | carry;

        self.set_register(7, result);
        self.cpu_flags.set_zf(false);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(((result >> 7) & 1) == 1);
        self.instruction_finished(1, 4);
    }

    fn rra(&mut self) {
        let value = self.get_register(7);
        let carry = self.cpu_flags.get_cf();
        let will_carry = (value & 1) == 1;
        let result = (value >> 1) | (carry << 7);

        self.set_register(7, result);
        self.cpu_flags.set_zf(result == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(will_carry);
        self.instruction_finished(1, 4);
    }

    fn daa(&mut self) {
        // I'll implement this whenever I find a ROM (that's not a test) that needs it.
        panic!("Unimplemented instruction: DAA");
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

    fn load_reg_into_reg(&mut self, target: u8, source: u8) {
        let value = self.get_register(source);
        self.set_register(target, value);
        self.instruction_finished(1, if source == 6 || target == 6 {8} else {4});
    }

    fn halt(&mut self) {
        panic!("Unimplemented instruction: HALT");
    }

    fn add(&mut self, index: u8) {
        let result = self.get_register(7) as u16 + self.get_register(index) as u16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(result > 0xFF);
        self.instruction_finished(1, if index == 6 {8} else {4});
    }

    fn adc(&mut self, index: u8) {
        let result = self.get_register(7) as u16 + self.get_register(index) as u16 + self.cpu_flags.get_cf() as u16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(result > 0xFF);
        self.instruction_finished(1, if index == 6 {8} else {4});
    }

    fn sub(&mut self, index: u8) {
        let result = self.get_register(7) as i16 - self.get_register(index) as i16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(result < 0);
        self.instruction_finished(1, if index == 6 {8} else {4});
    }

    fn sbc(&mut self, index: u8) {
        let result = self.get_register(7) as i16 - self.get_register(index) as i16 - self.cpu_flags.get_cf() as i16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(false);
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
        let values = (self.get_register(7), self.get_register(index));
        self.cpu_flags.set_zf(values.0 == values.1);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(false);
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

        self.memory.write(address, value);
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

    fn load_a_from_ff_imm(&mut self) {
        let address = 0xFF00 + self.memory.read(self.pc + 1) as u16;
        let value = self.memory.read(address);

        self.set_register(7, value);
        self.instruction_finished(2, 12);
    }

    fn load_sp_imm_to_hl(&mut self) {
        let imm = self.memory.read(self.pc + 1) as i8;
        let sp_result = self.get_rp(3).wrapping_add(imm as u16);
        let result = self.get_rp(2).wrapping_add(sp_result);

        self.set_rp(2, result);
        self.cpu_flags.set_zf(false);
        self.cpu_flags.set_nf(false);
        // TODO: Proper flags
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(false);
        self.instruction_finished(2, 16);
    }

    fn pop(&mut self, index: u8) {
        let mut bytes = vec![0; 2];
        let sp = self.get_rp(3);

        bytes[0] = self.memory.read(sp);
        bytes[1] = self.memory.read(sp + 1);
        self.set_rp(3, sp.wrapping_add(2));

        let value = LittleEndian::read_u16(&bytes);
        self.set_rp2(index, value);
        self.instruction_finished(1, 12);
    }

    fn ret(&mut self) {
        let mut bytes = vec![0; 2];
        let sp = self.get_rp(3);

        bytes[0] = self.memory.read(sp);
        bytes[1] = self.memory.read(sp - 1);
        self.set_rp(3, sp.wrapping_sub(2));

        let value = LittleEndian::read_u16(&bytes);
        self.pc = value;
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
        let sp = self.get_rp(3);
        self.set_rp(2, sp);
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
            self.instruction_finished(2, 12);
        }
    }

    fn save_a_to_ff_c(&mut self) {
        let address = 0xFF00 + self.get_register(1) as u16;
        let value = self.get_register(7);

        self.memory.write(address, value);
        self.instruction_finished(1, 8);
    }

    fn save_a_to_imm(&mut self) {
        let bytes = vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)];
        let value = self.get_register(7);

        self.memory.write(LittleEndian::read_u16(&bytes), value);
        self.instruction_finished(3, 16);
    }

    fn load_a_from_ff_c(&mut self) {
        let address = 0xFF00 + self.get_register(1) as u16;
        let value = self.memory.read(address);

        self.set_register(7, value);
        self.instruction_finished(1, 8);
    }

    fn load_a_from_imm(&mut self) {
        let address = 0xFF00 + self.memory.read(self.pc + 1) as u16;
        let value = self.memory.read(address);

        self.set_register(7, value);
        self.instruction_finished(3, 16);
    }

    fn jp(&mut self) {
        let bytes = vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)];
        let address = LittleEndian::read_u16(&bytes);

        self.pc = address;
        self.instruction_finished(1, 16);
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
        let sp = self.get_rp(3);
        let hi = (reg >> 8) as u8;
        let low = reg as u8;

        self.memory.write(sp - 1, hi);
        self.memory.write(sp - 2, low);
        self.set_rp(3, sp - 2);
        self.instruction_finished(1, 16);
    }

    fn call(&mut self) {
        let bytes = vec![self.memory.read(self.pc + 1), self.memory.read(self.pc + 2)];
        let target_address = LittleEndian::read_u16(&bytes);
        let ret_address = self.pc + 3;
        let sp = self.get_rp(3);
        let hi = (ret_address >> 8) as u8;
        let low = ret_address as u8;

        self.memory.write(sp, hi);
        self.memory.write(sp - 1, low);
        self.set_rp(3, sp - 2);
        
        self.pc = target_address;
        self.instruction_finished(0, 24);
    }

    fn add_imm(&mut self) {
        let result = self.get_register(7) as u16 + self.memory.read(self.pc + 1) as u16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(result > 0xFF);
        self.instruction_finished(2, 8);
    }

    fn adc_imm(&mut self) {
        let result = self.get_register(7) as u16 + self.memory.read(self.pc + 1) as u16 + self.cpu_flags.get_cf() as u16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(false);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(result > 0xFF);
        self.instruction_finished(2, 8);
    }

    fn sub_imm(&mut self) {
        let result = self.get_register(7) as i16 - self.memory.read(self.pc + 1) as i16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(result < 0);
        self.instruction_finished(2, 8);
    }

    fn sbc_imm(&mut self) {
        let result = self.get_register(7) as i16 - self.memory.read(self.pc + 1) as i16 - self.cpu_flags.get_cf() as i16;

        self.set_register(7, result as u8);
        self.cpu_flags.set_zf(result as u8 == 0);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(false);
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
        let values = (self.get_register(7), self.memory.read(self.pc + 1));
        self.cpu_flags.set_zf(values.0 == values.1);
        self.cpu_flags.set_nf(true);
        self.cpu_flags.set_hf(false);
        self.cpu_flags.set_cf(values.0 < values.1);
        self.instruction_finished(2, 8);
    }

    fn rst(&mut self, offset: u8) {
        let target_address = offset as u16;
        let ret_address = self.pc + 1;
        let sp = self.get_rp(3);
        let hi = (ret_address >> 8) as u8;
        let low = ret_address as u8;

        self.memory.write(sp, hi);
        self.memory.write(sp - 1, low);
        self.set_rp(3, sp - 2);
        
        self.pc = target_address;
        self.instruction_finished(0, 16);
    }



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