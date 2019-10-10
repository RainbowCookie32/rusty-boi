use std::sync::mpsc;

use super::utils;

use super::cpu;
use super::cpu::CpuState;
use super::cpu::CycleResult;

use super::memory::MemoryAccess;

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

pub fn run_instruction(current_state: &mut CpuState, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), opcode: u8) -> CycleResult {

    let mut result = CycleResult::Success;

    match opcode {

        0x00 => instruction_finished(nop(), current_state),
        0x01 => instruction_finished(ld_imm_into_full(&mut current_state.bc, memory, &current_state.pc.get()), current_state),
        0x02 => instruction_finished(save_a_to_full(&mut current_state.af, &mut current_state.bc, memory), current_state),
        0x03 => instruction_finished(increment_full(&mut current_state.bc), current_state),
        0x04 => instruction_finished(increment_lb(&mut current_state.bc, &mut current_state.af), current_state),
        0x05 => instruction_finished(decrement_lb(&mut current_state.bc, &mut current_state.af), current_state),
        0x06 => instruction_finished(ld_imm_into_hi(&mut current_state.bc, memory, &current_state.pc.get()), current_state),
        0x07 => instruction_finished(rlc_a(&mut current_state.af), current_state),
        0x08 => instruction_finished(save_sp_to_imm(&mut current_state.sp, memory, &current_state.pc.get()), current_state),
        0x09 => instruction_finished(add_full(&mut current_state.hl, &mut current_state.bc, &mut current_state.af), current_state),
        0x0A => instruction_finished(ld_hi_from_hl(&mut current_state.af, &mut current_state.bc, memory), current_state),
        0x0B => instruction_finished(decrement_full(&mut current_state.bc), current_state),
        0x0C => instruction_finished(increment_rb(&mut current_state.bc, &mut current_state.af), current_state),
        0x0D => instruction_finished(decrement_rb(&mut current_state.bc, &mut current_state.af), current_state),
        0x0E => instruction_finished(ld_imm_into_low(&mut current_state.bc, memory, &current_state.pc.get()), current_state),
        0x0F => instruction_finished(rrc_a(&mut current_state.af), current_state),

        0x10 => result = stop(current_state),
        0x11 => instruction_finished(ld_imm_into_full(&mut current_state.de, memory, &current_state.pc.get()), current_state),
        0x12 => instruction_finished(save_a_to_full(&mut current_state.af, &mut current_state.de, memory), current_state),
        0x13 => instruction_finished(increment_full(&mut current_state.de), current_state),
        0x14 => instruction_finished(increment_lb(&mut current_state.de, &mut current_state.af), current_state),
        0x15 => instruction_finished(decrement_lb(&mut current_state.de, &mut current_state.af), current_state),
        0x16 => instruction_finished(ld_imm_into_hi(&mut current_state.de, memory, &current_state.pc.get()), current_state),
        0x17 => instruction_finished(rla(&mut current_state.af), current_state),
        0x18 => relative_jump(memory, current_state),
        0x19 => instruction_finished(add_full(&mut current_state.hl, &mut current_state.de, &mut current_state.af), current_state),
        0x1A => instruction_finished(ld_hi_from_hl(&mut current_state.af, &mut current_state.de, memory), current_state),
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
        0x27 => instruction_finished(daa(&mut current_state.af), current_state),
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
        0x35 => instruction_finished(decrement_at_hl(&mut current_state.af, &mut current_state.hl, memory), current_state),
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
        0x41 => instruction_finished(ld_self_low_to_hi(&mut current_state.bc), current_state),
        0x42 => instruction_finished(ld_hi_into_hi(&mut current_state.bc, &mut current_state.de), current_state),
        0x43 => instruction_finished(ld_low_into_hi(&mut current_state.bc, &mut current_state.de), current_state),
        0x44 => instruction_finished(ld_hi_into_hi(&mut current_state.bc, &mut current_state.hl), current_state),
        0x45 => instruction_finished(ld_low_into_hi(&mut current_state.bc, &mut current_state.hl), current_state),
        0x46 => instruction_finished(ld_hi_from_hl(&mut current_state.bc, &mut current_state.hl, memory), current_state),
        0x47 => instruction_finished(ld_hi_into_hi(&mut current_state.bc, &mut current_state.af), current_state),
        0x48 => instruction_finished(ld_self_hi_to_low(&mut current_state.bc), current_state),
        0x49 => instruction_finished((1, 4), current_state),
        0x4A => instruction_finished(ld_hi_into_low(&mut current_state.bc, &mut current_state.de), current_state),
        0x4B => instruction_finished(ld_low_into_low(&mut current_state.bc, &mut current_state.de), current_state),
        0x4C => instruction_finished(ld_hi_into_low(&mut current_state.bc, &mut current_state.hl), current_state),
        0x4D => instruction_finished(ld_low_into_low(&mut current_state.bc, &mut current_state.hl), current_state),
        0x4E => instruction_finished(ld_low_from_hl(&mut current_state.bc, &mut current_state.hl, memory), current_state),
        0x4F => instruction_finished(ld_hi_into_low(&mut current_state.bc, &mut current_state.af), current_state),

        0x50 => instruction_finished(ld_hi_into_hi(&mut current_state.de, &mut current_state.bc), current_state),
        0x51 => instruction_finished(ld_low_into_hi(&mut current_state.de, &mut current_state.bc), current_state),
        0x52 => instruction_finished((1, 4), current_state),
        0x53 => instruction_finished(ld_self_low_to_hi(&mut current_state.de), current_state),
        0x54 => instruction_finished(ld_hi_into_hi(&mut current_state.de, &mut current_state.hl), current_state),
        0x55 => instruction_finished(ld_low_into_hi(&mut current_state.de, &mut current_state.hl), current_state),
        0x56 => instruction_finished(ld_hi_from_hl(&mut current_state.de, &mut current_state.hl, memory), current_state),
        0x57 => instruction_finished(ld_hi_into_hi(&mut current_state.de, &mut current_state.af), current_state),
        0x58 => instruction_finished(ld_hi_into_low(&mut current_state.de, &mut current_state.bc), current_state),
        0x59 => instruction_finished(ld_low_into_low(&mut current_state.de, &mut current_state.bc), current_state),
        0x5A => instruction_finished(ld_self_hi_to_low(&mut current_state.de), current_state),
        0x5B => instruction_finished((1, 4), current_state),
        0x5C => instruction_finished(ld_hi_into_low(&mut current_state.de, &mut current_state.hl), current_state),
        0x5D => instruction_finished(ld_low_into_low(&mut current_state.de, &mut current_state.hl), current_state),
        0x5E => instruction_finished(ld_low_from_hl(&mut current_state.de, &mut current_state.hl, memory), current_state),
        0x5F => instruction_finished(ld_hi_into_low(&mut current_state.de, &mut current_state.af), current_state),

        0x60 => instruction_finished(ld_hi_into_hi(&mut current_state.hl, &mut current_state.bc), current_state),
        0x61 => instruction_finished(ld_low_into_hi(&mut current_state.hl, &mut current_state.bc), current_state),
        0x62 => instruction_finished(ld_hi_into_hi(&mut current_state.hl, &mut current_state.de), current_state),
        0x63 => instruction_finished(ld_low_into_hi(&mut current_state.hl, &mut current_state.de), current_state),
        0x64 => instruction_finished((1, 4), current_state),
        0x65 => instruction_finished(ld_self_low_to_hi(&mut current_state.hl), current_state),
        0x66 => instruction_finished(ld_h_from_hl(&mut current_state.hl, memory), current_state),
        0x67 => instruction_finished(ld_hi_into_hi(&mut current_state.hl, &mut current_state.af), current_state),
        0x68 => instruction_finished(ld_hi_into_low(&mut current_state.hl, &mut current_state.bc), current_state),
        0x69 => instruction_finished(ld_low_into_low(&mut current_state.hl, &mut current_state.bc), current_state),
        0x6A => instruction_finished(ld_hi_into_low(&mut current_state.hl, &mut current_state.de), current_state),
        0x6B => instruction_finished(ld_low_into_low(&mut current_state.hl, &mut current_state.de), current_state),
        0x6C => instruction_finished(ld_self_hi_to_low(&mut current_state.hl), current_state),
        0x6D => instruction_finished((1, 4), current_state),
        0x6E => instruction_finished(ld_l_from_hl(&mut current_state.hl, memory), current_state),
        0x6F => instruction_finished(ld_hi_into_low(&mut current_state.hl, &mut current_state.af), current_state),

        0x70 => instruction_finished(save_hi_to_hl(&mut current_state.bc, &mut current_state.hl, memory), current_state),
        0x71 => instruction_finished(save_low_to_hl(&mut current_state.bc, &mut current_state.hl, memory), current_state),
        0x72 => instruction_finished(save_hi_to_hl(&mut current_state.de, &mut current_state.hl, memory), current_state),
        0x73 => instruction_finished(save_low_to_hl(&mut current_state.de, &mut current_state.hl, memory), current_state),
        0x74 => instruction_finished(save_h_to_hl(&mut current_state.hl, memory), current_state),
        0x75 => instruction_finished(save_l_to_hl(&mut current_state.hl, memory), current_state),
        0x76 => result = halt(current_state, memory),
        0x77 => instruction_finished(save_hi_to_hl(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x78 => instruction_finished(ld_hi_into_hi(&mut current_state.af, &mut current_state.bc), current_state),
        0x79 => instruction_finished(ld_low_into_hi(&mut current_state.af, &mut current_state.bc), current_state),
        0x7A => instruction_finished(ld_hi_into_hi(&mut current_state.af, &mut current_state.de), current_state),
        0x7B => instruction_finished(ld_low_into_hi(&mut current_state.af, &mut current_state.de), current_state),
        0x7C => instruction_finished(ld_hi_into_hi(&mut current_state.af, &mut current_state.hl), current_state),
        0x7D => instruction_finished(ld_low_into_hi(&mut current_state.af, &mut current_state.hl), current_state),
        0x7E => instruction_finished(ld_a_from_hl(&mut current_state.af, &mut current_state.hl, memory), current_state),
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

        0xC0 => conditional_ret(current_state, memory, JumpCondition::ZNotSet),
        0xC1 => instruction_finished(pop(&mut current_state.bc, &mut current_state.sp, memory), current_state),
        0xC2 => conditional_jump(JumpCondition::ZNotSet, memory, current_state),
        0xC3 => jump(memory, current_state),
        0xC4 => conditional_call(memory, current_state, JumpCondition::ZNotSet),
        0xC5 => instruction_finished(push(&mut current_state.bc, &mut current_state.sp, memory), current_state),
        0xC6 => instruction_finished(add_imm_to_a(&mut current_state.af, &current_state.pc.get(), memory), current_state),
        0xC7 => rst(0x0000, memory, current_state),
        0xC8 => conditional_ret(current_state, memory, JumpCondition::ZSet),
        0xC9 => ret(current_state, memory),
        0xCA => conditional_jump(JumpCondition::ZSet, memory, current_state),
        0xCB => result = CycleResult::InvalidOp, // Shouldn't have a CB at this stage, so mark as invalid if it happens.
        0xCC => conditional_call(memory, current_state, JumpCondition::ZSet),
        0xCD => call(memory, current_state),
        0xCE => instruction_finished(adc_imm_to_a(&mut current_state.af, &current_state.pc.get(), memory), current_state),
        0xCF => rst(0x0008, memory, current_state),

        0xD0 => conditional_ret(current_state, memory, JumpCondition::CNotSet),
        0xD1 => instruction_finished(pop(&mut current_state.de, &mut current_state.sp, memory), current_state),
        0xD2 => conditional_jump(JumpCondition::CNotSet, memory, current_state),
        0xD3 => result = CycleResult::InvalidOp,
        0xD4 => conditional_call(memory, current_state, JumpCondition::CNotSet),
        0xD5 => instruction_finished(push(&mut current_state.de, &mut current_state.sp, memory), current_state),
        0xD6 => instruction_finished(sub_imm_from_a(&mut current_state.af, memory, &current_state.pc.get()), current_state),
        0xD7 => rst(0x0010, memory, current_state),
        0xD8 => conditional_ret(current_state, memory, JumpCondition::CSet),
        0xD9 => reti(current_state, memory),
        0xDA => conditional_jump(JumpCondition::CSet, memory, current_state),
        0xDB => result = CycleResult::InvalidOp,
        0xDC => conditional_call(memory, current_state, JumpCondition::CSet),
        0xDD => result = CycleResult::InvalidOp,
        0xDE => instruction_finished(sbc_imm_from_a(&mut current_state.af, memory, &current_state.pc.get()), current_state),
        0xDF => rst(0x0017, memory, current_state),

        0xE0 => instruction_finished(save_a_to_ff_imm(&mut current_state.af, &mut current_state.pc.get(), memory), current_state),
        0xE1 => instruction_finished(pop(&mut current_state.hl, &mut current_state.sp, memory), current_state),
        0xE2 => instruction_finished(save_a_to_ff_c(&mut current_state.af, &mut current_state.bc, memory), current_state),
        0xE3 => result = CycleResult::InvalidOp,
        0xE4 => result = CycleResult::InvalidOp,
        0xE5 => instruction_finished(push(&mut current_state.hl, &mut current_state.sp, memory), current_state),
        0xE6 => instruction_finished(and_a_with_imm(&mut current_state.af, &current_state.pc.get(), memory), current_state),
        0xE7 => rst(0x0020, memory, current_state),
        0xE8 => instruction_finished(add_imm_to_sp(&mut current_state.af, &mut current_state.sp, &current_state.pc.get(), memory), current_state),
        0xE9 => jump_to_hl(current_state),
        0xEA => instruction_finished(save_a_to_nn(&mut current_state.af, &current_state.pc.get(), memory), current_state),
        0xEB => result = CycleResult::InvalidOp,
        0xEC => result = CycleResult::InvalidOp,
        0xED => result = CycleResult::InvalidOp,
        0xEE => instruction_finished(xor_a_with_imm(&mut current_state.af, &current_state.pc.get(), memory), current_state),
        0xEF => rst(0x0028, memory, current_state),

        0xF0 => instruction_finished(ld_a_from_ff_imm(&mut current_state.af, &mut current_state.pc.get(), memory), current_state),
        0xF1 => instruction_finished(pop(&mut current_state.af, &mut current_state.sp, memory), current_state),
        0xF2 => instruction_finished(ld_a_from_ff_c(&mut current_state.af, &mut current_state.bc, memory), current_state),
        0xF3 => instruction_finished(di(current_state), current_state),
        0xF4 => result = CycleResult::InvalidOp,
        0xF5 => instruction_finished(push(&mut current_state.af, &mut current_state.sp, memory), current_state),
        0xF6 => instruction_finished(or_a_with_imm(&mut current_state.af, &current_state.pc.get(), memory), current_state),
        0xF7 => rst(0x0030, memory, current_state),
        0xF8 => instruction_finished(add_imm_to_sp_ld_to_hl(current_state, memory), current_state),
        0xF9 => instruction_finished(ld_hl_into_sp(&mut current_state.sp, &mut current_state.hl), current_state),
        0xFA => instruction_finished(ld_a_from_imm_addr(&mut current_state.af, &current_state.pc.get(), memory), current_state),
        0xFB => instruction_finished(ei(current_state), current_state),
        0xFC => result = CycleResult::InvalidOp,
        0xFD => result = CycleResult::InvalidOp,
        0xFE => instruction_finished(cp_a_with_imm(&mut current_state.af, &current_state.pc.get(), memory), current_state),
        0xFF => rst(0x0038, memory, current_state),
    }

    result
}

fn instruction_finished(values: (u16, u16), state: &mut CpuState) {

    if state.halt_bug {
        state.cycles.add(values.1);
        state.halt_bug = false;
    }
    else {
        state.pc.add(values.0); state.cycles.add(values.1);
    }
}



// NOP

fn nop() -> (u16, u16) {

    (1, 4)
}

// DAA

fn daa(af: &mut CpuReg) -> (u16, u16) {

    let value = af.get_register_lb();

    if !utils::get_nf(af) {
        
        if utils::get_cf(af) || value > 0x99 {
            let new_value = value.overflowing_sub(0x60);
            af.set_register_lb(new_value.0);
            utils::set_cf(true, af);
        }
        else if utils::get_hf(af) || (value & 0x0F) > 0x09 {
            let new_value = value.overflowing_sub(0x6);
            af.set_register_lb(new_value.0);
        }
    }
    else {

        if utils::get_cf(af) {
            let new_value = value.overflowing_sub(0x60);
            af.set_register_lb(new_value.0);
        }
        else if utils::get_hf(af) {
            let new_value = value.overflowing_sub(0x6);
            af.set_register_lb(new_value.0);
        }
    }

    utils::set_zf(af.get_register_lb() == 0, af);
    utils::set_hf(false, af);

    (1, 4)
}

// HALT and STOP

fn halt(current_state: &mut CpuState, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> CycleResult {

    current_state.pc.add(1);
    current_state.cycles.add(4);
    current_state.halt_bug = cpu::memory_read_u8(0xFF0F, memory) != 0 && !current_state.interrupts.can_interrupt;
    CycleResult::Halt
}

fn stop(current_state: &mut CpuState) -> CycleResult {

    current_state.pc.add(2);
    current_state.cycles.add(4);
    CycleResult::Stop
}


// Jumps

fn jump(memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), state: &mut CpuState) {

    let current_pc = state.pc.get();
    state.pc.set(cpu::memory_read_u16(current_pc + 1, memory));
    state.cycles.add(16);
}

fn jump_to_hl(state: &mut CpuState) {

    let target = state.hl.get_register();
    state.pc.set(target);
    state.cycles.add(4);
}

fn relative_jump(memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), state: &mut CpuState) {

    let current_pc = state.pc.get();
    let target = cpu::memory_read_u8(current_pc + 1, memory) as i8;
    state.pc.set(current_pc.wrapping_add(target as u16) + 2);
    state.cycles.add(12);
}

fn conditional_jump(condition: JumpCondition, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), state: &mut CpuState) {

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

fn conditional_relative_jump(condition: JumpCondition, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), state: &mut CpuState) {

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


// Calls and Returns

fn call(memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), state: &mut CpuState) {

    let current_pc = state.pc.get();
    let next_pc = state.pc.get() + 3;

    cpu::stack_write(&mut state.sp, next_pc, &memory.0);
    state.pc.set(cpu::memory_read_u16(current_pc + 1, memory));
    state.cycles.add(24);
}

