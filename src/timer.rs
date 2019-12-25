use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};

use super::memory::SharedMemory;


pub struct TimerModule {
    div_cycles: u16,
    timer_cycles: u16,
    cycles_needed: u16,

    total_cycles: Arc<AtomicU16>,
    shared_memory: Arc<SharedMemory>,
}

impl TimerModule {
    pub fn new(cycles: Arc<AtomicU16>, memory: Arc<SharedMemory>) -> TimerModule {
        TimerModule {
            div_cycles: 0,
            timer_cycles: 0,
            cycles_needed: 0,

            total_cycles: cycles,
            shared_memory: memory,
        }
    }

    pub fn timer_cycle(&mut self) {
        let tac = self.shared_memory.read(0xFF07);
        let timer_enabled = ((tac >> 2) & 1) == 1;
        
        self.div_cycles += self.total_cycles.load(Ordering::Relaxed);

        if self.div_cycles >= 256 {
            let div_value = self.shared_memory.read(0xFF04);
            self.shared_memory.write(0xFF04, div_value.wrapping_add(1), false);
            self.div_cycles = 0;
        }

        if timer_enabled {
            self.cycles_needed = TimerModule::get_timer_frequency(tac);
            self.timer_cycles = self.total_cycles.load(Ordering::Relaxed);

            if self.timer_cycles >= self.cycles_needed {
                let tima_value = self.shared_memory.read(0xFF05);
                let result = tima_value.overflowing_add(1);

                self.timer_cycles = 0;

                if result.1 {
                    let if_value = self.shared_memory.read(0xFF0F) | (1 << 2);
                    let modulo_value = self.shared_memory.read(0xFF06);

                    self.shared_memory.write(0xFF05, modulo_value, false);
                    self.shared_memory.write(0xFF06, if_value, false);
                }
                else {
                    self.shared_memory.write(0xFF05, result.0, false);
                }
            }
        }
    }

    fn get_timer_frequency(tac_value: u8) -> u16 {
        let tac_value = tac_value & 3;
    
        match tac_value {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => 0,
        }
    }
}