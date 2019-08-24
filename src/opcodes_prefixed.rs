use super::utils;

use super::cpu;
use super::cpu::Memory;
use super::cpu::CpuState;

use super::register::CpuReg;
use super::register::Register;
use super::register::PcTrait;
use super::register::CycleCounter;

pub fn run_prefixed_instruction(current_state: &mut CpuState, memory: &mut Memory, opcode: u8) {

    match opcode {

        0x10 => instruction_finished(rl_lb(&mut current_state.bc, &mut current_state.af), current_state),
        0x11 => instruction_finished(rl_rb(&mut current_state.bc, &mut current_state.af), current_state),
        0x12 => instruction_finished(rl_lb(&mut current_state.de, &mut current_state.af), current_state),
        0x13 => instruction_finished(rl_rb(&mut current_state.de, &mut current_state.af), current_state),
        0x14 => instruction_finished(rl_lb(&mut current_state.hl, &mut current_state.af), current_state),
        0x15 => instruction_finished(rl_rb(&mut current_state.hl, &mut current_state.af), current_state),
        
        0x40 => instruction_finished(bit_lb(&mut current_state.bc, 0, &mut current_state.af), current_state),
        0x41 => instruction_finished(bit_rb(&mut current_state.bc, 0, &mut current_state.af), current_state),
        0x42 => instruction_finished(bit_lb(&mut current_state.de, 0, &mut current_state.af), current_state),
        0x43 => instruction_finished(bit_rb(&mut current_state.de, 0, &mut current_state.af), current_state),
        0x44 => instruction_finished(bit_lb(&mut current_state.hl, 0, &mut current_state.af), current_state),
        0x45 => instruction_finished(bit_rb(&mut current_state.hl, 0, &mut current_state.af), current_state),
        0x46 => instruction_finished(bit_hl(0, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x48 => instruction_finished(bit_lb(&mut current_state.bc, 1, &mut current_state.af), current_state),
        0x49 => instruction_finished(bit_rb(&mut current_state.bc, 1, &mut current_state.af), current_state),
        0x4A => instruction_finished(bit_lb(&mut current_state.de, 1, &mut current_state.af), current_state),
        0x4B => instruction_finished(bit_rb(&mut current_state.de, 1, &mut current_state.af), current_state),
        0x4C => instruction_finished(bit_lb(&mut current_state.hl, 1, &mut current_state.af), current_state),
        0x4D => instruction_finished(bit_rb(&mut current_state.hl, 1, &mut current_state.af), current_state),
        0x4E => instruction_finished(bit_hl(1, &mut current_state.af, &mut current_state.hl, memory), current_state),

        0x50 => instruction_finished(bit_lb(&mut current_state.bc, 2, &mut current_state.af), current_state),
        0x51 => instruction_finished(bit_rb(&mut current_state.bc, 2, &mut current_state.af), current_state),
        0x52 => instruction_finished(bit_lb(&mut current_state.de, 2, &mut current_state.af), current_state),
        0x53 => instruction_finished(bit_rb(&mut current_state.de, 2, &mut current_state.af), current_state),
        0x54 => instruction_finished(bit_lb(&mut current_state.hl, 2, &mut current_state.af), current_state),
        0x55 => instruction_finished(bit_rb(&mut current_state.hl, 2, &mut current_state.af), current_state),
        0x56 => instruction_finished(bit_hl(2, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x58 => instruction_finished(bit_lb(&mut current_state.bc, 3, &mut current_state.af), current_state),
        0x59 => instruction_finished(bit_rb(&mut current_state.bc, 3, &mut current_state.af), current_state),
        0x5A => instruction_finished(bit_lb(&mut current_state.de, 3, &mut current_state.af), current_state),
        0x5B => instruction_finished(bit_rb(&mut current_state.de, 3, &mut current_state.af), current_state),
        0x5C => instruction_finished(bit_lb(&mut current_state.hl, 3, &mut current_state.af), current_state),
        0x5D => instruction_finished(bit_rb(&mut current_state.hl, 3, &mut current_state.af), current_state),
        0x5E => instruction_finished(bit_hl(3, &mut current_state.af, &mut current_state.hl, memory), current_state),

        0x60 => instruction_finished(bit_lb(&mut current_state.bc, 4, &mut current_state.af), current_state),
        0x61 => instruction_finished(bit_rb(&mut current_state.bc, 4, &mut current_state.af), current_state),
        0x62 => instruction_finished(bit_lb(&mut current_state.de, 4, &mut current_state.af), current_state),
        0x63 => instruction_finished(bit_rb(&mut current_state.de, 4, &mut current_state.af), current_state),
        0x64 => instruction_finished(bit_lb(&mut current_state.hl, 4, &mut current_state.af), current_state),
        0x65 => instruction_finished(bit_rb(&mut current_state.hl, 4, &mut current_state.af), current_state),
        0x66 => instruction_finished(bit_hl(4, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x68 => instruction_finished(bit_lb(&mut current_state.bc, 5, &mut current_state.af), current_state),
        0x69 => instruction_finished(bit_rb(&mut current_state.bc, 5, &mut current_state.af), current_state),
        0x6A => instruction_finished(bit_lb(&mut current_state.de, 5, &mut current_state.af), current_state),
        0x6B => instruction_finished(bit_rb(&mut current_state.de, 5, &mut current_state.af), current_state),
        0x6C => instruction_finished(bit_lb(&mut current_state.hl, 5, &mut current_state.af), current_state),
        0x6D => instruction_finished(bit_rb(&mut current_state.hl, 5, &mut current_state.af), current_state),
        0x6E => instruction_finished(bit_hl(5, &mut current_state.af, &mut current_state.hl, memory), current_state),

        0x70 => instruction_finished(bit_lb(&mut current_state.bc, 6, &mut current_state.af), current_state),
        0x71 => instruction_finished(bit_rb(&mut current_state.bc, 6, &mut current_state.af), current_state),
        0x72 => instruction_finished(bit_lb(&mut current_state.de, 6, &mut current_state.af), current_state),
        0x73 => instruction_finished(bit_rb(&mut current_state.de, 6, &mut current_state.af), current_state),
        0x74 => instruction_finished(bit_lb(&mut current_state.hl, 6, &mut current_state.af), current_state),
        0x75 => instruction_finished(bit_rb(&mut current_state.hl, 6, &mut current_state.af), current_state),
        0x76 => instruction_finished(bit_hl(6, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x78 => instruction_finished(bit_lb(&mut current_state.bc, 7, &mut current_state.af), current_state),
        0x79 => instruction_finished(bit_rb(&mut current_state.bc, 7, &mut current_state.af), current_state),
        0x7A => instruction_finished(bit_lb(&mut current_state.de, 7, &mut current_state.af), current_state),
        0x7B => instruction_finished(bit_rb(&mut current_state.de, 7, &mut current_state.af), current_state),
        0x7C => instruction_finished(bit_lb(&mut current_state.hl, 7, &mut current_state.af), current_state),
        0x7D => instruction_finished(bit_rb(&mut current_state.hl, 7, &mut current_state.af), current_state),
        0x7E => instruction_finished(bit_hl(7, &mut current_state.af, &mut current_state.hl, memory), current_state),

        0x80 => instruction_finished(res_lb(&mut current_state.bc, 0), current_state),
        0x81 => instruction_finished(res_rb(&mut current_state.bc, 0), current_state),
        0x82 => instruction_finished(res_lb(&mut current_state.de, 0), current_state),
        0x83 => instruction_finished(res_rb(&mut current_state.de, 0), current_state),
        0x84 => instruction_finished(res_lb(&mut current_state.hl, 0), current_state),
        0x85 => instruction_finished(res_rb(&mut current_state.hl, 0), current_state),
        0x86 => instruction_finished(res_hl(0, &mut current_state.hl, memory), current_state),
        0x87 => instruction_finished(res_lb(&mut current_state.af, 1), current_state),
        0x88 => instruction_finished(res_lb(&mut current_state.bc, 1), current_state),
        0x89 => instruction_finished(res_rb(&mut current_state.bc, 1), current_state),
        0x8A => instruction_finished(res_lb(&mut current_state.de, 1), current_state),
        0x8B => instruction_finished(res_rb(&mut current_state.de, 1), current_state),
        0x8C => instruction_finished(res_lb(&mut current_state.hl, 1), current_state),
        0x8D => instruction_finished(res_rb(&mut current_state.hl, 1), current_state),
        0x8E => instruction_finished(res_hl(1, &mut current_state.hl, memory), current_state),

        0x90 => instruction_finished(res_lb(&mut current_state.bc, 2), current_state),
        0x91 => instruction_finished(res_rb(&mut current_state.bc, 2), current_state),
        0x92 => instruction_finished(res_lb(&mut current_state.de, 2), current_state),
        0x93 => instruction_finished(res_rb(&mut current_state.de, 2), current_state),
        0x94 => instruction_finished(res_lb(&mut current_state.hl, 2), current_state),
        0x95 => instruction_finished(res_rb(&mut current_state.hl, 2), current_state),
        0x96 => instruction_finished(res_hl(2, &mut current_state.hl, memory), current_state),
        0x97 => instruction_finished(res_lb(&mut current_state.af, 3), current_state),
        0x98 => instruction_finished(res_lb(&mut current_state.bc, 3), current_state),
        0x99 => instruction_finished(res_rb(&mut current_state.bc, 3), current_state),
        0x9A => instruction_finished(res_lb(&mut current_state.de, 3), current_state),
        0x9B => instruction_finished(res_rb(&mut current_state.de, 3), current_state),
        0x9C => instruction_finished(res_lb(&mut current_state.hl, 3), current_state),
        0x9D => instruction_finished(res_rb(&mut current_state.hl, 3), current_state),
        0x9E => instruction_finished(res_hl(3, &mut current_state.hl, memory), current_state),

        0xA0 => instruction_finished(res_lb(&mut current_state.bc, 4), current_state),
        0xA1 => instruction_finished(res_rb(&mut current_state.bc, 4), current_state),
        0xA2 => instruction_finished(res_lb(&mut current_state.de, 4), current_state),
        0xA3 => instruction_finished(res_rb(&mut current_state.de, 4), current_state),
        0xA4 => instruction_finished(res_lb(&mut current_state.hl, 4), current_state),
        0xA5 => instruction_finished(res_rb(&mut current_state.hl, 4), current_state),
        0xA6 => instruction_finished(res_hl(4, &mut current_state.hl, memory), current_state),
        0xA7 => instruction_finished(res_lb(&mut current_state.af, 5), current_state),
        0xA8 => instruction_finished(res_lb(&mut current_state.bc, 5), current_state),
        0xA9 => instruction_finished(res_rb(&mut current_state.bc, 5), current_state),
        0xAA => instruction_finished(res_lb(&mut current_state.de, 5), current_state),
        0xAB => instruction_finished(res_rb(&mut current_state.de, 5), current_state),
        0xAC => instruction_finished(res_lb(&mut current_state.hl, 5), current_state),
        0xAD => instruction_finished(res_rb(&mut current_state.hl, 5), current_state),
        0xAE => instruction_finished(res_hl(5, &mut current_state.hl, memory), current_state),

        0xB0 => instruction_finished(res_lb(&mut current_state.bc, 6), current_state),
        0xB1 => instruction_finished(res_rb(&mut current_state.bc, 6), current_state),
        0xB2 => instruction_finished(res_lb(&mut current_state.de, 6), current_state),
        0xB3 => instruction_finished(res_rb(&mut current_state.de, 6), current_state),
        0xB4 => instruction_finished(res_lb(&mut current_state.hl, 6), current_state),
        0xB5 => instruction_finished(res_rb(&mut current_state.hl, 6), current_state),
        0xB6 => instruction_finished(res_hl(6, &mut current_state.hl, memory), current_state),
        0xB7 => instruction_finished(res_lb(&mut current_state.af, 7), current_state),
        0xB8 => instruction_finished(res_lb(&mut current_state.bc, 7), current_state),
        0xB9 => instruction_finished(res_rb(&mut current_state.bc, 7), current_state),
        0xBA => instruction_finished(res_lb(&mut current_state.de, 7), current_state),
        0xBB => instruction_finished(res_rb(&mut current_state.de, 7), current_state),
        0xBC => instruction_finished(res_lb(&mut current_state.hl, 7), current_state),
        0xBD => instruction_finished(res_rb(&mut current_state.hl, 7), current_state),
        0xBE => instruction_finished(res_hl(7, &mut current_state.hl, memory), current_state),

        0xC0 => instruction_finished(set_lb(&mut current_state.bc, 0), current_state),
        0xC1 => instruction_finished(set_rb(&mut current_state.bc, 0), current_state),
        0xC2 => instruction_finished(set_lb(&mut current_state.de, 0), current_state),
        0xC3 => instruction_finished(set_rb(&mut current_state.de, 0), current_state),
        0xC4 => instruction_finished(set_lb(&mut current_state.hl, 0), current_state),
        0xC5 => instruction_finished(set_rb(&mut current_state.hl, 0), current_state),
        0xC6 => instruction_finished(set_hl(0, &mut current_state.hl, memory), current_state),
        0xC7 => instruction_finished(set_lb(&mut current_state.af, 1), current_state),
        0xC8 => instruction_finished(set_lb(&mut current_state.bc, 1), current_state),
        0xC9 => instruction_finished(set_rb(&mut current_state.bc, 1), current_state),
        0xCA => instruction_finished(set_lb(&mut current_state.de, 1), current_state),
        0xCB => instruction_finished(set_rb(&mut current_state.de, 1), current_state),
        0xCC => instruction_finished(set_lb(&mut current_state.hl, 1), current_state),
        0xCD => instruction_finished(set_rb(&mut current_state.hl, 1), current_state),
        0xCE => instruction_finished(set_hl(1, &mut current_state.hl, memory), current_state),

        0xD0 => instruction_finished(set_lb(&mut current_state.bc, 2), current_state),
        0xD1 => instruction_finished(set_rb(&mut current_state.bc, 2), current_state),
        0xD2 => instruction_finished(set_lb(&mut current_state.de, 2), current_state),
        0xD3 => instruction_finished(set_rb(&mut current_state.de, 2), current_state),
        0xD4 => instruction_finished(set_lb(&mut current_state.hl, 2), current_state),
        0xD5 => instruction_finished(set_rb(&mut current_state.hl, 2), current_state),
        0xD6 => instruction_finished(set_hl(2, &mut current_state.hl, memory), current_state),
        0xD7 => instruction_finished(set_lb(&mut current_state.af, 3), current_state),
        0xD8 => instruction_finished(set_lb(&mut current_state.bc, 3), current_state),
        0xD9 => instruction_finished(set_rb(&mut current_state.bc, 3), current_state),
        0xDA => instruction_finished(set_lb(&mut current_state.de, 3), current_state),
        0xDB => instruction_finished(set_rb(&mut current_state.de, 3), current_state),
        0xDC => instruction_finished(set_lb(&mut current_state.hl, 3), current_state),
        0xDD => instruction_finished(set_rb(&mut current_state.hl, 3), current_state),
        0xDE => instruction_finished(set_hl(3, &mut current_state.hl, memory), current_state),

        0xE0 => instruction_finished(set_lb(&mut current_state.bc, 4), current_state),
        0xE1 => instruction_finished(set_rb(&mut current_state.bc, 4), current_state),
        0xE2 => instruction_finished(set_lb(&mut current_state.de, 4), current_state),
        0xE3 => instruction_finished(set_rb(&mut current_state.de, 4), current_state),
        0xE4 => instruction_finished(set_lb(&mut current_state.hl, 4), current_state),
        0xE5 => instruction_finished(set_rb(&mut current_state.hl, 4), current_state),
        0xE6 => instruction_finished(set_hl(4, &mut current_state.hl, memory), current_state),
        0xE7 => instruction_finished(set_lb(&mut current_state.af, 5), current_state),
        0xE8 => instruction_finished(set_lb(&mut current_state.bc, 5), current_state),
        0xE9 => instruction_finished(set_rb(&mut current_state.bc, 5), current_state),
        0xEA => instruction_finished(set_lb(&mut current_state.de, 5), current_state),
        0xEB => instruction_finished(set_rb(&mut current_state.de, 5), current_state),
        0xEC => instruction_finished(set_lb(&mut current_state.hl, 5), current_state),
        0xED => instruction_finished(set_rb(&mut current_state.hl, 5), current_state),
        0xEE => instruction_finished(set_hl(5, &mut current_state.hl, memory), current_state),

        0xF0 => instruction_finished(set_lb(&mut current_state.bc, 6), current_state),
        0xF1 => instruction_finished(set_rb(&mut current_state.bc, 6), current_state),
        0xF2 => instruction_finished(set_lb(&mut current_state.de, 6), current_state),
        0xF3 => instruction_finished(set_rb(&mut current_state.de, 6), current_state),
        0xF4 => instruction_finished(set_lb(&mut current_state.hl, 6), current_state),
        0xF5 => instruction_finished(set_rb(&mut current_state.hl, 6), current_state),
        0xF6 => instruction_finished(set_hl(6, &mut current_state.hl, memory), current_state),
        0xF7 => instruction_finished(set_lb(&mut current_state.af, 7), current_state),
        0xF8 => instruction_finished(set_lb(&mut current_state.bc, 7), current_state),
        0xF9 => instruction_finished(set_rb(&mut current_state.bc, 7), current_state),
        0xFA => instruction_finished(set_lb(&mut current_state.de, 7), current_state),
        0xFB => instruction_finished(set_rb(&mut current_state.de, 7), current_state),
        0xFC => instruction_finished(set_lb(&mut current_state.hl, 7), current_state),
        0xFD => instruction_finished(set_rb(&mut current_state.hl, 7), current_state),
        0xFE => instruction_finished(set_hl(7, &mut current_state.hl, memory), current_state),
        
        _ => { 
            println!("Tried to run unimplemented prefixed opcode 0x{} at PC {}", format!("{:X}", opcode), format!("{:X}", current_state.pc.get()));
            current_state.should_execute = false;
        }
    }
}

fn instruction_finished(values: (u16, u32), state: &mut CpuState) {

    state.pc.add(values.0); state.cycles.add(values.1);
}

fn rl_lb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {

    let mut value = reg.get_register_lb();
    let carry: u8;
    if utils::check_bit(reg.get_register_rb(), 7) {carry = 1}
    else {carry = 0}
    utils::set_cf(utils::check_bit(value, 7), af);
    value = value << 1;
    reg.set_register_lb(value | carry);
    utils::set_hf(false, af);
    utils::set_nf(false, af);
    utils::set_zf(false, af);
    (2, 4)
}

fn rl_rb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {

    let mut value = reg.get_register_rb();
    let carry: u8;
    if utils::check_bit(reg.get_register_rb(), 7) {carry = 1}
    else {carry = 0}
    utils::set_cf(utils::check_bit(value, 7), af);
    value = value << 1;
    reg.set_register_rb(value | carry);
    utils::set_hf(false, af);
    utils::set_nf(false, af);
    utils::set_zf(false, af);
    (2, 4)
}

fn bit_lb(reg: &mut CpuReg, bit: u8, af: &mut CpuReg) -> (u16, u32) {

    let result = utils::check_bit(reg.get_register_lb(), bit);
    utils::set_zf(!result, af); utils::set_nf(false, af);
    utils::set_hf(true, af);
    (2, 8)
}

fn bit_rb(reg: &mut CpuReg, bit: u8, af: &mut CpuReg) -> (u16, u32) {

    let result = utils::check_bit(reg.get_register_rb(), bit);
    utils::set_zf(!result, af); utils::set_nf(false, af);
    utils::set_hf(true, af);
    (2, 8)
}

fn bit_hl(bit: u8, af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let result = utils::check_bit(cpu::memory_read_u8(&hl.get_register(), memory), bit);
    utils::set_zf(!result, af); utils::set_nf(false, af);
    utils::set_hf(true, af);
    (2, 16)
}

fn res_lb(reg: &mut CpuReg, bit: u8) -> (u16, u32) {

    let result = utils::reset_bit_u8(reg.get_register_lb(), bit);
    reg.set_register_lb(result);
    (2, 8)
}

fn res_rb(reg: &mut CpuReg, bit: u8) -> (u16, u32) {

    let result = utils::reset_bit_u8(reg.get_register_rb(), bit);
    reg.set_register_rb(result);
    (2, 8)
}

fn res_hl(bit: u8, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let result = utils::reset_bit_u8(cpu::memory_read_u8(&hl.get_register(), memory), bit);
    cpu::memory_write(hl.get_register(), result, memory);
    (2, 16)
}

fn set_lb(reg: &mut CpuReg, bit: u8) -> (u16, u32) {
    
    let result = utils::set_bit_u8(reg.get_register_lb(), bit);
    reg.set_register_lb(result);
    (2, 8)
}

fn set_rb(reg: &mut CpuReg, bit: u8) -> (u16, u32) {
    
    let result = utils::set_bit_u8(reg.get_register_rb(), bit);
    reg.set_register_rb(result);
    (2, 8)
}

fn set_hl(bit: u8, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let result = utils::set_bit_u8(cpu::memory_read_u8(&hl.get_register(), memory), bit);
    cpu::memory_write(hl.get_register(), result, memory);
    (2, 16)
}