fn conditional_call(memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), state: &mut CpuState, condition: JumpCondition) {

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

fn ret(state: &mut CpuState, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) {
    
    state.pc.set(cpu::stack_read(&mut state.sp, memory));
    state.cycles.add(16);
}

fn reti(state: &mut CpuState, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) {

    cpu::toggle_interrupts(state, true);
    ret(state, memory);
}

fn conditional_ret(state: &mut CpuState, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), condition: JumpCondition) {

    let should_ret: bool;

    match condition {

        JumpCondition::ZNotSet => should_ret = !utils::check_bit(state.af.get_register_rb(), 7),
        JumpCondition::CNotSet => should_ret = !utils::check_bit(state.af.get_register_rb(), 4),
        JumpCondition::ZSet => should_ret = utils::check_bit(state.af.get_register_rb(), 7),
        JumpCondition::CSet => should_ret = utils::check_bit(state.af.get_register_rb(), 4),
    }

    if should_ret { ret(state, memory);}
    else { state.pc.add(1); state.cycles.add(8) }
}


// Load register to register

fn ld_hi_into_low(target: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    target.set_register_rb(source.get_register_lb());
    (1, 4)
}

fn ld_hi_into_hi(target: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {
    
    target.set_register_lb(source.get_register_lb());
    (1, 4)
}

fn ld_low_into_low(target: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    target.set_register_rb(source.get_register_rb());
    (1, 4)
}

fn ld_low_into_hi(target: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {
    
    target.set_register_lb(source.get_register_rb());
    (1, 4)
}

fn ld_self_low_to_hi(reg: &mut CpuReg) -> (u16, u16) {

    let value = reg.get_register_rb();
    reg.set_register_lb(value);
    (1, 4)
}

fn ld_self_hi_to_low(reg: &mut CpuReg) -> (u16, u16) {

    let value = reg.get_register_lb();
    reg.set_register_rb(value);
    (1, 4)
}

fn add_imm_to_sp_ld_to_hl(state: &mut CpuState, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(state.pc.get() + 1, memory) as i8;
    let result = state.sp.get_register().overflowing_add(value as u16);

    state.hl.set_register(result.0);

    utils::set_zf(false, &mut state.af);
    utils::set_nf(false, &mut state.af);
    utils::set_cf(result.1, &mut state.af);
    (2, 12)
}


// Load register from immediate

fn ld_imm_into_hi(target: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), pc: &u16) -> (u16, u16) {
    
    target.set_register_lb(cpu::memory_read_u8(pc + 1, memory));
    (2, 8)
}

fn ld_imm_into_low(target: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), pc: &u16) -> (u16, u16) {

    target.set_register_rb(cpu::memory_read_u8(pc + 1, memory));
    (2, 8)
}

