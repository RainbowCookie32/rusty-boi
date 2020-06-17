use std::sync::Arc;
use std::sync::atomic::Ordering;
use super::memory::EmulatedMemory;

const DIV: u16 = 0xFF04;
const TIMA: u16 = 0xFF05;
const TMA: u16 = 0xFF06;
const TAC: u16 = 0xFF07;

pub struct Timer {
    div_cycles: u16,
    timer_cycles: u16,
    needed_cycles: u16,

    memory: Arc<EmulatedMemory>,
}

impl Timer {
    pub fn new(memory: Arc<EmulatedMemory>) -> Timer {
        Timer {
            div_cycles: 0,
            timer_cycles: 0,
            needed_cycles: 0,

            memory: memory
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

    pub fn step(&mut self) {
        let timer_control = self.memory.read(TAC);
        let timer_enabled = (timer_control & 4) != 0;
        let elapsed_cycles = super::GLOBAL_CYCLE_COUNTER.load(Ordering::Relaxed);

        self.div_cycles = self.div_cycles.wrapping_add(elapsed_cycles);

        if self.div_cycles >= 256 {
            let div_value = self.memory.read(DIV).wrapping_add(1);
            self.memory.write(DIV, div_value, false);
            self.div_cycles = 0;
        }

        if timer_enabled {
            self.timer_cycles += elapsed_cycles;
            self.needed_cycles = Timer::get_frequency(timer_control);

            if self.timer_cycles >= self.needed_cycles {
                let tima = self.memory.read(TIMA) as u16 + 1;

                if tima > 0xFF {
                    let if_value = self.memory.read(0xFF0F);
                    let modulo_value = self.memory.read(TMA);

                    self.memory.write(TIMA, modulo_value, false);
                    //self.memory.write(0xFF0F, if_value | (1 << 2), false);
                }
                else {
                    self.memory.write(0xFF05, tima as u8, false);
                }

                self.timer_cycles = 0;
            }
        }
    }
}