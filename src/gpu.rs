use super::cpu;
use super::cpu::Memory;

pub struct GpuState
{
    pub char_ram: Vec<u8>,
    pub bg_map: Vec<u8>,
    pub oam_mem: Vec<u8>,

    pub mode: u8,
    pub modeclock: u32,
    pub line: u8,
}

pub fn init_gpu() -> GpuState {

    let initial_state = GpuState{
        char_ram: vec![0; 6144],
        bg_map: vec![0; 2048],
        oam_mem: vec![0; 160],

        mode: 0,
        modeclock: 0,
        line: 0,
    };

    initial_state
}

pub fn gpu_tick(state: &mut GpuState, memory: &mut Memory, cycles: &u32) {

    state.modeclock += cycles;

    match state.mode {

        // OAM Read mode
        2 => {
            if state.modeclock >= 80 {

                state.modeclock = 0;
                state.mode = 3;
            }
        }

        // VRAM Read mode
        3 => {
            if state.modeclock >= 172 {

                state.modeclock = 0;
                state.mode = 0;

                // Draw a line
                
            }
        }

        // HBlank mode
        0 => {
            if state.modeclock >= 204 {
                
                state.modeclock = 0;
                state.line += 1;
                cpu::memory_write(0xFF44, state.line, memory);

                if state.line == 144 {
                    // Go into VBlank mode.
                    state.mode = 1;
                    // Send data to screen.
                }
            }
        }
        
        // VBlank mode
        1 => {
            if state.modeclock >= 456 {

                state.modeclock = 0;
                state.line += 1;
                cpu::memory_write(0xFF44, state.line, memory);

                if state.line == 154 {

                    // End of the screen, restart.
                    state.mode = 2;
                    state.line = 1;
                    cpu::memory_write(0xFF44, state.line, memory);
                }
            }
        }

        _ => panic!("Invalid GPU Mode"),
    }
}