fn ld_imm_into_full(target_reg: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), pc: &u16) -> (u16, u16) {

    target_reg.set_register(cpu::memory_read_u16(pc + 1, memory));
    (3, 12)
}


// Load register from address

fn ld_a_from_imm_addr(af: &mut CpuReg, pc: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let target_addr = cpu::memory_read_u16(pc + 1, memory);
    af.set_register_lb(cpu::memory_read_u8(target_addr, memory));
    (3, 16)
}

fn ld_a_from_ff_imm(af: &mut CpuReg, pc: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let target_addr = 0xFF00 + cpu::memory_read_u8(pc + 1, memory) as u16;
    af.set_register_lb(cpu::memory_read_u8(target_addr, memory));
    (2, 12)
}

fn ld_a_from_ff_c(af: &mut CpuReg, bc: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let address = 0xFF00 + bc.get_register_rb() as u16;
    let value = cpu::memory_read_u8(address, memory);

    af.set_register_lb(value);

    (1, 8)
}

// Load register from HL

fn ld_hi_from_hl(reg: &mut CpuReg, full: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    reg.set_register_lb(cpu::memory_read_u8(full.get_register(), memory));
    (1, 8)
}

fn ld_low_from_hl(reg: &mut CpuReg, full: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    reg.set_register_rb(cpu::memory_read_u8(full.get_register(), memory));
    (1, 8)
}

