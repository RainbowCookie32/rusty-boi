use std::convert::TryInto;

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

pub fn init_cpu(rom: Vec<u8>) {

    let initial_state = CpuState {
        af: 0x1180,
        bc: 0x0000,
        de: 0xFF56,
        hl: 0x000D,
        pc: 0x0100,
        sp: 0xFFFE,

        stack: Vec::new(),
        ram: Vec::new(),
        loaded_rom: rom,

        should_execute: true,
    };

    println!("CPU initialized");
    exec_loop(initial_state);

}

fn exec_loop(state: CpuState) {

    let mut current_state = state;
    
    while current_state.should_execute {
        current_state = run_instruction(memory_read(&current_state.pc, &current_state), current_state);
    }

    println!("should_execute is false, stopping CPU execution");
    
}

fn memory_read(addr: &u16, state: &CpuState) -> u8 {

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
        state.loaded_rom[memory_addr]
    }
    else
    {
        panic!("Read unvalid or unimplemented addr");
    }
}

fn memory_write(addr: u16, value: u8, state: &CpuState) {

}

fn run_instruction(opcode: u8, state: CpuState) -> CpuState {

    // Setting up the default result state using the values the CPU had when starting this opcode
    let mut result_state = CpuState { 
        af: state.af, 
        bc: state.bc, 
        de: state.de, 
        hl: state.hl,
        pc: state.pc, 
        sp: state.sp,
        stack: state.stack,
        ram: state.ram,
        loaded_rom: state.loaded_rom,
        should_execute: state.should_execute,
    };

    println!("Running opcode {}", opcode);

    match opcode {

        0x00 => result_state = nop(result_state),

        _    => 
        {
            result_state.should_execute = false;
            println!("Unrecognized opcode: {}", opcode);
        },
    }

    result_state
}

fn nop(state: CpuState) -> CpuState {

    let mut result_state = state;
    result_state.pc += 1;
    result_state
}