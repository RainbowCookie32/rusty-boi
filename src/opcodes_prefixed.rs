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

pub fn run_prefixed_instruction(current_state: &mut CpuState, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>), opcode: u8) -> CycleResult {

    let result = CycleResult::Success;

    match opcode {

        0x00 => instruction_finished(rlc_lb(&mut current_state.af, &mut current_state.bc), current_state),
        0x01 => instruction_finished(rlc_rb(&mut current_state.af, &mut current_state.bc), current_state),
        0x02 => instruction_finished(rlc_lb(&mut current_state.af, &mut current_state.de), current_state),
        0x03 => instruction_finished(rlc_rb(&mut current_state.af, &mut current_state.de), current_state),
        0x04 => instruction_finished(rlc_lb(&mut current_state.af, &mut current_state.hl), current_state),
        0x05 => instruction_finished(rlc_rb(&mut current_state.af, &mut current_state.hl), current_state),
        0x06 => instruction_finished(rlc_hl(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x07 => instruction_finished(rlc_a(&mut current_state.af), current_state),
        0x08 => instruction_finished(rrc_lb(&mut current_state.af, &mut current_state.bc), current_state),
        0x09 => instruction_finished(rrc_rb(&mut current_state.af, &mut current_state.bc), current_state),
        0x0A => instruction_finished(rrc_lb(&mut current_state.af, &mut current_state.de), current_state),
        0x0B => instruction_finished(rrc_rb(&mut current_state.af, &mut current_state.de), current_state),
        0x0C => instruction_finished(rrc_lb(&mut current_state.af, &mut current_state.hl), current_state),
        0x0D => instruction_finished(rrc_rb(&mut current_state.af, &mut current_state.hl), current_state),
        0x0E => instruction_finished(rrc_hl(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x0F => instruction_finished(rrc_a(&mut current_state.af), current_state),
        
        0x10 => instruction_finished(rl_lb(&mut current_state.bc, &mut current_state.af), current_state),
        0x11 => instruction_finished(rl_rb(&mut current_state.bc, &mut current_state.af), current_state),
        0x12 => instruction_finished(rl_lb(&mut current_state.de, &mut current_state.af), current_state),
        0x13 => instruction_finished(rl_rb(&mut current_state.de, &mut current_state.af), current_state),
        0x14 => instruction_finished(rl_lb(&mut current_state.hl, &mut current_state.af), current_state),
        0x15 => instruction_finished(rl_rb(&mut current_state.hl, &mut current_state.af), current_state),
        0x16 => instruction_finished(rl_hl(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x17 => instruction_finished(rl_a(&mut current_state.af), current_state),
        0x18 => instruction_finished(rr_lb(&mut current_state.bc, &mut current_state.af), current_state),
        0x19 => instruction_finished(rr_rb(&mut current_state.bc, &mut current_state.af), current_state),
        0x1A => instruction_finished(rr_lb(&mut current_state.de, &mut current_state.af), current_state),
        0x1B => instruction_finished(rr_rb(&mut current_state.de, &mut current_state.af), current_state),
        0x1C => instruction_finished(rr_lb(&mut current_state.hl, &mut current_state.af), current_state),
        0x1D => instruction_finished(rr_rb(&mut current_state.hl, &mut current_state.af), current_state),
        0x1E => instruction_finished(rr_hl(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x1F => instruction_finished(rr_a(&mut current_state.af), current_state),

        0x20 => instruction_finished(sla_lb(&mut current_state.af, &mut current_state.bc), current_state),
        0x21 => instruction_finished(sla_rb(&mut current_state.af, &mut current_state.bc), current_state),
        0x22 => instruction_finished(sla_lb(&mut current_state.af, &mut current_state.de), current_state),
        0x23 => instruction_finished(sla_rb(&mut current_state.af, &mut current_state.de), current_state),
        0x24 => instruction_finished(sla_lb(&mut current_state.af, &mut current_state.hl), current_state),
        0x25 => instruction_finished(sla_rb(&mut current_state.af, &mut current_state.hl), current_state),
        0x26 => instruction_finished(sla_val(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x27 => instruction_finished(sla_a(&mut current_state.af), current_state),
        0x28 => instruction_finished(sra_lb(&mut current_state.af, &mut current_state.bc), current_state),
        0x29 => instruction_finished(sra_rb(&mut current_state.af, &mut current_state.bc), current_state),
        0x2A => instruction_finished(sra_lb(&mut current_state.af, &mut current_state.de), current_state),
        0x2B => instruction_finished(sra_rb(&mut current_state.af, &mut current_state.de), current_state),
        0x2C => instruction_finished(sra_lb(&mut current_state.af, &mut current_state.hl), current_state),
        0x2D => instruction_finished(sra_rb(&mut current_state.af, &mut current_state.hl), current_state),
        0x2E => instruction_finished(sra_val(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x2F => instruction_finished(sra_a(&mut current_state.af), current_state),

        0x30 => instruction_finished(swap_lb(&mut current_state.af, &mut current_state.bc), current_state),
        0x31 => instruction_finished(swap_rb(&mut current_state.af, &mut current_state.bc), current_state),
        0x32 => instruction_finished(swap_lb(&mut current_state.af, &mut current_state.de), current_state),
        0x33 => instruction_finished(swap_rb(&mut current_state.af, &mut current_state.de), current_state),
        0x34 => instruction_finished(swap_lb(&mut current_state.af, &mut current_state.hl), current_state),
        0x35 => instruction_finished(swap_rb(&mut current_state.af, &mut current_state.hl), current_state),
        0x36 => instruction_finished(swap_hl(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x37 => instruction_finished(swap_a(&mut current_state.af), current_state),
        0x38 => instruction_finished(srl_lb(&mut current_state.af, &mut current_state.bc), current_state),
        0x39 => instruction_finished(srl_rb(&mut current_state.af, &mut current_state.bc), current_state),
        0x3A => instruction_finished(srl_lb(&mut current_state.af, &mut current_state.de), current_state),
        0x3B => instruction_finished(srl_rb(&mut current_state.af, &mut current_state.de), current_state),
        0x3C => instruction_finished(srl_lb(&mut current_state.af, &mut current_state.hl), current_state),
        0x3D => instruction_finished(srl_rb(&mut current_state.af, &mut current_state.hl), current_state),
        0x3E => instruction_finished(srl_val(&mut current_state.af, &mut current_state.hl, memory), current_state),
        0x3F => instruction_finished(srl_a(&mut current_state.af), current_state),
        
        0x40 => instruction_finished(bit_lb(&mut current_state.bc, 0, &mut current_state.af), current_state),
        0x41 => instruction_finished(bit_rb(&mut current_state.bc, 0, &mut current_state.af), current_state),
        0x42 => instruction_finished(bit_lb(&mut current_state.de, 0, &mut current_state.af), current_state),
        0x43 => instruction_finished(bit_rb(&mut current_state.de, 0, &mut current_state.af), current_state),
        0x44 => instruction_finished(bit_lb(&mut current_state.hl, 0, &mut current_state.af), current_state),
        0x45 => instruction_finished(bit_rb(&mut current_state.hl, 0, &mut current_state.af), current_state),
        0x46 => instruction_finished(bit_hl(0, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x47 => instruction_finished(bit_a(&mut current_state.af, 0), current_state),
        0x48 => instruction_finished(bit_lb(&mut current_state.bc, 1, &mut current_state.af), current_state),
        0x49 => instruction_finished(bit_rb(&mut current_state.bc, 1, &mut current_state.af), current_state),
        0x4A => instruction_finished(bit_lb(&mut current_state.de, 1, &mut current_state.af), current_state),
        0x4B => instruction_finished(bit_rb(&mut current_state.de, 1, &mut current_state.af), current_state),
        0x4C => instruction_finished(bit_lb(&mut current_state.hl, 1, &mut current_state.af), current_state),
        0x4D => instruction_finished(bit_rb(&mut current_state.hl, 1, &mut current_state.af), current_state),
        0x4E => instruction_finished(bit_hl(1, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x4F => instruction_finished(bit_a(&mut current_state.af, 1), current_state),

        0x50 => instruction_finished(bit_lb(&mut current_state.bc, 2, &mut current_state.af), current_state),
        0x51 => instruction_finished(bit_rb(&mut current_state.bc, 2, &mut current_state.af), current_state),
        0x52 => instruction_finished(bit_lb(&mut current_state.de, 2, &mut current_state.af), current_state),
        0x53 => instruction_finished(bit_rb(&mut current_state.de, 2, &mut current_state.af), current_state),
        0x54 => instruction_finished(bit_lb(&mut current_state.hl, 2, &mut current_state.af), current_state),
        0x55 => instruction_finished(bit_rb(&mut current_state.hl, 2, &mut current_state.af), current_state),
        0x56 => instruction_finished(bit_hl(2, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x57 => instruction_finished(bit_a(&mut current_state.af, 2), current_state),
        0x58 => instruction_finished(bit_lb(&mut current_state.bc, 3, &mut current_state.af), current_state),
        0x59 => instruction_finished(bit_rb(&mut current_state.bc, 3, &mut current_state.af), current_state),
        0x5A => instruction_finished(bit_lb(&mut current_state.de, 3, &mut current_state.af), current_state),
        0x5B => instruction_finished(bit_rb(&mut current_state.de, 3, &mut current_state.af), current_state),
        0x5C => instruction_finished(bit_lb(&mut current_state.hl, 3, &mut current_state.af), current_state),
        0x5D => instruction_finished(bit_rb(&mut current_state.hl, 3, &mut current_state.af), current_state),
        0x5E => instruction_finished(bit_hl(3, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x5F => instruction_finished(bit_a(&mut current_state.af, 3), current_state),

        0x60 => instruction_finished(bit_lb(&mut current_state.bc, 4, &mut current_state.af), current_state),
        0x61 => instruction_finished(bit_rb(&mut current_state.bc, 4, &mut current_state.af), current_state),
        0x62 => instruction_finished(bit_lb(&mut current_state.de, 4, &mut current_state.af), current_state),
        0x63 => instruction_finished(bit_rb(&mut current_state.de, 4, &mut current_state.af), current_state),
        0x64 => instruction_finished(bit_lb(&mut current_state.hl, 4, &mut current_state.af), current_state),
        0x65 => instruction_finished(bit_rb(&mut current_state.hl, 4, &mut current_state.af), current_state),
        0x66 => instruction_finished(bit_hl(4, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x67 => instruction_finished(bit_a(&mut current_state.af, 4), current_state),
        0x68 => instruction_finished(bit_lb(&mut current_state.bc, 5, &mut current_state.af), current_state),
        0x69 => instruction_finished(bit_rb(&mut current_state.bc, 5, &mut current_state.af), current_state),
        0x6A => instruction_finished(bit_lb(&mut current_state.de, 5, &mut current_state.af), current_state),
        0x6B => instruction_finished(bit_rb(&mut current_state.de, 5, &mut current_state.af), current_state),
        0x6C => instruction_finished(bit_lb(&mut current_state.hl, 5, &mut current_state.af), current_state),
        0x6D => instruction_finished(bit_rb(&mut current_state.hl, 5, &mut current_state.af), current_state),
        0x6E => instruction_finished(bit_hl(5, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x6F => instruction_finished(bit_a(&mut current_state.af, 5), current_state),

        0x70 => instruction_finished(bit_lb(&mut current_state.bc, 6, &mut current_state.af), current_state),
        0x71 => instruction_finished(bit_rb(&mut current_state.bc, 6, &mut current_state.af), current_state),
        0x72 => instruction_finished(bit_lb(&mut current_state.de, 6, &mut current_state.af), current_state),
        0x73 => instruction_finished(bit_rb(&mut current_state.de, 6, &mut current_state.af), current_state),
        0x74 => instruction_finished(bit_lb(&mut current_state.hl, 6, &mut current_state.af), current_state),
        0x75 => instruction_finished(bit_rb(&mut current_state.hl, 6, &mut current_state.af), current_state),
        0x76 => instruction_finished(bit_hl(6, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x77 => instruction_finished(bit_a(&mut current_state.af, 6), current_state),
        0x78 => instruction_finished(bit_lb(&mut current_state.bc, 7, &mut current_state.af), current_state),
        0x79 => instruction_finished(bit_rb(&mut current_state.bc, 7, &mut current_state.af), current_state),
        0x7A => instruction_finished(bit_lb(&mut current_state.de, 7, &mut current_state.af), current_state),
        0x7B => instruction_finished(bit_rb(&mut current_state.de, 7, &mut current_state.af), current_state),
        0x7C => instruction_finished(bit_lb(&mut current_state.hl, 7, &mut current_state.af), current_state),
        0x7D => instruction_finished(bit_rb(&mut current_state.hl, 7, &mut current_state.af), current_state),
        0x7E => instruction_finished(bit_hl(7, &mut current_state.af, &mut current_state.hl, memory), current_state),
        0x7F => instruction_finished(bit_a(&mut current_state.af, 7), current_state),

        0x80 => instruction_finished(res_lb(&mut current_state.bc, 0), current_state),
        0x81 => instruction_finished(res_rb(&mut current_state.bc, 0), current_state),
        0x82 => instruction_finished(res_lb(&mut current_state.de, 0), current_state),
        0x83 => instruction_finished(res_rb(&mut current_state.de, 0), current_state),
        0x84 => instruction_finished(res_lb(&mut current_state.hl, 0), current_state),
        0x85 => instruction_finished(res_rb(&mut current_state.hl, 0), current_state),
        0x86 => instruction_finished(res_hl(0, &mut current_state.hl, memory), current_state),
        0x87 => instruction_finished(res_lb(&mut current_state.af, 0), current_state),
        0x88 => instruction_finished(res_lb(&mut current_state.bc, 1), current_state),
        0x89 => instruction_finished(res_rb(&mut current_state.bc, 1), current_state),
        0x8A => instruction_finished(res_lb(&mut current_state.de, 1), current_state),
        0x8B => instruction_finished(res_rb(&mut current_state.de, 1), current_state),
        0x8C => instruction_finished(res_lb(&mut current_state.hl, 1), current_state),
        0x8D => instruction_finished(res_rb(&mut current_state.hl, 1), current_state),
        0x8E => instruction_finished(res_hl(1, &mut current_state.hl, memory), current_state),
        0x8F => instruction_finished(res_lb(&mut current_state.af, 1), current_state),

        0x90 => instruction_finished(res_lb(&mut current_state.bc, 2), current_state),
        0x91 => instruction_finished(res_rb(&mut current_state.bc, 2), current_state),
        0x92 => instruction_finished(res_lb(&mut current_state.de, 2), current_state),
        0x93 => instruction_finished(res_rb(&mut current_state.de, 2), current_state),
        0x94 => instruction_finished(res_lb(&mut current_state.hl, 2), current_state),
        0x95 => instruction_finished(res_rb(&mut current_state.hl, 2), current_state),
        0x96 => instruction_finished(res_hl(2, &mut current_state.hl, memory), current_state),
        0x97 => instruction_finished(res_lb(&mut current_state.af, 2), current_state),
        0x98 => instruction_finished(res_lb(&mut current_state.bc, 3), current_state),
        0x99 => instruction_finished(res_rb(&mut current_state.bc, 3), current_state),
        0x9A => instruction_finished(res_lb(&mut current_state.de, 3), current_state),
        0x9B => instruction_finished(res_rb(&mut current_state.de, 3), current_state),
        0x9C => instruction_finished(res_lb(&mut current_state.hl, 3), current_state),
        0x9D => instruction_finished(res_rb(&mut current_state.hl, 3), current_state),
        0x9E => instruction_finished(res_hl(3, &mut current_state.hl, memory), current_state),
        0x9F => instruction_finished(res_lb(&mut current_state.af, 3), current_state),

        0xA0 => instruction_finished(res_lb(&mut current_state.bc, 4), current_state),
        0xA1 => instruction_finished(res_rb(&mut current_state.bc, 4), current_state),
        0xA2 => instruction_finished(res_lb(&mut current_state.de, 4), current_state),
        0xA3 => instruction_finished(res_rb(&mut current_state.de, 4), current_state),
        0xA4 => instruction_finished(res_lb(&mut current_state.hl, 4), current_state),
        0xA5 => instruction_finished(res_rb(&mut current_state.hl, 4), current_state),
        0xA6 => instruction_finished(res_hl(4, &mut current_state.hl, memory), current_state),
        0xA7 => instruction_finished(res_lb(&mut current_state.af, 4), current_state),
        0xA8 => instruction_finished(res_lb(&mut current_state.bc, 5), current_state),
        0xA9 => instruction_finished(res_rb(&mut current_state.bc, 5), current_state),
        0xAA => instruction_finished(res_lb(&mut current_state.de, 5), current_state),
        0xAB => instruction_finished(res_rb(&mut current_state.de, 5), current_state),
        0xAC => instruction_finished(res_lb(&mut current_state.hl, 5), current_state),
        0xAD => instruction_finished(res_rb(&mut current_state.hl, 5), current_state),
        0xAE => instruction_finished(res_hl(5, &mut current_state.hl, memory), current_state),
        0xAF => instruction_finished(res_lb(&mut current_state.af, 5), current_state),

        0xB0 => instruction_finished(res_lb(&mut current_state.bc, 6), current_state),
        0xB1 => instruction_finished(res_rb(&mut current_state.bc, 6), current_state),
        0xB2 => instruction_finished(res_lb(&mut current_state.de, 6), current_state),
        0xB3 => instruction_finished(res_rb(&mut current_state.de, 6), current_state),
        0xB4 => instruction_finished(res_lb(&mut current_state.hl, 6), current_state),
        0xB5 => instruction_finished(res_rb(&mut current_state.hl, 6), current_state),
        0xB6 => instruction_finished(res_hl(6, &mut current_state.hl, memory), current_state),
        0xB7 => instruction_finished(res_lb(&mut current_state.af, 6), current_state),
        0xB8 => instruction_finished(res_lb(&mut current_state.bc, 7), current_state),
        0xB9 => instruction_finished(res_rb(&mut current_state.bc, 7), current_state),
        0xBA => instruction_finished(res_lb(&mut current_state.de, 7), current_state),
        0xBB => instruction_finished(res_rb(&mut current_state.de, 7), current_state),
        0xBC => instruction_finished(res_lb(&mut current_state.hl, 7), current_state),
        0xBD => instruction_finished(res_rb(&mut current_state.hl, 7), current_state),
        0xBE => instruction_finished(res_hl(7, &mut current_state.hl, memory), current_state),
        0xBF => instruction_finished(res_lb(&mut current_state.af, 7), current_state),

        0xC0 => instruction_finished(set_lb(&mut current_state.bc, 0), current_state),
        0xC1 => instruction_finished(set_rb(&mut current_state.bc, 0), current_state),
        0xC2 => instruction_finished(set_lb(&mut current_state.de, 0), current_state),
        0xC3 => instruction_finished(set_rb(&mut current_state.de, 0), current_state),
        0xC4 => instruction_finished(set_lb(&mut current_state.hl, 0), current_state),
        0xC5 => instruction_finished(set_rb(&mut current_state.hl, 0), current_state),
        0xC6 => instruction_finished(set_hl(0, &mut current_state.hl, memory), current_state),
        0xC7 => instruction_finished(set_lb(&mut current_state.af, 0), current_state),
        0xC8 => instruction_finished(set_lb(&mut current_state.bc, 1), current_state),
        0xC9 => instruction_finished(set_rb(&mut current_state.bc, 1), current_state),
        0xCA => instruction_finished(set_lb(&mut current_state.de, 1), current_state),
        0xCB => instruction_finished(set_rb(&mut current_state.de, 1), current_state),
        0xCC => instruction_finished(set_lb(&mut current_state.hl, 1), current_state),
        0xCD => instruction_finished(set_rb(&mut current_state.hl, 1), current_state),
        0xCE => instruction_finished(set_hl(1, &mut current_state.hl, memory), current_state),
        0xCF => instruction_finished(set_lb(&mut current_state.af, 1), current_state),

        0xD0 => instruction_finished(set_lb(&mut current_state.bc, 2), current_state),
        0xD1 => instruction_finished(set_rb(&mut current_state.bc, 2), current_state),
        0xD2 => instruction_finished(set_lb(&mut current_state.de, 2), current_state),
        0xD3 => instruction_finished(set_rb(&mut current_state.de, 2), current_state),
        0xD4 => instruction_finished(set_lb(&mut current_state.hl, 2), current_state),
        0xD5 => instruction_finished(set_rb(&mut current_state.hl, 2), current_state),
        0xD6 => instruction_finished(set_hl(2, &mut current_state.hl, memory), current_state),
        0xD7 => instruction_finished(set_lb(&mut current_state.af, 2), current_state),
        0xD8 => instruction_finished(set_lb(&mut current_state.bc, 3), current_state),
        0xD9 => instruction_finished(set_rb(&mut current_state.bc, 3), current_state),
        0xDA => instruction_finished(set_lb(&mut current_state.de, 3), current_state),
        0xDB => instruction_finished(set_rb(&mut current_state.de, 3), current_state),
        0xDC => instruction_finished(set_lb(&mut current_state.hl, 3), current_state),
        0xDD => instruction_finished(set_rb(&mut current_state.hl, 3), current_state),
        0xDE => instruction_finished(set_hl(3, &mut current_state.hl, memory), current_state),
        0xDF => instruction_finished(set_lb(&mut current_state.af, 3), current_state),

        0xE0 => instruction_finished(set_lb(&mut current_state.bc, 4), current_state),
        0xE1 => instruction_finished(set_rb(&mut current_state.bc, 4), current_state),
        0xE2 => instruction_finished(set_lb(&mut current_state.de, 4), current_state),
        0xE3 => instruction_finished(set_rb(&mut current_state.de, 4), current_state),
        0xE4 => instruction_finished(set_lb(&mut current_state.hl, 4), current_state),
        0xE5 => instruction_finished(set_rb(&mut current_state.hl, 4), current_state),
        0xE6 => instruction_finished(set_hl(4, &mut current_state.hl, memory), current_state),
        0xE7 => instruction_finished(set_lb(&mut current_state.af, 4), current_state),
        0xE8 => instruction_finished(set_lb(&mut current_state.bc, 5), current_state),
        0xE9 => instruction_finished(set_rb(&mut current_state.bc, 5), current_state),
        0xEA => instruction_finished(set_lb(&mut current_state.de, 5), current_state),
        0xEB => instruction_finished(set_rb(&mut current_state.de, 5), current_state),
        0xEC => instruction_finished(set_lb(&mut current_state.hl, 5), current_state),
        0xED => instruction_finished(set_rb(&mut current_state.hl, 5), current_state),
        0xEE => instruction_finished(set_hl(5, &mut current_state.hl, memory), current_state),
        0xEF => instruction_finished(set_lb(&mut current_state.af, 5), current_state),

        0xF0 => instruction_finished(set_lb(&mut current_state.bc, 6), current_state),
        0xF1 => instruction_finished(set_rb(&mut current_state.bc, 6), current_state),
        0xF2 => instruction_finished(set_lb(&mut current_state.de, 6), current_state),
        0xF3 => instruction_finished(set_rb(&mut current_state.de, 6), current_state),
        0xF4 => instruction_finished(set_lb(&mut current_state.hl, 6), current_state),
        0xF5 => instruction_finished(set_rb(&mut current_state.hl, 6), current_state),
        0xF6 => instruction_finished(set_hl(6, &mut current_state.hl, memory), current_state),
        0xF7 => instruction_finished(set_lb(&mut current_state.af, 6), current_state),
        0xF8 => instruction_finished(set_lb(&mut current_state.bc, 7), current_state),
        0xF9 => instruction_finished(set_rb(&mut current_state.bc, 7), current_state),
        0xFA => instruction_finished(set_lb(&mut current_state.de, 7), current_state),
        0xFB => instruction_finished(set_rb(&mut current_state.de, 7), current_state),
        0xFC => instruction_finished(set_lb(&mut current_state.hl, 7), current_state),
        0xFD => instruction_finished(set_rb(&mut current_state.hl, 7), current_state),
        0xFE => instruction_finished(set_hl(7, &mut current_state.hl, memory), current_state),
        0xFF => instruction_finished(set_lb(&mut current_state.af, 7), current_state),
    }

    result
}

fn instruction_finished(values: (u16, u32), state: &mut CpuState) {

    state.pc.add(values.0); state.cycles.add(values.1);
}


// RLC opcodes

fn rlc(af: &mut CpuReg, value: u8) -> u8 {

    let carry = utils::check_bit(value, 7);
    let result = value.rotate_left(1);

    utils::set_zf(result == 0, af);
    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(carry, af);

    result
}

fn rlc_lb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = rlc(af, reg.get_register_lb());
    reg.set_register_lb(result);

    (2, 8)
}

fn rlc_rb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = rlc(af, reg.get_register_rb());
    reg.set_register_rb(result);

    (2, 8)
}

fn rlc_a(af: &mut CpuReg) -> (u16, u32) {

    let reg_value = af.get_register_lb();
    let result = rlc(af, reg_value);
    af.set_register_lb(result);

    (2, 8)
}

fn rlc_hl(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    let result = rlc(af, value);
    cpu::memory_write(&hl.get_register(), result, &memory.0);

    (2, 16)
}


// RRC opcodes

fn rrc(af: &mut CpuReg, value: u8) -> u8 {

    let carry = utils::check_bit(value, 0);
    let result = value.rotate_right(1);

    utils::set_zf(result == 0, af);
    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(carry, af);

    result
}

fn rrc_lb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = rrc(af, reg.get_register_lb());
    reg.set_register_lb(result);

    (2, 8)
}

fn rrc_rb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = rrc(af, reg.get_register_rb());
    reg.set_register_rb(result);

    (2, 8)
}

fn rrc_a(af: &mut CpuReg) -> (u16, u32) {

    let reg_value = af.get_register_lb();
    let result = rrc(af, reg_value);
    af.set_register_lb(result);

    (2, 8)
}

fn rrc_hl(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    let result = rrc(af, value);
    cpu::memory_write(&hl.get_register(), result, &memory.0);
    
    (2, 16)
}


// RL opcodes

fn rl(af: &mut CpuReg, value: u8) -> u8 {

    let will_carry = utils::check_bit(value, 7);
    let old_carry = utils::get_carry(af);
    let mut result = value << 1;
    result = result | old_carry;

    utils::set_cf(will_carry, af);
    utils::set_hf(false, af);
    utils::set_nf(false, af);
    utils::set_zf(result == 0, af);

    result
}

fn rl_lb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {

    let result = rl(af, reg.get_register_lb());
    reg.set_register_lb(result);
    
    (2, 8)
}

fn rl_rb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {

    let result = rl(af, reg.get_register_rb());
    reg.set_register_rb(result);

    (2, 8)
}

fn rl_a(af: &mut CpuReg) -> (u16, u32) {

    let reg_value = af.get_register_lb();
    let result = rl(af, reg_value);
    af.set_register_lb(result);

    (2, 8)
}

fn rl_hl(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    let result = rl(af, value);
    cpu::memory_write(&hl.get_register(), result, &memory.0);

    (2, 16)
}


// RR opcodes

fn rr(af: &mut CpuReg, value: u8) -> u8 {
    
    let will_carry = utils::check_bit(value, 0);
    let old_carry = utils::get_carry(af);
    let mut result = value >> 1;
    result = result | (old_carry << 7);

    utils::set_cf(will_carry, af);
    utils::set_hf(false, af);
    utils::set_nf(false, af);
    utils::set_zf(result == 0, af);
    
    result
}

fn rr_lb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {

    let result = rr(af, reg.get_register_lb());
    reg.set_register_lb(result);
    
    (2, 8)
}

fn rr_rb(reg: &mut CpuReg, af: &mut CpuReg) -> (u16, u32) {

    let result = rr(af, reg.get_register_rb());
    reg.set_register_rb(result);

    (2, 8)
}

fn rr_a(af: &mut CpuReg) -> (u16, u32) {

    let reg_value = af.get_register_lb();
    let result = rr(af, reg_value);
    af.set_register_lb(result);

    (2, 8)
}

fn rr_hl(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    let result = rr(af, value);
    cpu::memory_write(&hl.get_register(), result, &memory.0);

    (2, 16)
}


// SLA opcodes

fn sla(af: &mut CpuReg, value: u8) -> u8 {
    
    let shifted_bit = utils::check_bit(value, 7);
    let result = value << 1;

    utils::set_zf(result == 0, af);
    utils::set_cf(shifted_bit, af);
    utils::set_nf(false, af);
    utils::set_hf(false, af);

    result
}

fn sla_lb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = sla(af, reg.get_register_lb());
    reg.set_register_lb(result);

    (2, 8)
}

fn sla_rb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = sla(af, reg.get_register_rb());
    reg.set_register_rb(result);

    (2, 8)
}

fn sla_a(af: &mut CpuReg) -> (u16, u32) {

    let reg_value = af.get_register_lb();
    let result = sla(af, reg_value);
    af.set_register_lb(result);

    (2, 8)
}

fn sla_val(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    let result = sla(af, value);
    cpu::memory_write(&hl.get_register(), result, &memory.0);

    (2, 16)
}


// SRA opcodes

fn sra(af: &mut CpuReg, value: u8) -> u8 {

    let shifted_bit = utils::check_bit(value, 0);
    let msb = utils::check_bit(value, 7);
    let mut result = value >> 1;
    if msb {result = utils::set_bit_u8(result, 7)}
    else {result = utils::reset_bit_u8(result, 7)}
    
    utils::set_zf(result == 0, af);
    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(shifted_bit, af);

    result
}

fn sra_lb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = sra(af, reg.get_register_lb());
    reg.set_register_lb(result);

    (2, 8)
}

fn sra_rb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = sra(af, reg.get_register_rb());
    reg.set_register_rb(result);

    (2, 8)
}

fn sra_a(af: &mut CpuReg) -> (u16, u32) {

    let reg_value = af.get_register_lb();
    let result = sra(af, reg_value);
    af.set_register_lb(result);

    (2, 8)
}

fn sra_val(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    let result = sra(af, value);
    cpu::memory_write(&hl.get_register(), result, &memory.0);

    (2, 16)
}


// SWAP opcodes

fn swap(af: &mut CpuReg, value: u8) -> u8 {

    let result = utils::swap_nibbles(value);

    utils::set_zf(result == 0, af);
    utils::set_nf(false, af);
    utils::set_hf(false, af);
    utils::set_cf(false, af);
    
    result
}

fn swap_lb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = swap(af, reg.get_register_lb());
    reg.set_register_lb(result);

    (2, 8)
}

fn swap_rb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = swap(af, reg.get_register_rb());
    reg.set_register_rb(result);

    (2, 8)
}

fn swap_a(af: &mut CpuReg) -> (u16, u32) {

    let reg_value = af.get_register_lb();
    let result = swap(af, reg_value);
    af.set_register_lb(result);

    (2, 8)
}

fn swap_hl(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    let result = swap(af, value);
    cpu::memory_write(&hl.get_register(), result, &memory.0);

    (2, 16)
}


// SRL opcodes

fn srl(af: &mut CpuReg, value: u8) -> u8 {

    let shifted_bit = utils::check_bit(value, 0);
    let result = value >> 1;

    utils::set_zf(result == 0, af);
    utils::set_cf(shifted_bit, af);
    utils::set_nf(false, af);
    utils::set_hf(false, af);

    result
}

fn srl_lb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = srl(af, reg.get_register_lb());
    reg.set_register_lb(result);

    (2, 8)
}

fn srl_rb(af: &mut CpuReg, reg: &mut CpuReg) -> (u16, u32) {

    let result = srl(af, reg.get_register_rb());
    reg.set_register_rb(result);

    (2, 8)
}

fn srl_a(af: &mut CpuReg) -> (u16, u32) {

    let reg_value = af.get_register_lb();
    let result = srl(af, reg_value);
    af.set_register_lb(result);
    
    (2, 8)
}

fn srl_val(af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    let result = srl(af, value);
    cpu::memory_write(&hl.get_register(), result, &memory.0);

    (2, 16)
}


// BIT opcodes

fn bit(af: &mut CpuReg, value: u8, bit: u8) {
    let result = utils::check_bit(value, bit);

    utils::set_zf(!result, af);
    utils::set_nf(false, af);
    utils::set_hf(true, af);
}

fn bit_a(af: &mut CpuReg, checked_bit: u8) -> (u16, u32) {

    let reg_value = af.get_register_lb();
    bit(af, reg_value, checked_bit);
    (2, 8)
}

fn bit_lb(reg: &mut CpuReg, checked_bit: u8, af: &mut CpuReg) -> (u16, u32) {

    bit(af, reg.get_register_lb(), checked_bit);
    (2, 8)
}

fn bit_rb(reg: &mut CpuReg, checked_bit: u8, af: &mut CpuReg) -> (u16, u32) {

    bit(af, reg.get_register_rb(), checked_bit);
    (2, 8)
}

fn bit_hl(checked_bit: u8, af: &mut CpuReg, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    bit(af, value, checked_bit);
    (2, 16)
}


// RES opcodes

fn res(value: u8, bit: u8) -> u8 {
    utils::reset_bit_u8(value, bit)
}

fn res_lb(reg: &mut CpuReg, bit: u8) -> (u16, u32) {

    let result = res(reg.get_register_lb(), bit);
    reg.set_register_lb(result);
    (2, 8)
}

fn res_rb(reg: &mut CpuReg, bit: u8) -> (u16, u32) {

    let result = res(reg.get_register_rb(), bit);
    reg.set_register_rb(result);
    (2, 8)
}

fn res_hl(bit: u8, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    let result = res(value, bit);
    cpu::memory_write(&hl.get_register(), result, &memory.0);
    (2, 16)
}


// SET opcodes

fn set(value: u8, bit: u8) -> u8 {
    utils::set_bit_u8(value, bit)
}

fn set_lb(reg: &mut CpuReg, bit: u8) -> (u16, u32) {
    
    let result = set(reg.get_register_lb(), bit);
    reg.set_register_lb(result);
    (2, 8)
}

fn set_rb(reg: &mut CpuReg, bit: u8) -> (u16, u32) {
    
    let result = set(reg.get_register_rb(), bit);
    reg.set_register_rb(result);
    (2, 8)
}

fn set_hl(bit: u8, hl: &mut CpuReg, memory: &(mpsc::Sender<MemoryAccess>, mpsc::Receiver<u8>)) -> (u16, u32) {

    let value = cpu::memory_read_u8(&hl.get_register(), memory);
    let result = set(value, bit);
    cpu::memory_write(&hl.get_register(), result, &memory.0);
    (2, 16)
}