fn ld_h_from_hl(hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let hl_value = hl.get_register();
    let value = cpu::memory_read_u8(hl_value, memory);
    hl.set_register_lb(value);
    (1, 8)
}

fn ld_l_from_hl(hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let addr = hl.get_register();
    hl.set_register_rb(cpu::memory_read_u8(addr, memory));
    (1, 8)
}

fn ld_a_from_hl(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    af.set_register_lb(cpu::memory_read_u8(hl.get_register(), memory));
    (1, 8)
}

fn ld_a_from_hl_inc(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    af.set_register_lb(cpu::memory_read_u8(hl.get_register(), memory));
    hl.increment();
    (1, 8)
}

fn ld_a_from_hl_dec(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {
    
    af.set_register_lb(cpu::memory_read_u8(hl.get_register(), memory));
    hl.decrement();
    (1, 8)
}


// Save register to HL

fn save_a_to_hl_inc(a: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    cpu::memory_write(hl.get_register(), a.get_register_lb(), &memory.0);
    hl.increment();
    (1, 8)
}

fn save_a_to_hl_dec(a: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    cpu::memory_write(hl.get_register(), a.get_register_lb(), &memory.0);
    hl.decrement();
    (1, 8)
}

fn save_hi_to_hl(reg: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    cpu::memory_write(hl.get_register(), reg.get_register_lb(), &memory.0);
    (1, 8)
}

fn save_low_to_hl(reg: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    cpu::memory_write(hl.get_register(), reg.get_register_rb(), &memory.0);
    (1, 8)
}

fn save_h_to_hl(hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    cpu::memory_write(hl.get_register(), hl.get_register_lb(), &memory.0);
    (1, 8)
}

fn save_l_to_hl(hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    cpu::memory_write(hl.get_register(), hl.get_register_rb(), &memory.0);
    (1, 8)
}

fn ld_hl_into_sp(sp: &mut CpuReg, hl: &mut CpuReg) -> (u16, u16) {

    sp.set_register(hl.get_register());
    (1, 8)
}

fn save_a_to_full(a: &mut CpuReg, full: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    cpu::memory_write(full.get_register(), a.get_register_lb(), &memory.0);
    (1, 8)
}


// Save register to address

fn save_a_to_ff_imm(af: &mut CpuReg, pc: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let target_addr = 0xFF00 + (cpu::memory_read_u8(pc + 1, memory) as u16);
    cpu::memory_write(target_addr, af.get_register_lb(), &memory.0);
    (2, 12)
}

fn save_a_to_ff_c(af: &mut CpuReg, bc: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let target_addr = 0xFF00 + (bc.get_register_rb() as u16);
    cpu::memory_write(target_addr, af.get_register_lb(), &memory.0);
    (1, 8)
}

fn save_a_to_nn(af: &mut CpuReg, pc: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let target_addr = cpu::memory_read_u16(pc + 1, memory);
    cpu::memory_write(target_addr, af.get_register_lb(), &memory.0);
    (3, 16)
}


// Save value to HL

fn save_imm_to_hl(hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), pc: &u16) -> (u16, u16) {

    let value = cpu::memory_read_u8(pc + 1, memory);
    cpu::memory_write(hl.get_register(), value, &memory.0);
    (2, 12)
}


