use std::sync::mpsc::{Sender, Receiver};

use super::utils;

use super::memory::MemoryOp;
use super::memory::MemoryAccess;


pub struct TimerState {

    div_cycles: u32,
    last_div_cycle: u32,

    timer_cycles: u32,
    last_timer_cycle: u32,
    needed_cycles: u16,
}

pub fn init_timer() -> TimerState {

    TimerState {
        div_cycles: 0,
        last_div_cycle: 0,
        timer_cycles: 0,
        last_timer_cycle: 0,
        needed_cycles: 0,
    }
}

pub fn timer_cycle(timer_state: &mut TimerState, cycles: u32, memory: &(Sender<MemoryAccess>, Receiver<u8>)) {

    let tac_value = memory_read(0xFF07, &memory);
    let timer_enabled = utils::check_bit(tac_value, 2);
    let mut current_state = timer_state;

    if timer_enabled {
            
        current_state.needed_cycles = get_frequency(tac_value);
        current_state.timer_cycles += cycles;
        current_state.div_cycles += cycles;

        if current_state.div_cycles - current_state.last_div_cycle >= 256 {

            let div_value = memory_read(0xFF04, &memory);
            let new_value = div_value.overflowing_add(1);
            memory_write(new_value.0, 0xFF04, &memory);
            current_state.last_div_cycle = current_state.div_cycles;
            if new_value.1 {
                current_state.div_cycles = 0;
                current_state.last_div_cycle = 0;
            }
        }

        if current_state.timer_cycles - current_state.last_timer_cycle >= current_state.needed_cycles as u32 {

            let tima_value = memory_read(0xFF05, &memory);
            let new_value = tima_value.overflowing_add(1);

            if new_value.1 {
                    
                let if_value = memory_read(0xFF0F, &memory);
                let modulo_value = memory_read(0xFF06, &memory);
                memory_write(modulo_value, 0xFF05, &memory);
                memory_write(utils::set_bit_u8(if_value, 2), 0xFF0F, &memory);
                current_state.timer_cycles = 0;
            }
            else {
                memory_write(new_value.0, 0xFF05, &memory);
                current_state.timer_cycles = 0;
            }

            current_state.last_timer_cycle = current_state.timer_cycles;
        }
    }
}

fn get_frequency(tac: u8) -> u16 {

    let bit0 = utils::check_bit(tac, 0);
    let bit1 = utils::check_bit(tac, 1);

    if !bit0 && !bit1 {
        1024
    }
    else if !bit0 && bit1{
        16
    }
    else if bit0 && !bit1 {
        64
    }
    else if bit0 && bit1 {
        256
    }
    else {
        0
    }
}

fn memory_read(addr: u16, memory: &(Sender<MemoryAccess>, Receiver<u8>)) -> u8 {
    
    let mem_request = MemoryAccess {
        operation: MemoryOp::Read,
        address: addr,
        value: 0,
    };
            
    memory.0.send(mem_request).unwrap();
    memory.1.recv().unwrap()
}

fn memory_write(value: u8, addr: u16, memory: &(Sender<MemoryAccess>, Receiver<u8>)) {

    let mem_request = MemoryAccess {
        operation: MemoryOp::Write,
        address: addr,
        value: value,
    };
            
    memory.0.send(mem_request).unwrap();
}