use std::sync::Arc;

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

pub fn timer_cycle(timer_state: &mut TimerState, cycles: u16, memory: &Arc<IoRegisters>) {

    let tac_value = memory::timer_read(0xFF07, memory);
    let timer_enabled = utils::check_bit(tac_value, 2);

    timer_state.div_cycles += cycles;

    if timer_state.div_cycles >= 256 {

        let div_value = memory::timer_read(0xFF04, memory);
        let new_value = div_value.overflowing_add(1).0;
        memory::timer_write(0xFF04, new_value, memory);
        timer_state.div_cycles = 0;
    }

    if timer_enabled {
            
        timer_state.needed_cycles = get_frequency(tac_value);
        timer_state.timer_cycles += cycles;

        if timer_state.timer_cycles >= timer_state.needed_cycles {

            let tima_value = memory::timer_read(0xFF05, memory);
            let new_value = tima_value.overflowing_add(1);
            timer_state.timer_cycles = 0;

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

    let value = tac & 3;
    
    match value {
        0 => 1024,
        1 => 16,
        2 => 64,
        3 => 256,
        _ => 0,
    }
}