// Save SP to immediate address

fn save_sp_to_imm(sp: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), pc: &u16) -> (u16, u16) {

    let target_addr = cpu::memory_read_u16(pc + 1, memory);
    cpu::memory_write(target_addr, sp.get_register_rb(), &memory.0);
    cpu::memory_write(target_addr + 1, sp.get_register_lb(), &memory.0);
    (3, 20)
}


// Increment registers

fn increment_full(reg: &mut CpuReg) -> (u16, u16) {

    reg.increment();
    (1, 8)
}

fn increment_lb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u16) {

    reg.increment_lb();
    let half_carry = (reg.get_register_lb() & 0x0F) == 0;
    utils::set_zf(reg.get_register_lb() == 0, af); utils::set_nf(false, af);
    utils::set_hf(half_carry, af);
    (1, 4)
}

fn increment_rb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u16) {
    
    reg.increment_rb();
    let half_carry = (reg.get_register_rb() & 0x0F) == 0;
    utils::set_zf(reg.get_register_rb() == 0, af); utils::set_nf(false, af);
    utils::set_hf(half_carry, af);
    (1, 4)
}

fn increment_a(af: &mut CpuReg) -> (u16, u16) {

    af.increment_lb();
    let half_carry = (af.get_register_lb() & 0x0F) == 0;
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(false, af);
    utils::set_hf(half_carry, af);
    (1, 4)
}


