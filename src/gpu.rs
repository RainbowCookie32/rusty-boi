use std::ops::Neg;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use log::error;

use sdl2;
use sdl2::rect::Point;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::render::Canvas;

use super::utils;
use super::memory;
use super::emulator::InputEvent;
use super::memory::{CpuMemory, GpuMemory};


pub struct GpuState {

    pub gpu_mode: u8,
    pub gpu_cycles: u16,
    pub line: u8,
    pub tile_bank0: Vec<Vec<u8>>,
    pub tile_bank1: Vec<Vec<u8>>,
    pub background_points: Vec<u8>,

    pub palette0: Vec<Color>,
    pub palette1: Vec<Color>,
    pub palette2: Vec<Color>,

    pub bg_dirty: bool,
    pub oam_dirty: bool,
    pub tiles_dirty: bool,
}

impl GpuState {
    pub fn new() -> GpuState {

        GpuState {
            gpu_mode: 0,
            gpu_cycles: 0,
            line: 0,
            tile_bank0: vec![vec![0; 64]; 256],
            tile_bank1: vec![vec![0; 64]; 256],
            background_points: vec![0; 65536],

            palette0: vec![Color::RGB(255, 255, 255), Color::RGB(192, 192, 192), Color::RGB(96, 96, 96), Color::RGB(0, 0, 0)],
            palette1: vec![Color::RGB(255, 255, 255), Color::RGB(192, 192, 192), Color::RGB(96, 96, 96), Color::RGB(0, 0, 0)],
            palette2: vec![Color::RGB(255, 255, 255), Color::RGB(192, 192, 192), Color::RGB(96, 96, 96), Color::RGB(0, 0, 0)],

            bg_dirty: false,
            oam_dirty: false,
            tiles_dirty: false,
        }
    }
}

