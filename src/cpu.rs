use std::convert::TryInto;

use byteorder::{ByteOrder, LittleEndian};



pub struct CpuState {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub pc: u16,
    pub sp: u16,

    pub stack: Vec<u8>,
    pub ram: Vec<u8>,

    pub loaded_rom: Vec<u8>,

    pub should_execute: bool,
    pub nops: u8,
}

enum JumpCondition {

    ZSet,
    ZNotSet,
    CSet,
    CNotSet,
}

enum TargetFlag {

    ZFlag,
    NFlag,
    HFlag,
    CFlag,
}

enum TargetReg {
    
    AF,
    BC,
    DE,
    HL,
    SP,

    A,
    B,
    C,
    D,
    E,
    H,
    L,
}



fn set_flag(flag: TargetFlag, state: CpuState) -> CpuState {

    let mut result_state = state;

    match flag {
        TargetFlag::ZFlag => result_state.af = set_bit(result_state.af, 7),
        TargetFlag::NFlag => result_state.af = set_bit(result_state.af, 6),
        TargetFlag::HFlag => result_state.af = set_bit(result_state.af, 5),
        TargetFlag::CFlag => result_state.af = set_bit(result_state.af, 4),
    }

    println!("Flag should be set");
    result_state
}

fn reset_flag(flag: TargetFlag, state: CpuState) -> CpuState {

    let mut result_state = state;

    match flag {
        TargetFlag::ZFlag => result_state.af = reset_bit(result_state.af, 7),
        TargetFlag::NFlag => result_state.af = reset_bit(result_state.af, 6),
        TargetFlag::HFlag => result_state.af = reset_bit(result_state.af, 5),
        TargetFlag::CFlag => result_state.af = reset_bit(result_state.af, 4),
    }

    println!("Flag should be reset");
    result_state
}

// assuming 16 bit values is all we ever deal with
// lb means "left byte", rb means "right byte"

// (left and right is used instead of high and low in order to
// prevent confusion when dealing with different endiannesses)

fn get_lb(value: u16) -> u8 {
    (value >> 8) as u8
}

fn set_lb(value: u16, lb_val: u8) -> u16 {
    value & 0xFF | (lb_val as u16) << 8
}

fn get_rb(value: u16) -> u8 {
    (value & 0xFF) as u8
}

fn set_rb(value: u16, rb_val: u8) -> u16 {
    value & !0xFF | rb_val as u16
}

fn set_bit(value: u16, offset: u8) -> u16 {
    value | 1 << offset
}

fn reset_bit(value: u16, offset: u8) -> u16 {
    value & !(1 << offset)
}

pub fn init_cpu(rom: Vec<u8>) {

    let initial_state = CpuState {
        af: 0x1180,
        bc: 0x0000,
        de: 0xFF56,
        hl: 0x000D,
        pc: 0x0100,
        sp: 0xFFFE,

        stack: Vec::new(),
        ram: vec![0; 8192],
        loaded_rom: rom,

        should_execute: true,
        nops: 0,
    };

    println!("CPU initialized");
    exec_loop(initial_state);

}

fn exec_loop(state: CpuState) {

    let mut current_state = state;
    
    while current_state.should_execute {
        current_state = run_instruction(memory_read_u8(&current_state.pc, &current_state), current_state);
    }

    println!("should_execute is false, stopping CPU execution");
    
}

fn memory_read_u8(addr: &u16, state: &CpuState) -> u8 {

    let address: u16 = *addr;
    if address > 0x0000 && address <= 0x3FFF
    {
        let memory_addr: usize = address.try_into().unwrap();
        state.loaded_rom[memory_addr]
    }
    else if address >= 0x4000 && address <= 0x7FFF
    {
        let memory_addr: usize = address.try_into().unwrap();
        state.loaded_rom[memory_addr]
    }
    else if address >= 0xC000 && address <= 0xCFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();
        state.ram[memory_addr]
    }
    else if address >= 0xD000 && address <= 0xDFFF
    {
        let memory_addr: usize = (address - 0xD000).try_into().unwrap();
        state.ram[memory_addr]
    }
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", addr));
    }
}

