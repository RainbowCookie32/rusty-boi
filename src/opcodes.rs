use byteorder::{ByteOrder, LittleEndian};

use super::utils;

use super::cpu;
use super::cpu::Memory;
use super::cpu::CpuState;
use super::cpu::CycleResult;

use super::register::CpuReg;
use super::register::Register;
use super::register::PcTrait;
use super::register::CycleCounter;

pub enum JumpCondition {

    ZSet,
    ZNotSet,
    CSet,
    CNotSet,
}

pub fn run_instruction(current_state: &mut CpuState, memory: &mut Memory, opcode: u8) -> CycleResult {

    let mut result = CycleResult::Success;

    println!("Running opcode 0x{} at PC {}", format!("{:X}", opcode), format!("{:X}", current_state.pc.get()));
    match opcode {

        0x00 => nop(current_state),
        0x01 => instruction_finished(ld_imm_into_full(&mut current_state.bc, memory, &current_state.pc.get()), current_state),
        0x02 => instruction_finished(save_a_to_full(&mut current_state.af, &mut current_state.bc, memory), current_state),
        0x03 => instruction_finished(increment_full(&mut current_state.bc), current_state),
        0x04 => instruction_finished(increment_lb(&mut current_state.bc, &mut current_state.af), current_state),
        0x05 => instruction_finished(decrement_lb(&mut current_state.bc, &mut current_state.af), current_state),
        0x06 => instruction_finished(ld_imm_into_hi(&mut current_state.bc, memory, &current_state.pc.get()), current_state),
        0x09 => instruction_finished(add_full(&mut current_state.hl, &mut current_state.bc, &mut current_state.af), current_state),
        0x0A => instruction_finished(ld_hi_from_full(&mut current_state.af, &mut current_state.bc, memory), current_state),
        0x0B => instruction_finished(decrement_full(&mut current_state.bc), current_state),
        0x0C => instruction_finished(increment_rb(&mut current_state.bc, &mut current_state.af), current_state),
        0x0D => instruction_finished(decrement_rb(&mut current_state.bc, &mut current_state.af), current_state),
        0x0E => instruction_finished(ld_imm_into_low(&mut current_state.bc, memory, &current_state.pc.get()), current_state),

        0x11 => instruction_finished(ld_imm_into_full(&mut current_state.de, memory, &current_state.pc.get()), current_state),
        0x12 => instruction_finished(save_a_to_full(&mut current_state.af, &mut current_state.de, memory), current_state),
        0x13 => instruction_finished(increment_full(&mut current_state.de), current_state),
        0x14 => instruction_finished(increment_lb(&mut current_state.de, &mut current_state.af), current_state),
        0x15 => instruction_finished(decrement_lb(&mut current_state.de, &mut current_state.af), current_state),
        0x16 => instruction_finished(ld_imm_into_hi(&mut current_state.de, memory, &current_state.pc.get()), current_state),
        0x17 => instruction_finished(rla(&mut current_state.af), current_state),
        0x18 => relative_jump(memory, current_state),
        0x19 => instruction_finished(add_full(&mut current_state.hl, &mut current_state.de, &mut current_state.af), current_state),
        0x1A => instruction_finished(ld_hi_from_full(&mut current_state.af, &mut current_state.de, memory), current_state),
        0x1B => instruction_finished(decrement_full(&mut current_state.de), current_state),
        0x1C => instruction_finished(increment_rb(&mut current_state.de, &mut current_state.af), current_state),
        0x1D => instruction_finished(decrement_rb(&mut current_state.de, &mut current_state.af), current_state),
        0x1E => instruction_finished(ld_imm_into_low(&mut current_state.de, memory, &current_state.pc.get()), current_state),
        0x1F => instruction_finished(rr_a(&mut current_state.af), current_state),

        0x20 => conditional_relative_jump(JumpCondition::ZNotSet, memory, current_state),
        0x21 => instruction_finished(ld_imm_into_full(&mut current_state.hl, memory, &current_state.pc.get()), current_state),
        0x22 => instruction_finished(save_a_to_hl_inc(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x23 => instruction_finished(increment_full(&mut current_state.hl), current_state),
        0x24 => instruction_finished(increment_lb(&mut current_state.hl, &mut current_state.af), current_state),
        0x25 => instruction_finished(decrement_lb(&mut current_state.hl, &mut current_state.af), current_state),
        0x26 => instruction_finished(ld_imm_into_hi(&mut current_state.hl, memory, &current_state.pc.get()), current_state),
        0x28 => conditional_relative_jump(JumpCondition::ZSet, memory, current_state),
        0x29 => instruction_finished(add_hl_to_hl(&mut current_state.hl, &mut current_state.af), current_state),
        0x2A => instruction_finished(ld_a_from_hl_inc(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x2B => instruction_finished(decrement_full(&mut current_state.hl), current_state),
        0x2C => instruction_finished(increment_rb(&mut current_state.hl, &mut current_state.af), current_state),
        0x2D => instruction_finished(decrement_rb(&mut current_state.hl, &mut current_state.af), current_state),
        0x2E => instruction_finished(ld_imm_into_low(&mut current_state.hl, memory, &current_state.pc.get()), current_state),
        0x2F => instruction_finished(cpl(&mut current_state.af), current_state),

        0x30 => conditional_relative_jump(JumpCondition::CNotSet, memory, current_state),        
        0x31 => instruction_finished(ld_imm_into_full(&mut current_state.sp, memory, &current_state.pc.get()), current_state),
        0x32 => instruction_finished(save_a_to_hl_dec(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x33 => instruction_finished(increment_full(&mut current_state.sp), current_state),
        0x34 => instruction_finished(increment_value(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x35 => instruction_finished(decrement_value(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x36 => instruction_finished(save_imm_to_hl(&mut current_state.hl, memory, &current_state.pc.get()), current_state),
        0x37 => instruction_finished(scf(&mut current_state.af), current_state),
        0x38 => conditional_relative_jump(JumpCondition::CSet, memory, current_state),
        0x39 => instruction_finished(add_full(&mut current_state.hl, &mut current_state.sp, &mut current_state.af), current_state),
        0x3A => instruction_finished(ld_a_from_hl_dec(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x3B => instruction_finished(decrement_full(&mut current_state.sp), current_state),
        0x3C => instruction_finished(increment_a(&mut current_state.af), current_state),
        0x3D => instruction_finished(decrement_a(&mut current_state.af), current_state),
        0x3E => instruction_finished(ld_imm_into_hi(&mut current_state.af, memory, &current_state.pc.get()), current_state),
        0x3F => instruction_finished(ccf(&mut current_state.af), current_state),

        0x40 => instruction_finished((1, 4), current_state),
        0x42 => instruction_finished(ld_hi_into_hi(&mut current_state.bc, &mut current_state.de), current_state),
        0x43 => instruction_finished(ld_low_into_hi(&mut current_state.bc, &mut current_state.de), current_state),
        0x44 => instruction_finished(ld_hi_into_hi(&mut current_state.bc, &mut current_state.hl), current_state),
        0x45 => instruction_finished(ld_low_into_hi(&mut current_state.bc, &mut current_state.hl), current_state),
        0x46 => instruction_finished(ld_hi_from_full(&mut current_state.bc, &mut current_state.hl, memory), current_state),
        0x47 => instruction_finished(ld_hi_into_hi(&mut current_state.bc, &mut current_state.af), current_state),
        0x49 => instruction_finished((1, 4), current_state),
        0x4A => instruction_finished(ld_hi_into_low(&mut current_state.bc, &mut current_state.de), current_state),
        0x4B => instruction_finished(ld_low_into_low(&mut current_state.bc, &mut current_state.de), current_state),
        0x4C => instruction_finished(ld_hi_into_low(&mut current_state.bc, &mut current_state.hl), current_state),
        0x4D => instruction_finished(ld_low_into_low(&mut current_state.bc, &mut current_state.hl), current_state),
        0x4E => instruction_finished(ld_low_from_full(&mut current_state.bc, &mut current_state.hl, memory), current_state),
        0x4F => instruction_finished(ld_hi_into_low(&mut current_state.bc, &mut current_state.af), current_state),

        0x50 => instruction_finished(ld_hi_into_hi(&mut current_state.de, &mut current_state.bc), current_state),
        0x51 => instruction_finished(ld_low_into_hi(&mut current_state.de, &mut current_state.bc), current_state),
        0x52 => instruction_finished((1, 4), current_state),
        0x54 => instruction_finished(ld_hi_into_hi(&mut current_state.de, &mut current_state.hl), current_state),
        0x55 => instruction_finished(ld_low_into_hi(&mut current_state.de, &mut current_state.hl), current_state),
        0x56 => instruction_finished(ld_hi_from_full(&mut current_state.de, &mut current_state.hl, memory), current_state),
        0x57 => instruction_finished(ld_hi_into_hi(&mut current_state.de, &mut current_state.af), current_state),
        0x58 => instruction_finished(ld_hi_into_low(&mut current_state.de, &mut current_state.bc), current_state),
        0x59 => instruction_finished(ld_low_into_low(&mut current_state.de, &mut current_state.bc), current_state),
        0x5B => instruction_finished((1, 4), current_state),
        0x5C => instruction_finished(ld_hi_into_low(&mut current_state.de, &mut current_state.hl), current_state),
        0x5D => instruction_finished(ld_low_into_low(&mut current_state.de, &mut current_state.hl), current_state),
        0x5E => instruction_finished(ld_low_from_full(&mut current_state.de, &mut current_state.hl, memory), current_state),
        0x5F => instruction_finished(ld_hi_into_low(&mut current_state.de, &mut current_state.af), current_state),

        0x60 => instruction_finished(ld_hi_into_hi(&mut current_state.hl, &mut current_state.bc), current_state),
        0x61 => instruction_finished(ld_low_into_hi(&mut current_state.hl, &mut current_state.bc), current_state),
        0x62 => instruction_finished(ld_hi_into_hi(&mut current_state.hl, &mut current_state.de), current_state),
        0x63 => instruction_finished(ld_low_into_hi(&mut current_state.hl, &mut current_state.de), current_state),
        0x64 => instruction_finished((1, 4), current_state),
        0x66 => instruction_finished(ld_h_from_hl(&mut current_state.de, memory), current_state),
        0x67 => instruction_finished(ld_hi_into_hi(&mut current_state.hl, &mut current_state.af), current_state),
        0x68 => instruction_finished(ld_hi_into_low(&mut current_state.hl, &mut current_state.bc), current_state),
        0x69 => instruction_finished(ld_low_into_low(&mut current_state.hl, &mut current_state.bc), current_state),
        0x6A => instruction_finished(ld_hi_into_low(&mut current_state.hl, &mut current_state.de), current_state),
        0x6B => instruction_finished(ld_low_into_low(&mut current_state.hl, &mut current_state.de), current_state),
        0x6D => instruction_finished((1, 4), current_state),
        0x6E => instruction_finished(ld_l_from_hl(&mut current_state.de, memory), current_state),
        0x6F => instruction_finished(ld_hi_into_low(&mut current_state.hl, &mut current_state.af), current_state),

        0x70 => instruction_finished(save_hi_to_hl(&mut current_state.bc, &mut current_state.hl, memory), current_state),
        0x71 => instruction_finished(save_low_to_hl(&mut current_state.bc, &mut current_state.hl, memory), current_state),
        0x72 => instruction_finished(save_hi_to_hl(&mut current_state.de, &mut current_state.hl, memory), current_state),
        0x73 => instruction_finished(save_low_to_hl(&mut current_state.de, &mut current_state.hl, memory), current_state),
        0x74 => instruction_finished(save_h_to_hl(&mut current_state.hl, memory), current_state),
        0x75 => instruction_finished(save_l_to_hl(&mut current_state.hl, memory), current_state),
        0x77 => instruction_finished(save_hi_to_hl(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x78 => instruction_finished(ld_hi_into_hi(&mut current_state.af, &mut current_state.bc), current_state),
        0x79 => instruction_finished(ld_low_into_hi(&mut current_state.af, &mut current_state.bc), current_state),
        0x7A => instruction_finished(ld_hi_into_hi(&mut current_state.af, &mut current_state.de), current_state),
        0x7B => instruction_finished(ld_low_into_hi(&mut current_state.af, &mut current_state.de), current_state),
        0x7C => instruction_finished(ld_hi_into_hi(&mut current_state.af, &mut current_state.hl), current_state),
        0x7D => instruction_finished(ld_low_into_hi(&mut current_state.af, &mut current_state.hl), current_state),
        0x7E => instruction_finished(save_hi_to_hl(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x7F => instruction_finished((1, 4), current_state),

        0x80 => instruction_finished(add_hi_to_a(&mut current_state.af, &mut current_state.bc), current_state),
        0x81 => instruction_finished(add_low_to_a(&mut current_state.af, &mut current_state.bc), current_state),
        0x82 => instruction_finished(add_hi_to_a(&mut current_state.af, &mut current_state.de), current_state),
        0x83 => instruction_finished(add_low_to_a(&mut current_state.af, &mut current_state.de), current_state),
        0x84 => instruction_finished(add_hi_to_a(&mut current_state.af, &mut current_state.hl), current_state),
        0x85 => instruction_finished(add_low_to_a(&mut current_state.af, &mut current_state.hl), current_state),
        0x86 => instruction_finished(add_val_to_a(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x87 => instruction_finished(add_a_to_a(&mut current_state.af), current_state),
        0x88 => instruction_finished(adc_hi_to_a(&mut current_state.af, &mut current_state.bc), current_state),
        0x89 => instruction_finished(adc_low_to_a(&mut current_state.af, &mut current_state.bc), current_state),
        0x8A => instruction_finished(adc_hi_to_a(&mut current_state.af, &mut current_state.de), current_state),
        0x8B => instruction_finished(adc_low_to_a(&mut current_state.af, &mut current_state.de), current_state),
        0x8C => instruction_finished(adc_hi_to_a(&mut current_state.af, &mut current_state.hl), current_state),
        0x8D => instruction_finished(adc_low_to_a(&mut current_state.af, &mut current_state.hl), current_state),
        0x8E => instruction_finished(adc_val_to_a(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x8F => instruction_finished(adc_a_to_a(&mut current_state.af), current_state),

        0x90 => instruction_finished(sub_hi_from_a(&mut current_state.af, &mut current_state.bc), current_state),
        0x91 => instruction_finished(sub_low_from_a(&mut current_state.af, &mut current_state.bc), current_state),
        0x92 => instruction_finished(sub_hi_from_a(&mut current_state.af, &mut current_state.de), current_state),
        0x93 => instruction_finished(sub_low_from_a(&mut current_state.af, &mut current_state.de), current_state),
        0x94 => instruction_finished(sub_hi_from_a(&mut current_state.af, &mut current_state.hl), current_state),
        0x95 => instruction_finished(sub_low_from_a(&mut current_state.af, &mut current_state.hl), current_state),
        0x96 => instruction_finished(sub_val_from_a(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x97 => instruction_finished(sub_a_from_a(&mut current_state.af), current_state),
        0x98 => instruction_finished(sbc_hi_from_a(&mut current_state.af, &mut current_state.bc), current_state),
        0x99 => instruction_finished(sbc_low_from_a(&mut current_state.af, &mut current_state.bc), current_state),
        0x9A => instruction_finished(sbc_hi_from_a(&mut current_state.af, &mut current_state.de), current_state),
        0x9B => instruction_finished(sbc_low_from_a(&mut current_state.af, &mut current_state.de), current_state),
        0x9C => instruction_finished(sbc_hi_from_a(&mut current_state.af, &mut current_state.hl), current_state),
        0x9D => instruction_finished(sbc_low_from_a(&mut current_state.af, &mut current_state.hl), current_state),
        0x9E => instruction_finished(sbc_val_from_a(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x9F => instruction_finished(sbc_a_from_a(&mut current_state.af), current_state),

        0xA0 => instruction_finished(and_a_with_hi(&mut current_state.af, &mut current_state.bc), current_state),
        0xA1 => instruction_finished(and_a_with_low(&mut current_state.af, &mut current_state.bc), current_state),
        0xA2 => instruction_finished(and_a_with_hi(&mut current_state.af, &mut current_state.de), current_state),
        0xA3 => instruction_finished(and_a_with_low(&mut current_state.af, &mut current_state.de), current_state),
        0xA4 => instruction_finished(and_a_with_hi(&mut current_state.af, &mut current_state.hl), current_state),
        0xA5 => instruction_finished(and_a_with_low(&mut current_state.af, &mut current_state.hl), current_state),
        0xA6 => instruction_finished(and_a_with_value(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0xA7 => instruction_finished(and_a_with_a(&mut current_state.af), current_state),
        0xA8 => instruction_finished(xor_a_with_hi(&mut current_state.af, &mut current_state.bc), current_state),
        0xA9 => instruction_finished(xor_a_with_low(&mut current_state.af, &mut current_state.bc), current_state),
        0xAA => instruction_finished(xor_a_with_hi(&mut current_state.af, &mut current_state.de), current_state),
        0xAB => instruction_finished(xor_a_with_low(&mut current_state.af, &mut current_state.de), current_state),
        0xAC => instruction_finished(xor_a_with_hi(&mut current_state.af, &mut current_state.hl), current_state),
        0xAD => instruction_finished(xor_a_with_low(&mut current_state.af, &mut current_state.hl), current_state),
        0xAE => instruction_finished(xor_a_with_value(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0xAF => instruction_finished(xor_a_with_a(&mut current_state.af), current_state),

        0xB0 => instruction_finished(or_a_with_hi(&mut current_state.af, &mut current_state.bc), current_state),
        0xB1 => instruction_finished(or_a_with_low(&mut current_state.af, &mut current_state.bc), current_state),
        0xB2 => instruction_finished(or_a_with_hi(&mut current_state.af, &mut current_state.de), current_state),
        0xB3 => instruction_finished(or_a_with_low(&mut current_state.af, &mut current_state.de), current_state),
        0xB4 => instruction_finished(or_a_with_hi(&mut current_state.af, &mut current_state.hl), current_state),
        0xB5 => instruction_finished(or_a_with_low(&mut current_state.af, &mut current_state.hl), current_state),
        0xB6 => instruction_finished(or_a_with_value(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0xB7 => instruction_finished(or_a_with_a(&mut current_state.af), current_state),
        0xB8 => instruction_finished(cp_a_with_hi(&mut current_state.af, &mut current_state.bc), current_state),
        0xB9 => instruction_finished(cp_a_with_low(&mut current_state.af, &mut current_state.bc), current_state),
        0xBA => instruction_finished(cp_a_with_hi(&mut current_state.af, &mut current_state.de), current_state),
        0xBB => instruction_finished(cp_a_with_low(&mut current_state.af, &mut current_state.de), current_state),
        0xBC => instruction_finished(cp_a_with_hi(&mut current_state.af, &mut current_state.hl), current_state),
        0xBD => instruction_finished(cp_a_with_low(&mut current_state.af, &mut current_state.hl), current_state),
        0xBE => instruction_finished(cp_a_with_value(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0xBF => instruction_finished(cp_a_with_a(&mut current_state.af), current_state),

        0xC0 => conditional_ret(current_state, JumpCondition::ZNotSet),
        0xC1 => instruction_finished(pop(&mut current_state.bc, &mut current_state.stack), current_state),
        0xC2 => conditional_jump(JumpCondition::ZNotSet, memory, current_state),
        0xC3 => jump(memory, current_state),
        0xC4 => conditional_call(memory, current_state, JumpCondition::ZNotSet),
        0xC5 => instruction_finished(push(&mut current_state.bc, &mut current_state.stack), current_state),
        0xC8 => conditional_ret(current_state, JumpCondition::ZSet),
        0xC9 => ret(current_state),
        0xCA => conditional_jump(JumpCondition::ZSet, memory, current_state),
        0xCB => result = CycleResult::InvalidOp,
        0xCC => conditional_call(memory, current_state, JumpCondition::ZSet),
        0xCD => call(memory, current_state),
        0xCE => instruction_finished(adc_imm_to_a(&mut current_state.af, &current_state.pc.get(), memory), current_state),

        0xD0 => conditional_ret(current_state, JumpCondition::CNotSet),
        0xD1 => instruction_finished(pop(&mut current_state.de, &mut current_state.stack), current_state),
        0xD2 => conditional_jump(JumpCondition::CNotSet, memory, current_state),
        0xD3 => result = CycleResult::InvalidOp,
        0xD4 => conditional_call(memory, current_state, JumpCondition::CNotSet),
        0xD5 => instruction_finished(push(&mut current_state.de, &mut current_state.stack), current_state),
        0xD6 => instruction_finished(sub_imm_from_a(&mut current_state.af, memory, &current_state.pc.get()), current_state),
        0xD8 => conditional_ret(current_state, JumpCondition::CSet),
        0xDA => conditional_jump(JumpCondition::CSet, memory, current_state),
        0xDB => result = CycleResult::InvalidOp,
        0xDC => conditional_call(memory, current_state, JumpCondition::CSet),
        0xDD => result = CycleResult::InvalidOp,
        0xDE => instruction_finished(sbc_imm_from_a(&mut current_state.af, memory, &current_state.pc.get()), current_state),

        0xE0 => instruction_finished(save_a_to_ff_imm(&mut current_state.af, &mut current_state.pc.get(), memory), current_state),
        0xE1 => instruction_finished(pop(&mut current_state.hl, &mut current_state.stack), current_state),
        0xE2 => instruction_finished(save_a_to_c_imm(&mut current_state.af, &mut current_state.bc, memory), current_state),
        0xE3 => result = CycleResult::InvalidOp,
        0xE4 => result = CycleResult::InvalidOp,
        0xE5 => instruction_finished(push(&mut current_state.hl, &mut current_state.stack), current_state),
        0xEA => instruction_finished(save_a_to_nn(&mut current_state.af, &current_state.pc.get(), memory), current_state),
        0xEB => result = CycleResult::InvalidOp,
        0xEC => result = CycleResult::InvalidOp,
        0xED => result = CycleResult::InvalidOp,

        0xF0 => instruction_finished(ld_a_from_ff_imm(&mut current_state.af, &mut current_state.pc.get(), memory), current_state),
        0xF1 => instruction_finished(pop(&mut current_state.af, &mut current_state.stack), current_state),
        0xF3 => instruction_finished(di(memory), current_state),
        0xF4 => result = CycleResult::InvalidOp,
        0xF5 => instruction_finished(push(&mut current_state.af, &mut current_state.stack), current_state),
        0xFA => instruction_finished(ld_a_from_imm_addr(&mut current_state.af, &current_state.pc.get(), memory), current_state),
        0xFB => instruction_finished(ei(memory), current_state),
        0xFC => result = CycleResult::InvalidOp,
        0xFD => result = CycleResult::InvalidOp,
        0xFE => instruction_finished(cp_a_with_imm(&mut current_state.af, &current_state.pc.get(), memory), current_state),

        _ => { 
            println!("Tried to run unimplemented opcode 0x{} at PC {}", format!("{:X}", opcode), format!("{:X}", current_state.pc.get()));
            result = CycleResult::UnimplementedOp;
        }
    }

    result
    
}

fn instruction_finished(values: (u16, u32), state: &mut CpuState) {

    state.pc.add(values.0); state.cycles.add(values.1);
}

fn nop(current_state: &mut CpuState) {

    current_state.pc.add(1);
    current_state.cycles.add(1);
}

fn jump(memory: &mut Memory, state: &mut CpuState) {

    let current_pc = state.pc.get();
    state.pc.set(cpu::memory_read_u16(&(current_pc + 1), memory));
    state.cycles.add(16);
}

fn relative_jump(memory: &mut Memory, state: &mut CpuState) {

    let current_pc = state.pc.get();
    let target = cpu::memory_read_u8(&(current_pc + 1), memory) as i8;
    state.pc.set(current_pc.wrapping_add(target as u16) + 2);
    state.cycles.add(12);
}

fn conditional_jump(condition: JumpCondition, memory: &mut Memory, state: &mut CpuState) {

    let should_jump: bool;
    match condition {

        JumpCondition::ZNotSet => should_jump = !utils::check_bit(state.af.get_register_rb(), 7),
        JumpCondition::CNotSet => should_jump = !utils::check_bit(state.af.get_register_rb(), 4),
        JumpCondition::ZSet => should_jump = utils::check_bit(state.af.get_register_rb(), 7),
        JumpCondition::CSet => should_jump = utils::check_bit(state.af.get_register_rb(), 4),
    }

    if should_jump { jump(memory, state) ;}
    else { state.pc.add(3); state.cycles.add(12) }
}

fn conditional_relative_jump(condition: JumpCondition, memory: &mut Memory, state: &mut CpuState) {

    let jump: bool;
    match condition {

        JumpCondition::ZNotSet => jump = !utils::check_bit(state.af.get_register_rb(), 7),
        JumpCondition::CNotSet => jump = !utils::check_bit(state.af.get_register_rb(), 4),
        JumpCondition::ZSet => jump = utils::check_bit(state.af.get_register_rb(), 7),
        JumpCondition::CSet => jump = utils::check_bit(state.af.get_register_rb(), 4),
    }

    if jump { relative_jump(memory, state) ;}
    else { state.pc.add(2); state.cycles.add(8) }
}

fn call(memory: &mut Memory, state: &mut CpuState) {

    let next_pc = state.pc.get() + 3;

    state.stack.push(utils::get_lb(next_pc));
    state.stack.push(utils::get_rb(next_pc));
    state.pc.set(cpu::memory_read_u16(&(next_pc - 2), memory));
    state.cycles.add(24);
}

fn conditional_call(memory: &mut Memory, state: &mut CpuState, condition: JumpCondition) {

    let should_call: bool;
    match condition {

        JumpCondition::ZNotSet => should_call = !utils::check_bit(state.af.get_register_rb(), 7),
        JumpCondition::CNotSet => should_call = !utils::check_bit(state.af.get_register_rb(), 4),
        JumpCondition::ZSet => should_call = utils::check_bit(state.af.get_register_rb(), 7),
        JumpCondition::CSet => should_call = utils::check_bit(state.af.get_register_rb(), 4),
    }

    if should_call { call(memory, state) ;}
    else { state.pc.add(3); state.cycles.add(12) }
}

fn ret(state: &mut CpuState) {
    
    let mut target_ret = vec![0, 2];
    target_ret[0] = state.stack.pop().unwrap();
    target_ret[1] = state.stack.pop().unwrap();
    state.pc.set(LittleEndian::read_u16(&target_ret));
    state.cycles.add(16);
}

fn conditional_ret(state: &mut CpuState, condition: JumpCondition) {

    let should_ret: bool;
    match condition {

        JumpCondition::ZNotSet => should_ret = !utils::check_bit(state.af.get_register_rb(), 7),
        JumpCondition::CNotSet => should_ret = !utils::check_bit(state.af.get_register_rb(), 4),
        JumpCondition::ZSet => should_ret = utils::check_bit(state.af.get_register_rb(), 7),
        JumpCondition::CSet => should_ret = utils::check_bit(state.af.get_register_rb(), 4),
    }

    if should_ret { ret(state);}
    else { state.pc.add(1); state.cycles.add(8) }
}

fn ld_imm_into_full(target_reg: &mut CpuReg, memory: &mut Memory, pc: &u16) -> (u16, u32) {

    target_reg.set_register(cpu::memory_read_u16(&(pc + 1), memory));
    (3, 12)
}

fn ld_hi_from_full(reg: &mut CpuReg, full: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    reg.set_register_lb(cpu::memory_read_u8(&full.get_register(), memory));
    (1, 8)
}

fn ld_low_from_full(reg: &mut CpuReg, full: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    reg.set_register_rb(cpu::memory_read_u8(&full.get_register(), memory));
    (1, 8)
}

fn ld_h_from_hl(hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let addr = hl.get_register();
    hl.set_register_lb(cpu::memory_read_u8(&addr, memory));
    (1, 8)
}

fn ld_l_from_hl(hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let addr = hl.get_register();
    hl.set_register_rb(cpu::memory_read_u8(&addr, memory));
    (1, 8)
}

fn ld_a_from_hl_inc(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    af.set_register_lb(cpu::memory_read_u8(&hl.get_register(), memory));
    hl.increment();
    (1, 8)
}

fn ld_a_from_hl_dec(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {
    
    af.set_register_lb(cpu::memory_read_u8(&hl.get_register(), memory));
    hl.decrement();
    (1, 8)
}

fn ld_a_from_imm_addr(af: &mut CpuReg, pc: &u16, memory: &mut Memory) -> (u16, u32) {

    let target_addr = cpu::memory_read_u16(&(pc + 1), memory);
    af.set_register_lb(cpu::memory_read_u8(&target_addr, memory));
    (3, 16)
}

fn ld_a_from_ff_imm(af: &mut CpuReg, pc: &u16, memory: &mut Memory) -> (u16, u32) {

    let target_addr = 0xFF00 + cpu::memory_read_u8(&(pc + 1), memory) as u16;
    af.set_register_lb(cpu::memory_read_u8(&target_addr, memory));
    (2, 12)
}

fn save_a_to_full(a: &mut CpuReg, full: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    cpu::memory_write(full.get_register(), a.get_register_lb(), memory);
    (1, 8)
}

fn save_a_to_hl_inc(a: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    cpu::memory_write(hl.get_register(), a.get_register_lb(), memory);
    hl.increment();
    (1, 8)
}

fn save_a_to_hl_dec(a: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    cpu::memory_write(hl.get_register(), a.get_register_lb(), memory);
    hl.decrement();
    (1, 8)
}

fn save_a_to_ff_imm(af: &mut CpuReg, pc: &u16, memory: &mut Memory) -> (u16, u32) {

    let target_addr = 0xFF00 + cpu::memory_read_u8(&(pc + 1), memory) as u16;
    cpu::memory_write(target_addr, af.get_register_lb(), memory);
    (2, 12)
}

fn save_a_to_c_imm(af: &mut CpuReg, bc: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let target_addr = 0xFF00 + bc.get_register_rb() as u16;
    cpu::memory_write(target_addr, af.get_register_lb(), memory);
    (2, 8)
}

fn save_a_to_nn(af: &mut CpuReg, pc: &u16, memory: &mut Memory) -> (u16, u32) {

    let target_addr = cpu::memory_read_u16(&(pc + 1), memory);
    cpu::memory_write(target_addr, af.get_register_lb(), memory);
    (3, 16)
}

fn save_hi_to_hl(reg: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    cpu::memory_write(hl.get_register(), reg.get_register_lb(), memory);
    (1, 8)
}

fn save_low_to_hl(reg: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    cpu::memory_write(hl.get_register(), reg.get_register_rb(), memory);
    (1, 8)
}

fn save_h_to_hl(hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    cpu::memory_write(hl.get_register(), hl.get_register_lb(), memory);
    (1, 8)
}

fn save_l_to_hl(hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    cpu::memory_write(hl.get_register(), hl.get_register_rb(), memory);
    (1, 8)
}

fn save_imm_to_hl(hl: &mut CpuReg, memory: &mut Memory, pc: &u16) -> (u16, u32) {

    let value = cpu::memory_read_u8(pc, memory);
    cpu::memory_write(hl.get_register(), value, memory);
    (2, 12)
}

fn increment_full(reg: &mut CpuReg) -> (u16, u32) {

    reg.increment();
    (1, 8)
}

fn increment_lb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {

    let overflow = reg.increment_lb();
    utils::set_zf(overflow, af); utils::set_nf(false, af);
    (1, 4)
}

fn increment_rb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {
    
    let overflow = reg.increment_rb();
    utils::set_zf(overflow, af); utils::set_nf(false, af);
    (1, 4)
}

fn increment_a(af: &mut CpuReg) -> (u16, u32) {

    let overflow = af.increment_lb();
    utils::set_zf(overflow, af); utils::set_nf(false, af);
    (1, 4)
}

fn increment_value(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {
    
    let value = cpu::memory_read_u8(&hl.get_register(), memory).overflowing_add(1);
    cpu::memory_write(hl.get_register(), value.0, memory);
    utils::set_zf(value.1, af); utils::set_nf(false, af);
    (1, 12)
}

fn decrement_full(reg: &mut CpuReg) -> (u16, u32) {
    
    reg.decrement();
    (1, 4)
}

fn decrement_lb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {
    
    reg.decrement_lb();
    utils::set_zf(reg.get_register_lb() == 0, af); utils::set_nf(true, af);
    (1, 4)
}

fn decrement_rb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {
    
    reg.decrement_rb();
    utils::set_zf(reg.get_register_rb() == 0, af); utils::set_nf(true, af);
    (1, 4)
}

fn decrement_a(af: &mut CpuReg) -> (u16, u32) {
    
    af.decrement_lb();
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    (1, 4)
}

fn decrement_value(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {
    
    let value = cpu::memory_read_u8(&hl.get_register(), memory).overflowing_sub(1);
    cpu::memory_write(hl.get_register(), value.0, memory);
    utils::set_zf(value.0 == 0, af); utils::set_nf(true, af);
    (1, 12)
}

fn ld_hi_into_low(target: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    target.set_register_rb(source.get_register_lb());
    (1, 4)
}

fn ld_hi_into_hi(target: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {
    
    target.set_register_lb(source.get_register_lb());
    (1, 4)
}

fn ld_low_into_low(target: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    target.set_register_rb(source.get_register_rb());
    (1, 4)
}

fn ld_low_into_hi(target: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {
    
    target.set_register_lb(source.get_register_rb());
    (1, 4)
}

fn ld_imm_into_hi(target: &mut CpuReg, memory: &mut Memory, pc: &u16) -> (u16, u32) {
    
    target.set_register_lb(cpu::memory_read_u8(&(pc + 1), memory));
    (2, 8)
}

fn ld_imm_into_low(target: &mut CpuReg, memory: &mut Memory, pc: &u16) -> (u16, u32) {

    target.set_register_rb(cpu::memory_read_u8(&(pc + 1), memory));
    (2, 8)
}

fn add_hl_to_hl(hl: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {

    let hl_value = hl.get_register();
    let overflow = hl.add_to_reg(hl_value);
    utils::set_nf(false, af); utils::set_cf(overflow, af);
    (1, 8)
}

fn add_full(target: &mut CpuReg, source: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {

    let overflow = target.add_to_reg(source.get_register());
    utils::set_nf(false, af); utils::set_cf(overflow, af);
    (1, 8)
}

fn add_hi_to_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let overflow = af.add_to_lb(source.get_register_lb());
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(false, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn add_low_to_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let overflow = af.add_to_lb(source.get_register_rb());
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(false, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn add_val_to_a(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let overflow = af.add_to_lb(cpu::memory_read_u8(&hl.get_register(), memory));
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(false, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn add_a_to_a(af: &mut CpuReg) -> (u16, u32) {

    let value = af.get_register_lb();
    let overflow = af.add_to_lb(value);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(false, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn adc_hi_to_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let carry = utils::get_carry(af);
    let overflow = af.add_to_lb(source.get_register_lb() + carry);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(false, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn adc_low_to_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let carry = utils::get_carry(af);
    let overflow = af.add_to_lb(source.get_register_rb() + carry);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(false, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn adc_val_to_a(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let carry = utils::get_carry(af);
    let overflow = af.add_to_lb(cpu::memory_read_u8(&hl.get_register(), memory) + carry);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(false, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn adc_a_to_a(af: &mut CpuReg) -> (u16, u32) {

    let carry = utils::get_carry(af);
    let value = af.get_register_lb();
    let overflow = af.add_to_lb(value + carry);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(false, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn adc_imm_to_a(af: &mut CpuReg, pc: &u16, memory: &mut Memory) -> (u16, u32) {

    let carry = utils::get_carry(af);
    let overflow = af.add_to_lb(cpu::memory_read_u8(&(pc + 1), memory) + carry);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(false, af);
    utils::set_cf(overflow, af);
    (2, 8)
}

fn sub_hi_from_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let overflow = af.sub_from_lb(source.get_register_lb());
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn sub_low_from_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let overflow = af.sub_from_lb(source.get_register_rb());
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn sub_val_from_a(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let overflow = af.sub_from_lb(cpu::memory_read_u8(&hl.get_register(), memory));
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn sub_a_from_a(af: &mut CpuReg) -> (u16, u32) {

    let value = af.get_register_lb();
    let overflow = af.sub_from_lb(value);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn sub_imm_from_a(af: &mut CpuReg, memory: &mut Memory, pc: &u16) -> (u16, u32) {

    let value = cpu::memory_read_u8(&(pc + 1), memory);
    let overflow = af.sub_from_lb(value);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_cf(overflow, af);
    (2, 8)
}

fn sbc_hi_from_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let carry = utils::get_carry(af);
    let overflow = af.sub_from_lb(source.get_register_lb() + carry);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn sbc_low_from_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let carry = utils::get_carry(af);
    let overflow = af.sub_from_lb(source.get_register_rb() + carry);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn sbc_val_from_a(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let carry = utils::get_carry(af);
    let overflow = af.sub_from_lb(cpu::memory_read_u8(&hl.get_register(), memory) + carry);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn sbc_a_from_a(af: &mut CpuReg) -> (u16, u32) {

    let carry = utils::get_carry(af);
    let value = af.get_register_lb();
    let overflow = af.sub_from_lb(value + carry);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_cf(overflow, af);
    (1, 4)
}

fn sbc_imm_from_a(af: &mut CpuReg, memory: &mut Memory, pc: &u16) -> (u16, u32) {

    let carry = utils::get_carry(af);
    let value = cpu::memory_read_u8(&(pc + 1), memory);
    let overflow = af.sub_from_lb(value + carry);
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_cf(overflow, af);
    (2, 8)
}

fn and_a_with_hi(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let result = af.get_register_lb() & source.get_register_lb();
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(true, af); utils::set_cf(false, af);
    (1, 4)
}

fn and_a_with_low(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let result = af.get_register_lb() & source.get_register_rb();
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(true, af); utils::set_cf(false, af);
    (1, 4)
}

fn and_a_with_value(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let result = af.get_register_lb() & cpu::memory_read_u8(&hl.get_register(), memory);
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(true, af); utils::set_cf(false, af);
    (1, 4)
}

fn and_a_with_a(af: &mut CpuReg) -> (u16, u32) {

    let result = af.get_register_lb() & af.get_register_lb();
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(true, af); utils::set_cf(false, af);
    (1, 4)
}

fn or_a_with_hi(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let result = af.get_register_lb() | source.get_register_lb();
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(false, af); utils::set_cf(false, af);
    (1, 4)
}

fn or_a_with_low(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let result = af.get_register_lb() | source.get_register_rb();
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(false, af); utils::set_cf(false, af);
    (1, 4)
}

fn or_a_with_value(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let result = af.get_register_lb() | cpu::memory_read_u8(&hl.get_register(), memory);
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(false, af); utils::set_cf(false, af);
    (1, 4)
}

fn or_a_with_a(af: &mut CpuReg) -> (u16, u32) {

    let result = af.get_register_lb() | af.get_register_lb();
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(false, af); utils::set_cf(false, af);
    (1, 4)
}

fn xor_a_with_hi(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let result = af.get_register_lb() ^ source.get_register_lb();
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(false, af); utils::set_cf(false, af);
    (1, 4)
}

fn xor_a_with_low(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let result = af.get_register_lb() ^ source.get_register_rb();
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(false, af); utils::set_cf(false, af);
    (1, 4)
}

fn xor_a_with_value(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let result = af.get_register_lb() ^ cpu::memory_read_u8(&hl.get_register(), memory);
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(false, af); utils::set_cf(false, af);
    (1, 4)
}

fn xor_a_with_a(af: &mut CpuReg) -> (u16, u32) {

    let result = af.get_register_lb() ^ af.get_register_lb();
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(false, af); utils::set_cf(false, af);
    (1, 4)
}

fn cp_a_with_hi(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {

    let value = source.get_register_lb();
    utils::set_zf(af.get_register_lb() == value, af); utils::set_nf(true, af);
    utils::set_cf(af.get_register_lb() < value, af);
    (1, 4)
}

fn cp_a_with_low(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u32) {
    
    let value = source.get_register_rb();
    utils::set_zf(af.get_register_lb() == value, af); utils::set_nf(true, af);
    utils::set_cf(af.get_register_lb() < value, af);
    (1, 4)
}

fn cp_a_with_value(af: &mut CpuReg, hl: &mut CpuReg, memory: &mut Memory) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    utils::set_zf(af.get_register_lb() == value, af); utils::set_nf(true, af);
    utils::set_cf(af.get_register_lb() < value, af);
    (1, 8)
}

fn cp_a_with_imm(af: &mut CpuReg, pc: &u16, memory: &mut Memory) -> (u16, u32) {

    let value = cpu::memory_read_u8(&(pc + 1), memory);
    utils::set_zf(af.get_register_lb() == value, af); utils::set_nf(true, af);
    utils::set_cf(af.get_register_lb() < value, af);
    (2, 8)
}

fn cp_a_with_a(af: &mut CpuReg) -> (u16, u32) {

    let value = af.get_register_lb();
    utils::set_zf(value == value, af); utils::set_nf(true, af);
    utils::set_cf(value < value, af);
    (1, 4)
}

fn pop(reg: &mut CpuReg, stack: &mut Vec<u8>) -> (u16, u32) {

    let mut values = vec![0, 2];
    values[0] = stack.pop().unwrap(); values[1] = stack.pop().unwrap();
    reg.set_register(LittleEndian::read_u16(&values));
    (1, 12)
}

fn push(reg: &mut CpuReg, stack: &mut Vec<u8>) -> (u16, u32) {

    stack.push(reg.get_register_lb());
    stack.push(reg.get_register_rb());
    (1, 16)
    
}

fn rla(af: &mut CpuReg) -> (u16, u32) {

    let mut value = af.get_register_lb();
    let carry: u8;
    if utils::check_bit(af.get_register_rb(), 7) {carry = 1}
    else {carry = 0}
    utils::set_cf(utils::check_bit(value, 7), af);
    value = value << 1;
    af.set_register_lb(value | carry);
    utils::set_hf(false, af);
    utils::set_nf(false, af);
    utils::set_zf(false, af);
    (1, 4)
}

fn rr_a(af: &mut CpuReg) -> (u16, u32) {

    let mut value = af.get_register_lb();
    let carry: u8;
    if utils::check_bit(af.get_register_rb(), 7) {carry = 1}
    else {carry = 0}
    utils::set_cf(utils::check_bit(value, 7), af);
    value = value >> 1;
    af.set_register_lb(value | carry);
    utils::set_hf(false, af);
    utils::set_nf(false, af);
    utils::set_zf(false, af);
    (1, 4)
}

fn ei(memory: &mut Memory) -> (u16, u32) {

    cpu::memory_write(0xFFFF, 0, memory);
    (1, 4)
}

fn di(memory: &mut Memory) -> (u16, u32) {

    cpu::memory_write(0xFFFF, 0, memory);
    (1, 4)
}

fn cpl(af: &mut CpuReg) -> (u16, u32) {

    let value = !af.get_register_lb();
    af.set_register_lb(value);
    utils::set_nf(true, af);
    utils::set_hf(true, af);
    (1, 4)
}

fn scf(af: &mut CpuReg) -> (u16, u32) {

    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(true, af);
    (1, 4)
}

fn ccf(af: &mut CpuReg) -> (u16, u32) {
    
    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(false, af);
    (1, 4)
}