// Increment value at HL

fn increment_value(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {
    
    let value = cpu::memory_read_u8(hl.get_register(), memory);
    let result = value.overflowing_add(1);
    let half_carry = (result.0 & 0x0F) == 0;
    cpu::memory_write(hl.get_register(), result.0, &memory.0);
    utils::set_zf(result.0 == 0, af); utils::set_nf(false, af);
    utils::set_hf(half_carry, af);
    (1, 12)
}


// Decrement registers

fn decrement_full(reg: &mut CpuReg) -> (u16, u16) {
    
    reg.decrement();
    (1, 4)
}

fn decrement_lb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u16) {
    
    reg.decrement_lb();
    let half_carry = (reg.get_register_lb() & 0x0F) == 0;
    utils::set_zf(reg.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_hf(half_carry, af);
    (1, 4)
}

fn decrement_rb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u16) {
    
    reg.decrement_rb();
    let half_carry = (reg.get_register_rb() & 0x0F) == 0;
    utils::set_zf(reg.get_register_rb() == 0, af); utils::set_nf(true, af);
    utils::set_hf(half_carry, af);
    (1, 4)
}

fn decrement_a(af: &mut CpuReg) -> (u16, u16) {
    
    af.decrement_lb();
    let half_carry = (af.get_register_lb() & 0x0F) == 0;
    utils::set_zf(af.get_register_lb() == 0, af); utils::set_nf(true, af);
    utils::set_hf(half_carry, af);
    (1, 4)
}


// Decrement value at HL

fn decrement_at_hl(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {
    
    let value = cpu::memory_read_u8(hl.get_register(), memory);
    let result = value.overflowing_sub(1);
    let half_carry = (result.0 & 0x0F) == 0;
    cpu::memory_write(hl.get_register(), result.0, &memory.0);
    utils::set_zf(result.0 == 0, af); utils::set_nf(true, af);
    utils::set_hf(half_carry, af);
    (1, 12)
}


// Add value to Registers

fn add_full(target: &mut CpuReg, source: &mut CpuReg, af: &mut CpuReg) -> (u16, u16) {

    let value = source.get_register();
    let half_carry = utils::check_half_carry_u16((&target.get_register(), &source.get_register()));
    let overflow = target.add_to_reg(value);
    utils::set_nf(false, af);
    utils::set_hf(half_carry, af);
    utils::set_cf(overflow, af);
    (1, 8)
}

fn add_hl_to_hl(hl: &mut CpuReg, af: &mut CpuReg) -> (u16, u16) {

    let hl_value = hl.get_register();
    let half_carry = utils::check_half_carry_u16((&hl_value, &hl_value));
    let overflow = hl.add_to_reg(hl_value);
    utils::set_nf(false, af); utils::set_cf(overflow, af);
    utils::set_hf(half_carry, af);
    (1, 8)
}

fn add_imm_to_sp(af: &mut CpuReg, sp: &mut CpuReg, pc: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(pc + 1, memory) as i8;
    sp.add_to_reg(value as u16);
    utils::set_zf(false, af);
    utils::set_nf(false, af);
    // TODO: Actually check the values for this flags.
    utils::set_cf(false, af);
    utils::set_cf(false, af);
    (2, 16)
}

fn add(af: &mut CpuReg, value: u8) -> u8 {

    let half_carry = utils::check_half_carry_u8((&af.get_register_lb(), &value));
    let result = af.get_register_lb().overflowing_add(value);

    utils::set_zf(result.0 == 0, af);
    utils::set_nf(false, af);
    utils::set_hf(half_carry, af);
    utils::set_cf(result.1, af);

    result.0
}

fn add_hi_to_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = add(af, source.get_register_lb());
    af.set_register_lb(result);
    (1, 4)
}

fn add_low_to_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = add(af, source.get_register_rb());
    af.set_register_lb(result);
    (1, 4)
}

fn add_a_to_a(af: &mut CpuReg) -> (u16, u16) {

    let value = af.get_register_lb();
    let result = add(af, value);
    af.set_register_lb(result);
    (1, 4)
}

fn add_val_to_a(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(hl.get_register(), memory);
    let result = add(af, value);
    af.set_register_lb(result);
    (1, 4)
}

fn add_imm_to_a(af: &mut CpuReg, pc: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(pc + 1, memory);
    let result = add(af, value);
    af.set_register_lb(result);
    (2, 8)
}


// ADC opcodes

fn adc(af: &mut CpuReg, value: u8) -> u8 {

    let carry = utils::get_carry(af);
    let half_carry = utils::check_half_carry_u8((&af.get_register_lb(), &value));
    let mut carry_results = (false, false);
    let mut result = af.get_register_lb().overflowing_add(value);
    carry_results.0 = result.1;
    result = result.0.overflowing_add(carry);
    carry_results.1 = result.1;

    utils::set_zf(result.0 == 0, af);
    utils::set_nf(false, af);
    utils::set_hf(half_carry, af);
    utils::set_cf(carry_results.0 || carry_results.1, af);

    result.0
}

