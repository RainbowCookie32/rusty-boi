use super::cpu;
use super::cpu::Memory;

use super::utils;

use sdl2::rect::Point;
use sdl2::pixels::Color;
use sdl2::video::Window;
use sdl2::render::Canvas;

pub struct GpuState
{
    pub mode: u8,
    pub modeclock: u32,
    pub line: u8,
    pub last_bg: u16,
}

pub fn init_gpu() -> GpuState {

    let initial_state = GpuState{
        mode: 0,
        modeclock: 0,
        line: 0,
        last_bg: 0x9800,
    };

    initial_state
}

pub fn gpu_tick(canvas: &mut Canvas<Window>, state: &mut GpuState, memory: &mut Memory, cycles: &u32) {

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
                    canvas.present();
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

fn get_color(bytes: &Vec<u8>, bit: u8) -> Color {

    let color_off = Color::RGB(255, 255, 255);
    let color_33 = Color::RGB(192, 192, 192);
    let color_66 = Color::RGB(96, 96, 96);
    let color_on = Color::RGB(0, 0, 0);

    if utils::check_bit(bytes[0], bit) && utils::check_bit(bytes[1], bit) {
        color_on
    }
    else if !utils::check_bit(bytes[0], bit) && utils::check_bit(bytes[1], bit) {
        color_66
    }
    else if utils::check_bit(bytes[0], bit) && !utils::check_bit(bytes[1], bit) {
        color_33
    }
    else if !utils::check_bit(bytes[0], bit) && !utils::check_bit(bytes[1], bit) {
        color_off
    }
    else {
        println!("Something's broken on the color byte, defaulting to color_off");
        color_off
    }
}