pub fn start_gpu(cycles: Arc<Mutex<u16>>, input_tx: Sender<InputEvent>, memory: (Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let mut gpu_state = GpuState::new();

    let sdl_context = sdl2::init().unwrap();
    let video_sys = sdl_context.video().unwrap();
    let game_window = video_sys.window("Rusty Boi - Game", 160 * 3, 144 * 3).position_centered().opengl().resizable().build().unwrap();
    let mut game_canvas = game_window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    game_canvas.set_scale(3.0, 3.0).unwrap();
    game_canvas.set_draw_color(Color::RGB(255, 255, 255));
    game_canvas.clear();
    game_canvas.present();

    loop {

        check_inputs(&mut event_pump, &input_tx);

        let lcdc = memory::gpu_read(0xFF40, &memory);
        let display = utils::check_bit(lcdc, 7);
        
        {
            let value = cycles.lock().unwrap();
            gpu_state.gpu_cycles = gpu_state.gpu_cycles.overflowing_add(*value).0;
        }

        if display {
            if gpu_state.gpu_mode == 0 && gpu_state.gpu_cycles >= 204 {
                hblank_mode(&mut gpu_state, &mut game_canvas, &memory);
            }
            else if gpu_state.gpu_mode == 1 && gpu_state.gpu_cycles >= 456 {
                vblank_mode(&mut gpu_state, &mut game_canvas, &memory);
            }
            else if gpu_state.gpu_mode == 2 && gpu_state.gpu_cycles >= 80 {
                oam_scan_mode(&mut gpu_state, &memory);
            }
            else if gpu_state.gpu_mode == 3 && gpu_state.gpu_cycles >= 172 {
                lcd_transfer_mode(&mut gpu_state, &memory);
            }
        }
    }
}

// GPU Modes

fn hblank_mode(state: &mut GpuState, canvas: &mut Canvas<Window>, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let mut stat_value = memory::gpu_read(0xFF41, &memory);

    stat_value = utils::reset_bit(stat_value, 1);
    stat_value = utils::reset_bit(stat_value, 0);
    memory::gpu_write(0xFF41, stat_value, memory);

    if state.background_points.len() >= 65536 {
        draw(state, canvas, memory);
    }

    state.gpu_cycles = 0;
    state.line += 1;
    memory::gpu_write(0xFF44, state.line, memory);
    
    if state.line == 144 {
        state.gpu_mode = 1;
        canvas.present();
    }

    if utils::check_bit(stat_value, 3) {

        let if_value = utils::set_bit(memory::gpu_read(0xFF0F, memory), 2);
        memory::gpu_write(0xFF0F, if_value, memory);
    }
}

fn vblank_mode(state: &mut GpuState, canvas: &mut Canvas<Window>, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let mut if_value = memory::gpu_read(0xFF0F, memory);
    let mut stat_value = memory::gpu_read(0xFF41, memory);

    state.gpu_cycles = 0;
    state.line += 1;
    memory::gpu_write(0xFF44, state.line, memory);
    
    if_value = utils::set_bit(if_value, 0);
    memory::gpu_write(0xFF0F, if_value, memory);

    stat_value = utils::reset_bit(stat_value, 1);
    stat_value = utils::set_bit(stat_value, 0);
    memory::gpu_write(0xFF41, stat_value, memory);

    if state.line == 154 {

        state.gpu_mode = 2;
        state.line = 0;

        memory::gpu_write(0xFF44, 1, memory);
        canvas.clear();
    }
}

fn oam_scan_mode(state: &mut GpuState, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let mut stat_value = memory::gpu_read(0xFF41, memory);

    state.gpu_cycles = 0;
    state.gpu_mode = 3;
    stat_value = utils::set_bit(stat_value, 1);
    stat_value = utils::reset_bit(stat_value, 0);
    memory::gpu_write(0xFF41, stat_value, memory);

    if utils::check_bit(stat_value, 5) {

        let if_value = utils::set_bit(memory::gpu_read(0xFF0F, memory), 2);
        memory::gpu_write(0xFF0F, if_value, memory);
    }
}

fn lcd_transfer_mode(state: &mut GpuState, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let mut stat_value = memory::gpu_read(0xFF41, memory);

    stat_value = utils::set_bit(stat_value, 1);
    stat_value = utils::set_bit(stat_value, 0);
    memory::gpu_write(0xFF41, stat_value, memory);

    state.gpu_cycles = 0;
    state.gpu_mode = 0;

    {
        let mut mem = memory.1.lock().unwrap();
        state.bg_dirty = mem.background_dirty;
        state.oam_dirty = mem.oam_dirty;
        state.tiles_dirty = mem.tiles_dirty;

        mem.background_dirty = false;
        mem.oam_dirty = false;
        mem.tiles_dirty = false;
    }

    if state.tiles_dirty {
        make_tiles(state, 0, memory);
        make_tiles(state, 1, memory);
        state.tiles_dirty = false;
        state.bg_dirty = true;
    }

    if state.tile_bank0.len() > 0 {
        make_background(state, memory);
        state.bg_dirty = false;
    }
}

// Drawing to screen.

fn draw(state: &mut GpuState, canvas: &mut Canvas<Window>, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let scroll_x = (memory::gpu_read(0xFF43, memory) as i32).neg();
    let scroll_y = (memory::gpu_read(0xFF42, memory) as i32).neg();
    let mut point_idx: u16 = 0;
    let mut drawn_pixels: u16 = 0;

    // Index offset for the points array in case the current line is not 0.
    point_idx += 256 * state.line as u16;

    // Draw a whole line from the background map.
    while drawn_pixels < 256 {

        let color = state.palette0[state.background_points[point_idx as usize] as usize];
        let final_point = Point::new(drawn_pixels as i32 + scroll_x, state.line as i32 + scroll_y);

        canvas.set_draw_color(color);
        canvas.draw_point(final_point).unwrap();
        point_idx += 1;
        drawn_pixels += 1;
    }
}


// Tile and Background cache generation

fn make_tiles(state: &mut GpuState, target_bank: u8, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let start_position = if target_bank == 0 {0x8000} else {0x8800};
    let end_position = if target_bank == 0 {0x8FFF} else {0x97FF};
    let mut memory_position = start_position;
    let mut tiles_position = 0;

    while memory_position < end_position {

        let mut loaded_bytes = 0;
        let mut tile_bytes: Vec<u8> = vec![0; 16];

        while loaded_bytes < 16 {

            tile_bytes[loaded_bytes] = memory::gpu_read(memory_position, memory);
            memory_position += 1;
            loaded_bytes += 1;
        }

        if target_bank == 0 {
            state.tile_bank0[tiles_position as usize] = make_tile(&tile_bytes);
        }
        else {
            state.tile_bank1[tiles_position as usize] = make_tile(&tile_bytes);
        }

        tiles_position += 1;
    }
}

fn make_tile(bytes: &Vec<u8>) -> Vec<u8> {

    let mut tile_index = 0;
    let mut processed_bytes = 0;
    let mut generated_tile: Vec<u8> = vec![0; 64];

    while processed_bytes < 16 {

        let mut current_bit = 8;
        let bytes_to_check = (bytes[processed_bytes], bytes[processed_bytes + 1]);
        processed_bytes += 2;

        while current_bit != 0 {

            current_bit -= 1;
            let bits = (utils::check_bit(bytes_to_check.0, current_bit), utils::check_bit(bytes_to_check.1, current_bit));
            if bits.0 && bits.1 {generated_tile[tile_index] = 3}
            else if !bits.0 && bits.1 {generated_tile[tile_index] = 2}
            else if bits.0 && !bits.1 {generated_tile[tile_index] = 1}
            else if !bits.0 && !bits.1 {generated_tile[tile_index] = 0}
            tile_index += 1;
        }
    }

    generated_tile
}

fn make_background(state: &mut GpuState, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let mut generated_lines: u16 = 0;
    let mut current_background = if utils::check_bit(memory::gpu_read(0xFF40, memory), 3) {0x9C00} else {0x9800};

    let lcdc_value =  utils::check_bit(memory::gpu_read(0xFF40, memory), 4);
    let tile_bank = if lcdc_value {&state.tile_bank0} else {&state.tile_bank1};

    let mut background_idx: usize = 0;
        
    while generated_lines < 256 {

        let mut tiles: Vec<&Vec<u8>> = Vec::new();
        let mut tile_idx: usize = 0;

        // Loads tile indexes from memory, then gets the tile from GPU State and saves it to tiles.
        // 32 tiles is the maximum amount of tiles per line in the background.
        while tile_idx < 32 {

            let bg_value = memory::gpu_read(current_background, memory);
            if lcdc_value {
                let target_tile = bg_value;
                tiles.insert(tile_idx, &tile_bank[target_tile as usize]);
                tile_idx += 1;
                current_background += 1;
            }
            else {
                let target_tile = (bg_value as i8 as i16 + 128) as u16;
                tiles.insert(tile_idx, &tile_bank[target_tile as usize]);
                tile_idx += 1;
                current_background += 1;
            }
        }

        let mut tile_line = 0;

        while tile_line < 8 {

            let line = make_background_line(&tiles, tile_line);
            for point in line.into_iter() {
                state.background_points[background_idx] = point;
                background_idx += 1;
            }
            tile_line += 1;
            generated_lines += 1;
        }
    }
}

fn make_background_line(tiles: &Vec<&Vec<u8>>, tile_line: u8) -> Vec<u8> {

    let start_idx = vec![0, 8, 16, 24, 32, 40, 48, 56];
    let mut generated_points = 0;
    let mut processed_tiles = 0;
    let mut final_line: Vec<u8> = vec![0; 256];

    while generated_points < 256 {

        while processed_tiles < 32 {

            let end_index = start_idx[tile_line as usize] + 8;
            let mut color_index = start_idx[tile_line as usize];
            let current_tile = tiles[processed_tiles as usize];

            while color_index < end_index {

                final_line[generated_points] = current_tile[color_index];

                color_index += 1;
                generated_points += 1;
            }
            processed_tiles += 1;
        }
    }  

    final_line
}


fn check_inputs(pump: &mut sdl2::EventPump, input_tx: &Sender<InputEvent>) {

    for event in pump.poll_iter() {
        match event {
            Event::Quit{..} => {
                input_tx.send(InputEvent::Quit).unwrap();
            }
            Event::KeyDown{keycode: Some(Keycode::A), ..} => {
                let mut count = 5;
                while count > 0 {
                    let result = input_tx.send(InputEvent::APressed);
                    match result {
                        Ok(_) => {},
                        Err(error) => {error!("Input: Failed to send event to CPU, error {}", error); count = 0},
                    }
                    count -= 1;
                }
            },
            Event::KeyDown{keycode: Some(Keycode::S), ..} => {
                let mut count = 5;
                while count > 0 {
                    let result = input_tx.send(InputEvent::BPressed);
                    match result {
                        Ok(_) => {},
                        Err(error) => {error!("Input: Failed to send event to CPU, error {}", error); count = 0},
                    }
                    count -= 1;
                }
            },
            Event::KeyDown{keycode: Some(Keycode::Return), ..} => {
                let mut count = 5;
                while count > 0 {
                    let result = input_tx.send(InputEvent::StartPressed);
                    match result {
                        Ok(_) => {},
                        Err(error) => {error!("Input: Failed to send event to CPU, error {}", error); count = 0},
                    }
                    count -= 1;
                }
            },
            Event::KeyDown{keycode: Some(Keycode::RShift), ..} => {
                let mut count = 5;
                while count > 0 {
                    let result = input_tx.send(InputEvent::SelectPressed);
                    match result {
                        Ok(_) => {},
                        Err(error) => {error!("Input: Failed to send event to CPU, error {}", error); count = 0},
                    }
                    count -= 1;
                }
            },
            Event::KeyDown{keycode: Some(Keycode::Up), ..} => {
                let mut count = 5;
                while count > 0 {
                    let result = input_tx.send(InputEvent::UpPressed);
                    match result {
                        Ok(_) => {},
                        Err(error) => {error!("Input: Failed to send event to CPU, error {}", error); count = 0},
                    }
                    count -= 1;
                }
            },
            Event::KeyDown{keycode: Some(Keycode::Down), ..} => {
                let mut count = 5;
                while count > 0 {
                    let result = input_tx.send(InputEvent::DownPressed);
                    match result {
                        Ok(_) => {},
                        Err(error) => {error!("Input: Failed to send event to CPU, error {}", error); count = 0},
                    }
                    count -= 1;
                }
            },
            Event::KeyDown{keycode: Some(Keycode::Left), ..} => {
                let mut count = 5;
                while count > 0 {
                    let result = input_tx.send(InputEvent::LeftPressed);
                    match result {
                        Ok(_) => {},
                        Err(error) => {error!("Input: Failed to send event to CPU, error {}", error); count = 0},
                    }
                    count -= 1;
                }
            },
            Event::KeyDown{keycode: Some(Keycode::Right), ..} => {
                let mut count = 5;
                while count > 0 {
                    let result = input_tx.send(InputEvent::RightPressed);
                    match result {
                        Ok(_) => {},
                        Err(error) => {error!("Input: Failed to send event to CPU, error {}", error); count = 0},
                    }
                    count -= 1;
                }
            },
            _ => {}
        }
    }
}