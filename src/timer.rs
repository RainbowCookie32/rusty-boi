use std::sync::{Arc, Mutex};

use super::utils;

use super::memory;
use super::memory::IoRegisters;


pub struct TimerState {

    div_cycles: u16,
    timer_cycles: u16,
    needed_cycles: u16,
}

pub fn init_timer() -> TimerState {

    TimerState {
        div_cycles: 0,
        timer_cycles: 0,
        needed_cycles: 0,
    }
}

pub fn timer_cycle(timer_state: &mut TimerState, cycles: u16, memory: &Arc<Mutex<IoRegisters>>) {

    let tac_value = memory::timer_read(0xFF07, memory);
    let timer_enabled = utils::check_bit(tac_value, 2);
    let mut current_state = timer_state;

    if timer_enabled {
            
        current_state.needed_cycles = get_frequency(tac_value);
        current_state.timer_cycles += cycles;
        current_state.div_cycles += cycles;

        if current_state.div_cycles >= 256 {

            let div_value = memory::timer_read(0xFF04, memory);
            let new_value = div_value.overflowing_add(1);
            memory::timer_write(0xFF04, new_value.0, memory);
            current_state.div_cycles = 0;
        }

        if current_state.timer_cycles >= current_state.needed_cycles {

            let tima_value = memory::timer_read(0xFF05, memory);
            let new_value = tima_value.overflowing_add(1);
            current_state.timer_cycles = 0;

            if new_value.1 {
                    
                let if_value = memory::timer_read(0xFF0F, memory);
                let modulo_value = memory::timer_read(0xFF06, memory);
                memory::timer_write(0xFF05, modulo_value, memory);
                memory::timer_write(0xFF0F, utils::set_bit(if_value, 2), memory);
            }
            else {
                memory::timer_write(0xFF05, new_value.0, memory);
            }
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