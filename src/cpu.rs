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

fn get_higher_byte(value: u16) -> u16 {

    let nibble = (value & 0xFF00) >> 8;
    nibble
}

fn get_lower_byte(value: u16) -> u16 {

    let nibble = (value & 0xFF) << 8;
    nibble
}

fn set_lower_byte(target_value: u16, new_value: u16) -> u16 {

    let mut result = target_value;
    let value = new_value << 8;
    result &= 0xFF00;
    result |= value & 0xFF;
    result
}

fn set_higher_byte(target_value: u16, new_value: u16) -> u16 {

    let mut result = target_value;
    let value = new_value << 8;
    result &= 0xFF;
    result |= value & 0xFF00;
    result
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
    if address > 0x0000 || address <= 0x3FFF
    {
        let memory_addr: usize = address.try_into().unwrap();
        state.loaded_rom[memory_addr]
    }
    else if address >= 0x4000 || address <= 0x7FFF
    {
        let memory_addr: usize = (address - 0x4000).try_into().unwrap();
        state.loaded_rom[memory_addr]
    }
    else if address >= 0xC000 || address <= 0xCFFF
    {
        let memory_addr: usize = (address - 0xC000).try_into().unwrap();
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
        let memory_addr: usize = (address - 0x4000).try_into().unwrap();
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
    else
    {
        panic!("Invalid or unimplemented read at {}", format!("{:#X}", address));
    }
}

fn run_instruction(opcode: u8, state: CpuState) -> CpuState {

    // Setting up the default result state using the values the CPU had when starting this opcode
    // TODO: Maybe copying the state around isn't the best approach, fix soon™
    let mut result_state = state;

    println!("Running opcode {}", format!("{:#X}", opcode));

    match opcode {

        0x00 => result_state = nop(result_state),
        0x01 => result_state = ld_full_from_imm(result_state, TargetReg::BC),
        0x02 => result_state = save_reg_to_full(result_state, TargetReg::A, TargetReg::BC),
        0x06 => result_state = ld_hi_from_imm(result_state, TargetReg::B),
        0x0E => result_state = ld_low_from_imm(result_state, TargetReg::C),

        0x11 => result_state = ld_full_from_imm(result_state, TargetReg::DE),
        0x12 => result_state = save_reg_to_full(result_state, TargetReg::A, TargetReg::DE),
        0x16 => result_state = ld_hi_from_imm(result_state, TargetReg::D),
        0x1E => result_state = ld_low_from_imm(result_state, TargetReg::E),

        0x21 => result_state = ld_full_from_imm(result_state, TargetReg::HL),
        0x26 => result_state = ld_hi_from_imm(result_state, TargetReg::H),
        0x2A => result_state = ld_a_from_hl_inc(result_state),
        0x2E => result_state = ld_low_from_imm(result_state, TargetReg::L),

        0x31 => result_state = ld_full_from_imm(result_state, TargetReg::SP),
        0x3E => result_state = ld_low_from_imm(result_state, TargetReg::A),

        0x47 => result_state = ld_hi_into_hi(result_state, TargetReg::A, TargetReg::B),

        0x70 => result_state = save_reg_to_full(result_state, TargetReg::B, TargetReg::HL),
        0x71 => result_state = save_reg_to_full(result_state, TargetReg::C, TargetReg::HL),
        0x72 => result_state = save_reg_to_full(result_state, TargetReg::D, TargetReg::HL),
        0x73 => result_state = save_reg_to_full(result_state, TargetReg::E, TargetReg::HL),
        0x74 => result_state = save_reg_to_full(result_state, TargetReg::H, TargetReg::HL),
        0x75 => result_state = save_reg_to_full(result_state, TargetReg::L, TargetReg::HL),

        0xC3 => result_state = jmp(result_state),


        _    => 
        {
            result_state.should_execute = false;
            println!("Unrecognized opcode: {}", format!("{:#X}", opcode));
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
    let new_value: u16 = memory_read_u8(&(result_state.pc + 1), &result_state).into();

    match target_reg {

        // Only the high byte of a Register can be the target of this instruction.
        TargetReg::A => result_state.af = set_higher_byte(result_state.af, new_value),
        TargetReg::B => result_state.bc = set_higher_byte(result_state.bc, new_value),
        TargetReg::D => result_state.de = set_higher_byte(result_state.de, new_value),
        TargetReg::H => result_state.hl = set_higher_byte(result_state.hl, new_value),

        _ => panic!("Invalid register selected for this instruction"),
    }

    result_state.pc += 2;
    result_state
}

fn ld_low_from_imm(state: CpuState, target_reg: TargetReg) -> CpuState {

    let mut result_state = state;
    let new_value: u16 = memory_read_u8(&(result_state.pc + 1), &result_state).into();

    match target_reg {

        TargetReg::A => result_state.af = set_higher_byte(result_state.af, new_value),
        TargetReg::C => result_state.bc = set_lower_byte(result_state.bc, new_value),
        TargetReg::E => result_state.de = set_lower_byte(result_state.de, new_value),
        TargetReg::L => result_state.hl = set_lower_byte(result_state.hl, new_value),

        _ => panic!("Invalid register selected for this instruction"),
    }

    result_state.pc += 2;
    result_state
}

fn ld_hi_into_hi(state: CpuState, source_reg: TargetReg, target_reg: TargetReg) -> CpuState {

    let mut result_state = state;
    let source: u16;
    let target: u16;

    match source_reg {
        TargetReg::A => source = get_higher_byte(result_state.af),
        TargetReg::B => source = get_higher_byte(result_state.bc),
        TargetReg::D => source = get_higher_byte(result_state.de),
        TargetReg::H => source = get_higher_byte(result_state.hl),
        
        _ => panic!("Invalid register in instruction"),
    }

    match target_reg {
        TargetReg::A => {
            target = result_state.af;
            result_state.af = set_higher_byte(target, source)
        },
        TargetReg::B => {
            target = result_state.bc;
            result_state.bc = set_higher_byte(target, source)
        },
        TargetReg::D => {
            target = result_state.af;
            result_state.de = set_higher_byte(target, source)
        },
        TargetReg::H => {
            target = result_state.af;
            result_state.hl = set_higher_byte(target, source)
        },
        
        _ => panic!("Invalid register in instruction"),
    }
    result_state.pc += 1;
    result_state
}

fn ld_a_from_hl_inc(state: CpuState) -> CpuState {

    let mut result_state = state;
    let new_value:u16 = memory_read_u8(&result_state.hl, &result_state).try_into().unwrap();
    result_state.af = set_higher_byte(result_state.af, new_value);
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

        TargetReg::A => value = get_higher_byte(result_state.af).try_into().unwrap(),
        TargetReg::B => value = get_higher_byte(result_state.bc).try_into().unwrap(),
        TargetReg::C => value = get_lower_byte(result_state.bc).try_into().unwrap(),
        TargetReg::D => value = get_higher_byte(result_state.de).try_into().unwrap(),
        TargetReg::E => value = get_lower_byte(result_state.de).try_into().unwrap(),
        TargetReg::H => value = get_higher_byte(result_state.hl).try_into().unwrap(),
        TargetReg::L => value = get_lower_byte(result_state.hl).try_into().unwrap(),

        _ => panic!("Unvalid reg for instruction"),
    }

    result_state = memory_write(addr, value, result_state);
    result_state.pc += 1;

    result_state
}