fn adc_hi_to_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = adc(af, source.get_register_lb());
    af.set_register_lb(result);
    (1, 4)
}

fn adc_low_to_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = adc(af, source.get_register_rb());
    af.set_register_lb(result);
    (1, 4)
}

fn adc_a_to_a(af: &mut CpuReg) -> (u16, u16) {

    let value = af.get_register_lb();
    let result = adc(af, value);
    af.set_register_lb(result);
    (1, 4)
}

fn adc_val_to_a(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(hl.get_register(), memory);
    let result = adc(af, value);
    af.set_register_lb(result);
    (1, 4)
}

fn adc_imm_to_a(af: &mut CpuReg, pc: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(pc + 1, memory);
    let result = adc(af, value);
    af.set_register_lb(result);
    (2, 8)
}


// Substract value from registers

fn sub(af: &mut CpuReg, value: u8) -> u8 {

    let half_carry = utils::check_half_borrow((af.get_register_lb(), value));
    let result = af.get_register_lb().overflowing_sub(value);

    utils::set_zf(result.0 == 0, af);
    utils::set_nf(true, af);
    utils::set_hf(half_carry, af);
    utils::set_cf(result.1, af);

    result.0
}

fn sub_hi_from_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = sub(af, source.get_register_lb());
    af.set_register_lb(result);
    (1, 4)
}

fn sub_low_from_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = sub(af, source.get_register_rb());
    af.set_register_lb(result);
    (1, 4)
}

fn sub_a_from_a(af: &mut CpuReg) -> (u16, u16) {

    let value = af.get_register_lb();
    let result = sub(af, value);
    af.set_register_lb(result);
    (1, 4)
}

fn sub_val_from_a(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(hl.get_register(), memory);
    let result = sub(af, value);
    af.set_register_lb(result);
    (1, 4)
}

fn sub_imm_from_a(af: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), pc: &u16) -> (u16, u16) {

    let value = cpu::memory_read_u8(pc + 1, memory);
    let result = sub(af, value);
    af.set_register_lb(result);
    (2, 8)
}


// SBC opcodes

fn sbc(af: &mut CpuReg, value: u8) -> u8 {

    let carry = utils::get_carry(af);
    let half_carry = utils::check_half_borrow((af.get_register_lb(), value));
    let mut borrow_results = (false, false);
    let mut result = af.get_register_lb().overflowing_sub(value);
    borrow_results.0 = result.1;
    result = result.0.overflowing_sub(carry);
    borrow_results.1 = result.1;

    utils::set_zf(result.0 == 0, af);
    utils::set_nf(true, af);
    utils::set_hf(half_carry, af);
    utils::set_cf(borrow_results.0 || borrow_results.1, af);

    result.0
}

fn sbc_hi_from_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = sbc(af, source.get_register_lb());
    af.set_register_lb(result);
    (1, 4)
}

fn sbc_low_from_a(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = sbc(af, source.get_register_rb());
    af.set_register_lb(result);
    (1, 4)
}

fn sbc_a_from_a(af: &mut CpuReg) -> (u16, u16) {

    let value = af.get_register_lb();
    let result = sbc(af, value);
    af.set_register_lb(result);
    (1, 4)
}

fn sbc_val_from_a(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(hl.get_register(), memory);
    let result = sbc(af, value);
    af.set_register_lb(result);
    (1, 4)
}

fn sbc_imm_from_a(af: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), pc: &u16) -> (u16, u16) {

    let value = cpu::memory_read_u8(pc + 1, memory);
    let result = sbc(af, value);
    af.set_register_lb(result);
    (2, 8)
}


// AND opcodes

fn and(af: &mut CpuReg, value: u8) -> u8 {

    let result = af.get_register_lb() & value;

    utils::set_zf(result == 0, af);
    utils::set_nf(false, af);
    utils::set_hf(true, af);
    utils::set_cf(false, af);
    
    result
}

fn and_a_with_hi(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = and(af, source.get_register_lb());
    af.set_register_lb(result);
    (1, 4)
}

fn and_a_with_low(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = and(af, source.get_register_rb());
    af.set_register_lb(result);
    (1, 4)
}

fn and_a_with_a(af: &mut CpuReg) -> (u16, u16) {

    let reg_value = af.get_register_lb();
    let result = and(af, reg_value);
    af.set_register_lb(result);
    (1, 4)
}

fn and_a_with_value(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(hl.get_register(), memory);
    let result = and(af, value);
    af.set_register_lb(result);
    (1, 4)
}

fn and_a_with_imm(af: &mut CpuReg, pc: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(pc + 1, memory);
    let result = and(af, value);
    af.set_register_lb(result);
    (2, 8)
}


// OR opcodes

fn or(af: &mut CpuReg, value: u8) -> u8 {

    let result = af.get_register_lb() | value;

    utils::set_zf(result == 0, af);
    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(false, af);

    result
}

fn or_a_with_hi(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = or(af, source.get_register_lb());
    af.set_register_lb(result);
    (1, 4)
}

fn or_a_with_low(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = or(af, source.get_register_rb());
    af.set_register_lb(result);
    (1, 4)
}

fn or_a_with_a(af: &mut CpuReg) -> (u16, u16) {

    let reg_value = af.get_register_lb();
    let result = or(af, reg_value);
    af.set_register_lb(result);
    (1, 4)
}

fn or_a_with_value(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(hl.get_register(), memory);
    let result = or(af, value);
    af.set_register_lb(result);
    (1, 4)
}