fn memory_read_u16(addr: &u16, state: &CpuState) -> u16 {

    let address: u16 = *addr;
    let mut target: Vec<u8> = vec![0; 2];
    let target_addr: u16;

    if address > 0x0000 && address <= 0x3FFF
    {
        let memory_addr: usize = address.try_into().unwrap();
        target[0] = state.loaded_rom[memory_addr];
        target[1] = state.loaded_rom[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0x4000 && address <= 0x7FFF
    {
        let memory_addr: usize = address.try_into().unwrap();
        target[0] = state.loaded_rom[memory_addr];
        target[1] = state.loaded_rom[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0xC000 && address <= 0xCFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();
        target[0] = state.ram[memory_addr];
        target[1] = state.ram[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else if address >= 0xD000 && address <= 0xDFFF
    {
        let memory_addr: usize = (address - 0xD000).try_into().unwrap();
        target[0] = state.ram[memory_addr];
        target[1] = state.ram[memory_addr + 1];
        target_addr = LittleEndian::read_u16(&target);
        target_addr
    }
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", addr));
    }
}

fn memory_write(address: u16, value: u8, state: CpuState) -> CpuState {

    let mut result_state = state;

    if address > 0x0000 && address <= 0x3FFF
    {
        panic!("Tried to write to cart, illegal write");
    }
    else if address >= 0x4000 && address <= 0x7FFF
    {
        panic!("Tried to write to cart, illegal write");
    }
    else if address >= 0xC000 && address <= 0xCFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();        
        result_state.ram[memory_addr] = value;
        result_state
    }
    else if address >= 0xD000 && address <= 0xDFFF
    {
        let memory_addr: usize = (address - 0xD000).try_into().unwrap();
        result_state.ram[memory_addr] = value;
        result_state
    }
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", address));
    }
}

fn run_instruction(opcode: u8, state: CpuState) -> CpuState {

    // Setting up the default result state using the values the CPU had when starting this opcode
    // TODO: Maybe copying the state around isn't the best approach, fix soon™
    let mut result_state = state;

    println!("Running opcode {} at PC: {}", format!("{:#X}", opcode), format!("{:#X}", result_state.pc));

    if opcode == 0x00 { 
        if result_state.nops == 5
        {
            result_state.should_execute = false;
            println!("We got flooded by NOPs, something's wrong");
        }
        else
        {
            result_state.nops += 1;
        }
    }

    match opcode {

        0x00 => result_state = nop(result_state),
        0x01 => result_state = ld_full_from_imm(result_state, TargetReg::BC),
        0x02 => result_state = save_reg_to_full(result_state, TargetReg::A, TargetReg::BC),
        0x03 => result_state = increment_full_reg(result_state, TargetReg::BC),
        0x04 => result_state = increment_reg(result_state, TargetReg::B),
        0x05 => result_state = decrement_reg(result_state, TargetReg::B),
        0x06 => result_state = ld_hi_from_imm(result_state, TargetReg::B),
        0x0B => result_state = decrement_full_reg(result_state, TargetReg::BC),
        0x0C => result_state = increment_reg(result_state, TargetReg::C),
        0x0D => result_state = decrement_reg(result_state, TargetReg::C),
        0x0E => result_state = ld_low_from_imm(result_state, TargetReg::C),

        0x11 => result_state = ld_full_from_imm(result_state, TargetReg::DE),
        0x12 => result_state = save_reg_to_full(result_state, TargetReg::A, TargetReg::DE),
        0x13 => result_state = increment_full_reg(result_state, TargetReg::DE),
        0x14 => result_state = increment_reg(result_state, TargetReg::D),
        0x15 => result_state = decrement_reg(result_state, TargetReg::D),
        0x16 => result_state = ld_hi_from_imm(result_state, TargetReg::D),
        0x18 => result_state = relative_jmp(result_state),
        0x1B => result_state = decrement_full_reg(result_state, TargetReg::DE),
        0x1C => result_state = increment_reg(result_state, TargetReg::E),
        0x1D => result_state = decrement_reg(result_state, TargetReg::E),
        0x1E => result_state = ld_low_from_imm(result_state, TargetReg::E),

        0x20 => result_state = conditional_relative_jump(result_state, JumpCondition::ZNotSet),
        0x21 => result_state = ld_full_from_imm(result_state, TargetReg::HL),
        0x23 => result_state = increment_full_reg(result_state, TargetReg::HL),
        0x24 => result_state = increment_reg(result_state, TargetReg::H),
        0x25 => result_state = decrement_reg(result_state, TargetReg::H),
        0x26 => result_state = ld_hi_from_imm(result_state, TargetReg::H),
        0x28 => result_state = conditional_relative_jump(result_state, JumpCondition::ZSet),
        0x2A => result_state = ld_a_from_hl_inc(result_state),
        0x2B => result_state = decrement_full_reg(result_state, TargetReg::HL),
        0x2C => result_state = increment_reg(result_state, TargetReg::L),
        0x2D => result_state = decrement_reg(result_state, TargetReg::L),
        0x2E => result_state = ld_low_from_imm(result_state, TargetReg::L),

        0x30 => result_state = conditional_relative_jump(result_state, JumpCondition::CNotSet),
        0x31 => result_state = ld_full_from_imm(result_state, TargetReg::SP),
        0x38 => result_state = conditional_relative_jump(result_state, JumpCondition::CSet),
        0x3C => result_state = increment_reg(result_state, TargetReg::A),
        0x3D => result_state = decrement_reg(result_state, TargetReg::A),
        0x3E => result_state = ld_low_from_imm(result_state, TargetReg::A),

        0x47 => result_state = ld_hi_into_hi(result_state, TargetReg::A, TargetReg::B),

        0x70 => result_state = save_reg_to_full(result_state, TargetReg::B, TargetReg::HL),
        0x71 => result_state = save_reg_to_full(result_state, TargetReg::C, TargetReg::HL),
        0x72 => result_state = save_reg_to_full(result_state, TargetReg::D, TargetReg::HL),
        0x73 => result_state = save_reg_to_full(result_state, TargetReg::E, TargetReg::HL),
        0x74 => result_state = save_reg_to_full(result_state, TargetReg::H, TargetReg::HL),
        0x75 => result_state = save_reg_to_full(result_state, TargetReg::L, TargetReg::HL),
        0x78 => result_state = ld_hi_into_hi(result_state, TargetReg::B, TargetReg::A),

        0xC3 => result_state = jmp(result_state),


        _    => 
        {
            result_state.should_execute = false;
            println!("Unrecognized opcode: {} at PC {}", format!("{:#X}", opcode), format!("{:#X}", result_state.pc));
        },
    }

    result_state
}

fn nop(state: CpuState) -> CpuState {

    let mut result_state = state;
    result_state.pc += 1;
    result_state
}

fn jmp(state: CpuState) -> CpuState {

    let mut result_state = state;
    result_state.pc = memory_read_u16(&(result_state.pc + 1), &result_state);
    result_state
}

fn relative_jmp(state: CpuState) -> CpuState {

    let mut result_state: CpuState = state;
    let signed_byte: i8 = memory_read_u8(&(result_state.pc + 1), &result_state) as i8;
    let target_addr: u16 = result_state.pc.wrapping_add(signed_byte as u16);
    
    result_state.pc = target_addr + 2;

    result_state
}

fn conditional_relative_jump(state: CpuState, condition: JumpCondition) -> CpuState {

    let mut result_state = state;
    let jump: bool;

    match condition {

        JumpCondition::CNotSet => jump = (get_rb(result_state.af) & (1 << 4)) == 0,
        JumpCondition::ZNotSet => {
            let value = get_rb(result_state.af);
            jump = (value & (1 << 7)) == 0;
        }
        JumpCondition::CSet => jump = (get_rb(result_state.af) & (1 << 4)) == 1,
        JumpCondition::ZSet => jump = (get_rb(result_state.af) & (1 << 7))  == 1,
    }

    if jump {
        result_state = relative_jmp(result_state);
    }
    else {
        result_state.pc += 2;
    }
    
    result_state
    
}

fn ld_full_from_imm(state: CpuState, target_reg: TargetReg) -> CpuState {
    
    let mut result_state = state;
    let new_value = memory_read_u16(&(result_state.pc + 1), &result_state);

    match target_reg {

        // Only the full registers are valid for this instructions.
        TargetReg::AF => result_state.af = new_value,
        TargetReg::BC => result_state.bc = new_value,
        TargetReg::DE => result_state.de = new_value,
        TargetReg::HL => result_state.hl = new_value,
        TargetReg::SP => result_state.sp = new_value,
        
        // Anything else, and something's wrong.
        _ => panic!("Invalid register selected"),
    }
    
    result_state.pc += 3;
    result_state
}

fn ld_hi_from_imm(state: CpuState, target_reg: TargetReg) -> CpuState {

    let mut result_state = state;
    let new_value = memory_read_u8(&(result_state.pc + 1), &result_state);

    match target_reg {

        // Only the high byte of a Register can be the target of this instruction.
        TargetReg::A => result_state.af = set_lb(result_state.af, new_value),
        TargetReg::B => result_state.bc = set_lb(result_state.bc, new_value),
        TargetReg::D => result_state.de = set_lb(result_state.de, new_value),
        TargetReg::H => result_state.hl = set_lb(result_state.hl, new_value),

        _ => panic!("Invalid register selected for this instruction"),
    }

    result_state.pc += 2;
    result_state
}

fn ld_low_from_imm(state: CpuState, target_reg: TargetReg) -> CpuState {

    let mut result_state = state;
    let new_value = memory_read_u8(&(result_state.pc + 1), &result_state);

    match target_reg {

        TargetReg::C => result_state.bc = set_rb(result_state.bc, new_value),
        TargetReg::E => result_state.de = set_rb(result_state.de, new_value),
        TargetReg::L => result_state.hl = set_rb(result_state.hl, new_value),

        _ => panic!("Invalid register selected for this instruction"),
    }

    result_state.pc += 2;
    result_state
}

fn ld_hi_into_hi(state: CpuState, source_reg: TargetReg, target_reg: TargetReg) -> CpuState {

    let mut result_state = state;
    let source: u8;
    let target: u16;

    match source_reg {
        TargetReg::A => source = get_lb(result_state.af),
        TargetReg::B => source = get_lb(result_state.bc),
        TargetReg::D => source = get_lb(result_state.de),
        TargetReg::H => source = get_lb(result_state.hl),
        
        _ => panic!("Invalid register in instruction"),
    }

    match target_reg {
        TargetReg::A => {
            target = result_state.af;
            result_state.af = set_lb(target, source)
        },
        TargetReg::B => {
            target = result_state.bc;
            result_state.bc = set_lb(target, source)
        },
        TargetReg::D => {
            target = result_state.af;
            result_state.de = set_lb(target, source)
        },
        TargetReg::H => {
            target = result_state.af;
            result_state.hl = set_lb(target, source)
        },
        
        _ => panic!("Invalid register in instruction"),
    }
    result_state.pc += 1;
    result_state
}

fn ld_a_from_hl_inc(state: CpuState) -> CpuState {

    let mut result_state = state;
    let new_value = memory_read_u8(&result_state.hl, &result_state);
    result_state.af = set_lb(result_state.af, new_value);
    result_state.hl += 1;
    result_state.pc += 1;
    result_state
}

fn save_reg_to_full(state: CpuState, target_reg: TargetReg, addr_reg: TargetReg) -> CpuState {

    let mut result_state = state;
    let value: u8;
    let addr: u16;

    match addr_reg {

        TargetReg::BC => addr = result_state.bc,
        TargetReg::DE => addr = result_state.de,
        TargetReg::HL => addr = result_state.hl,

        _ => panic!("Unvalid reg for instruction"),
    }

    match target_reg {

        TargetReg::A => value = get_lb(result_state.af),
        TargetReg::B => value = get_lb(result_state.bc),
        TargetReg::C => value = get_rb(result_state.bc),
        TargetReg::D => value = get_lb(result_state.de),
        TargetReg::E => value = get_rb(result_state.de),
        TargetReg::H => value = get_lb(result_state.hl),
        TargetReg::L => value = get_rb(result_state.hl),

        _ => panic!("Unvalid reg for instruction"),
    }

    println!("About to write {} to {}", format!("{:#X}", value), format!("{:#X}", addr));

    result_state = memory_write(addr, value, result_state);
    result_state.pc += 1;

    result_state
}

fn increment_reg(state: CpuState, reg: TargetReg) -> CpuState {

    let mut result_state = state;

    match reg {

        TargetReg::A => {
            let result = get_lb(result_state.af).overflowing_add(1);

            result_state.af = set_lb(result_state.af, result.0);
            if result.1 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = reset_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::B => {
            let result = get_lb(result_state.bc).overflowing_add(1);

            result_state.bc = set_lb(result_state.bc, result.0);
            if result.1 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = reset_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::C => {
            let result = get_rb(result_state.bc).overflowing_add(1);

            result_state.bc = set_rb(result_state.bc, result.0);
            if result.1 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = reset_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::D => {
            let result = get_lb(result_state.de).overflowing_add(1);

            result_state.de = set_lb(result_state.de, result.0);
            if result.1 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = reset_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::E => {
            let result = get_rb(result_state.de).overflowing_add(1);

            result_state.de = set_rb(result_state.de, result.0);
            if result.1 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = reset_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::H => {
            let result = get_lb(result_state.hl).overflowing_add(1);

            result_state.hl = set_lb(result_state.hl, result.0);
            if result.1 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = reset_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::L => {
            let result = get_rb(result_state.hl).overflowing_add(1);

            result_state.hl = set_rb(result_state.hl, result.0);
            if result.1 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = reset_flag(TargetFlag::NFlag, result_state);
        },

        _ => panic!("Invalid reg for instruction"),

    }

    result_state.pc += 1;
    result_state
}

fn decrement_reg(state: CpuState, reg: TargetReg) -> CpuState {

    let mut result_state = state;

    match reg {

        TargetReg::A => {
            let result = get_lb(result_state.af).overflowing_sub(1);

            result_state.af = set_lb(result_state.af, result.0);
            if result.0 == 0 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = set_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::B => {
            let result = get_lb(result_state.bc).overflowing_sub(1);

            result_state.bc = set_lb(result_state.bc, result.0);
            if result.0 == 0 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = set_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::C => {
            let result = get_rb(result_state.bc).overflowing_sub(1);

            result_state.bc = set_rb(result_state.bc, result.0);
            if result.0 == 0 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = set_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::D => {
            let result = get_lb(result_state.de).overflowing_sub(1);

            result_state.de = set_lb(result_state.de, result.0);
            if result.0 == 0 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = set_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::E => {
            let result = get_rb(result_state.de).overflowing_sub(1);

            result_state.de = set_rb(result_state.de, result.0);
            if result.0 == 0 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = set_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::H => {
            let result = get_lb(result_state.hl).overflowing_sub(1);

            result_state.hl = set_lb(result_state.hl, result.0);
            if result.0 == 0 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = set_flag(TargetFlag::NFlag, result_state);
        },

        TargetReg::L => {
            let result = get_rb(result_state.hl).overflowing_sub(1);

            result_state.hl = set_rb(result_state.hl, result.0);
            if result.0 == 0 { result_state = set_flag(TargetFlag::ZFlag, result_state) }
            else { result_state = reset_flag(TargetFlag::ZFlag, result_state) }

            result_state = set_flag(TargetFlag::NFlag, result_state);
        },

        _ => panic!("Invalid reg for instruction"),

    }

    result_state.pc += 1;
    result_state
}

fn increment_full_reg(state: CpuState, reg: TargetReg) -> CpuState {

    let mut result_state = state;

    match reg {

        TargetReg::BC => result_state.bc = result_state.bc.overflowing_add(1).0,
        TargetReg::DE => result_state.de = result_state.de.overflowing_add(1).0,
        TargetReg::HL => result_state.hl = result_state.hl.overflowing_add(1).0,
        _ => panic!("Invalid reg for instruction"),
    }

    result_state.pc += 1;
    result_state
}

fn decrement_full_reg(state: CpuState, reg: TargetReg) -> CpuState {

    let mut result_state = state;

    match reg {

        TargetReg::BC => result_state.bc = result_state.bc.overflowing_sub(1).0,
        TargetReg::DE => result_state.de = result_state.de.overflowing_sub(1).0,
        TargetReg::HL => result_state.hl = result_state.hl.overflowing_sub(1).0,
        _ => panic!("Invalid reg for instruction"),
    }

    result_state.pc += 1;
    result_state
}
