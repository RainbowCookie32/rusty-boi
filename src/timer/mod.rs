use super::memory::EmulatedMemory;

pub struct Timer {
    div_cycles: u16,
    timer_cycles: u16,
    needed_cycles: u16,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            div_cycles: 0,
            timer_cycles: 0,
            needed_cycles: 0,
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

    pub fn step(&mut self, cycles: u16, memory: &mut EmulatedMemory) {
        let tac_value = memory.read(0xFF07);
        let timer_enabled = ((tac_value >> 2) & 1) == 1;

        self.div_cycles += cycles;

        if self.div_cycles >= 256 {
            let div_value = memory.read(0xFF04).wrapping_add(1);
            memory.write(0xFF04, div_value, false);
            self.div_cycles = 0;
        }

        if timer_enabled {
            self.needed_cycles = Timer::get_frequency(tac_value);
            self.timer_cycles += cycles;

            if self.timer_cycles >= self.needed_cycles {
                let tima = memory.read(0xFF05).overflowing_add(1);
                self.timer_cycles = 0;

                if tima.1 {
                    let if_value = memory.read(0xFF0F);
                    let modulo_value = memory.read(0xFF06);

                    memory.write(0xFF05, modulo_value, false);
                    memory.write(0xFF0F, if_value | (1 << 2), false);
                }
                else {
                    memory.write(0xFF05, tima.0, false);
                }
            }
        }
    }
}