fn or_a_with_imm(af: &mut CpuReg, pc: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(pc + 1, memory);
    let result = or(af, value);
    af.set_register_lb(result);
    (2, 8)
}


// XOR opcodes

fn xor(af: &mut CpuReg, value: u8) -> u8 {

    let result = af.get_register_lb() ^ value;

    utils::set_zf(result == 0, af);
    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(false, af);

    result
}

fn xor_a_with_hi(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = xor(af, source.get_register_lb());
    af.set_register_lb(result);
    (1, 4)
}

fn xor_a_with_low(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    let result = xor(af, source.get_register_rb());
    af.set_register_lb(result);
    (1, 4)
}

fn xor_a_with_a(af: &mut CpuReg) -> (u16, u16) {

    let result = af.get_register_lb() ^ af.get_register_lb();
    af.set_register_lb(result);
    utils::set_zf(result == 0, af); utils::set_nf(false, af);
    utils::set_hf(false, af); utils::set_cf(false, af);
    (1, 4)
}

fn xor_a_with_value(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(hl.get_register(), memory);
    let result = xor(af, value);
    af.set_register_lb(result);
    (1, 4)
}

fn xor_a_with_imm(af: &mut CpuReg, pc: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(pc + 1, memory);
    let result = xor(af, value);
    af.set_register_lb(result);
    (2, 8)
}


// Complement (logical NOT) A

fn cpl(af: &mut CpuReg) -> (u16, u16) {

    let value = !af.get_register_lb();
    af.set_register_lb(value);
    utils::set_nf(true, af);
    utils::set_hf(true, af);
    (1, 4)
}


// CP opcodes

fn cp(af: &mut CpuReg, value: u8) {

    let half_carry = utils::check_half_borrow((af.get_register_lb(), value));
    let result = af.get_register_lb().overflowing_sub(value);

    utils::set_zf(result.0 == 0, af);
    utils::set_nf(true, af);
    utils::set_hf(half_carry, af);
    utils::set_cf(result.1, af);
}

fn cp_a_with_hi(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {

    cp(af, source.get_register_lb());
    (1, 4)
}

fn cp_a_with_low(af: &mut CpuReg, source: &mut CpuReg) -> (u16, u16) {
    
    cp(af, source.get_register_rb());
    (1, 4)
}

fn cp_a_with_a(af: &mut CpuReg) -> (u16, u16) {

    let value = af.get_register_lb();
    cp(af, value);
    (1, 4)
}

fn cp_a_with_value(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(hl.get_register(), memory);
    cp(af, value);
    (1, 8)
}

fn cp_a_with_imm(af: &mut CpuReg, pc: &u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::memory_read_u8(pc + 1, memory);
    cp(af, value);
    (2, 8)
}


// Push and Pop

fn pop(reg: &mut CpuReg, sp: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    let value = cpu::stack_read(sp, memory);
    reg.set_register(value);
    (1, 12)
}

fn push(reg: &mut CpuReg, sp: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u16) {

    cpu::stack_write(sp, reg.get_register(), &memory.0);
    (1, 16)
}


// Rotation opcodes

fn rla(af: &mut CpuReg) -> (u16, u16) {

    let mut value = af.get_register_lb();
    let old_carry = utils::get_carry(af);

    utils::set_cf(utils::check_bit(value, 7), af);
    utils::set_hf(false, af);
    utils::set_nf(false, af);
    utils::set_zf(false, af);

    value = value << 1;
    af.set_register_lb(value | old_carry);
    (1, 4)
}

fn rlc_a(af: &mut CpuReg) -> (u16, u16) {

    let carry = utils::check_bit(af.get_register_lb(), 7);
    let result = af.get_register_lb().rotate_left(1);

    af.set_register_lb(result);
    utils::set_zf(false, af);
    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(carry, af);
    (1, 4)
}

fn rr_a(af: &mut CpuReg) -> (u16, u16) {

    let will_carry = utils::check_bit(af.get_register_lb(), 0);
    let old_carry = utils::get_carry(af);
    let mut result = af.get_register_lb() >> 1;
    result = result | (old_carry << 7);
    af.set_register_lb(result);

    utils::set_cf(will_carry, af);
    utils::set_hf(false, af);
    utils::set_nf(false, af);
    utils::set_zf(result == 0, af);
    
    (1, 4)
}

fn rrc_a(af: &mut CpuReg) -> (u16, u16) {

    let carry = utils::check_bit(af.get_register_lb(), 0);
    let result = af.get_register_lb().rotate_right(1);

    af.set_register_lb(result);
    utils::set_zf(false, af);
    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(carry, af);
    (2, 8)
}


// Enable/Disable interrupts

fn ei(state: &mut CpuState) -> (u16, u16) {

    cpu::toggle_interrupts(state, true);
    (1, 4)
}

fn di(state: &mut CpuState) -> (u16, u16) {

    cpu::toggle_interrupts(state, false);
    (1, 4)
}


// Set/complement flags

fn scf(af: &mut CpuReg) -> (u16, u16) {

    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(true, af);
    (1, 4)
}

fn ccf(af: &mut CpuReg) -> (u16, u16) {
    
    let carry = utils::get_carry(af) == 1;
    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(!carry, af);
    (1, 4)
}


// Reset opcode

fn rst(target: u16, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), state: &mut CpuState) {

    cpu::stack_write(&mut state.sp, state.pc.get() + 1, &memory.0);
    state.cycles.add(32);
    state.pc.set(target);
}