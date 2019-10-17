use std::ops::Neg;
use std::sync::{Arc, Mutex};

use log::info;

use sdl2::rect::Point;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::render::Canvas;

use super::utils;
use super::memory;
use super::memory::{CpuMemory, GpuMemory};


pub struct Tile {

    pub tile_colors: Vec<Color>,
}

pub struct BGPoint {

    pub point: Point,
    pub color: Color,
}

pub struct GpuState {

    pub gpu_mode: u8,
    pub gpu_cycles: u16,
    pub line: u8,
    pub all_tiles: Vec<Tile>,
    pub background_points: Vec<BGPoint>,

    pub bg_dirty: bool,
    pub tiles_dirty: bool,
}

pub fn init_gpu() -> GpuState {

    GpuState {
        gpu_mode: 0,
        gpu_cycles: 0,
        line: 0,
        all_tiles: Vec::new(),
        background_points: Vec::new(),
        bg_dirty: false,
        tiles_dirty: false,
    }
}

pub fn gpu_loop(cycles: &Arc<Mutex<u16>>, state: &mut GpuState, canvas: &mut Canvas<Window>, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let lcdc_stat = memory::gpu_read(0xFF40, &memory);
    let display_enabled = utils::check_bit(lcdc_stat, 7);

    let mem = memory.1.lock().unwrap();
    state.bg_dirty = mem.background_dirty;
    state.tiles_dirty = mem.tiles_dirty;
    std::mem::drop(mem);

    let cyc_mut = cycles.lock().unwrap();
    state.gpu_cycles = state.gpu_cycles.overflowing_add(*cyc_mut).0;
    std::mem::drop(cyc_mut);

    if display_enabled {

        if state.gpu_mode == 0 && state.gpu_cycles >= 204 {
            hblank_mode(state, canvas, &memory);
        }
        else if state.gpu_mode == 1 && state.gpu_cycles >= 456 {
            vblank_mode(state, canvas, &memory);
        }
        else if state.gpu_mode == 2 && state.gpu_cycles >= 80 {
            oam_scan_mode(state, &memory);
        }
        else if state.gpu_mode == 3 && state.gpu_cycles >= 172 {
            lcd_transfer_mode(state, &memory);
            let mut mem = memory.1.lock().unwrap();
            mem.background_dirty = false;
            mem.tiles_dirty = false;
        }

    }
}

// GPU Modes

fn hblank_mode(state: &mut GpuState, canvas: &mut Canvas<Window>, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let mut stat_value = memory::gpu_read(0xFF41, &memory);

    stat_value = utils::reset_bit(stat_value, 1);
    stat_value = utils::reset_bit(stat_value, 0);
    memory::gpu_write(0xFF41, stat_value, memory);

    if state.all_tiles.len() >= 128 && state.background_points.len() >= 65536 {
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
        state.line = 1;

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

    if state.tiles_dirty {
        make_tiles(state, memory);
        state.tiles_dirty = false;
        state.bg_dirty = true;
    }
    if state.bg_dirty && state.all_tiles.len() != 0 {
        make_tiles(state, memory);
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

        let current_point = &state.background_points[point_idx as usize];
        let final_point = current_point.point.offset(scroll_x, scroll_y);

        canvas.set_draw_color(current_point.color);
        canvas.draw_point(final_point).unwrap();
        point_idx += 1;
        drawn_pixels += 1;
    }
}


// Tile and Background cache generation

fn make_tiles(state: &mut GpuState, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let start_position = 0x8000;
    let end_position = 0x97FF;
    let mut memory_position = start_position;
    let mut tiles_position = 0;
    let mut new_tiles:Vec<Tile> = Vec::new();

    info!("GPU: Regenerating tile cache");

    while memory_position < end_position {

        let mut loaded_bytes = 0;
        let mut tile_bytes: Vec<u8> = vec![0; 16];

        while loaded_bytes < tile_bytes.len() {

            tile_bytes[loaded_bytes] = memory::gpu_read(memory_position, memory);
            memory_position += 1;
            loaded_bytes += 1;
        }

        new_tiles.insert(tiles_position, make_tile(&tile_bytes));
        tiles_position += 1;
    }

    state.all_tiles = new_tiles;
}

fn make_tile(bytes: &Vec<u8>) -> Tile {

    let new_tile: Tile;
    let mut color_index = 0;
    let mut current_byte = 0;
    let mut generated_colors = 0;
    let mut colors: Vec<Color> = vec![Color::RGB(255, 255, 255); 64];
    
    while generated_colors < 64 {

        let mut bit_counter = 8;
        let tile_bytes = vec![bytes[current_byte], bytes[current_byte + 1]];

        // If both bytes are zero, then we won't have colors since all bits will be 0.
        // Just skip checking them if that's the case and move on to the next ones.
        if tile_bytes[0] == 0 && tile_bytes[1] == 0 {
            generated_colors += 8;
            color_index += 8;
        }
        else {
            while bit_counter != 0 {

                bit_counter -= 1;
                colors[color_index] = get_color(&tile_bytes, bit_counter);
                color_index += 1;
                generated_colors += 1;
            }
        }
        
        current_byte += 2;
    }

    new_tile = Tile { tile_colors: colors };
    new_tile
}

fn make_background(state: &mut GpuState, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let mut generated_lines: u16 = 0;
    let mut new_points: Vec<BGPoint> = Vec::new();
    let mut current_background = if utils::check_bit(memory::gpu_read(0xFF40, memory), 3) {0x9C00} else {0x9800};
    
    info!("GPU: Regenerating background cache");
    
    while generated_lines < 256 {

        let mut tiles: Vec<&Tile> = Vec::new();
        let mut tile_idx: usize = 0;

        // Loads tile indexes from memory, then gets the tile from GPU State and saves it to tiles.
        // 32 tiles is the maximum amount of tiles per line in the background.
        while tiles.len() < 32 {

            let target_tile = memory::gpu_read(current_background, memory);
            tiles.insert(tile_idx, &state.all_tiles[target_tile as usize]);
            tile_idx += 1;
            current_background += 1;
        }

        let mut tile_line = 0;

        while tile_line < 8 {

            new_points.append(&mut make_background_line(&tiles, tile_line, generated_lines as u8));
            tile_line += 1;
            generated_lines += 1;
        }
    }

    state.background_points = new_points;
}

fn make_background_line(tiles: &Vec<&Tile>, tile_line: u8, screen_line: u8) -> Vec<BGPoint> {

    let start_idx = vec![0, 8, 16, 24, 32, 40, 48, 56];
    let mut generated_points = 0;
    let mut processed_tiles = 0;
    let mut final_line: Vec<BGPoint> = Vec::new();

    while generated_points < 256 {

        while processed_tiles < 32 {

            let mut color_index = start_idx[tile_line as usize];
            let current_tile = &tiles[processed_tiles as usize];

            while color_index < start_idx[tile_line as usize] + 8 {

                let generated_point = Point::new(generated_points as i32, screen_line as i32);
                let generated_color = current_tile.tile_colors[color_index];
                let bg_point = BGPoint{ point: generated_point, color: generated_color };

                final_line.insert(generated_points as usize, bg_point);

                color_index += 1;
                generated_points += 1;
            }

            processed_tiles += 1;
        }
    }  

    final_line
}

fn get_color(bytes: &Vec<u8>, bit: u8) -> Color {

    let color_off = Color::RGB(255, 255, 255);
    let color_33 = Color::RGB(192, 192, 192);
    let color_66 = Color::RGB(96, 96, 96);
    let color_on = Color::RGB(0, 0, 0);

    let byte0 = utils::check_bit(bytes[0], bit);
    let byte1 = utils::check_bit(bytes[1], bit);

    // TODO: Implement color palettes to fix wrong colors being used.
    if  byte0 && byte1 {
        color_on
    }
    else if !byte0 && byte1 {
        color_66
    }
    else if byte0 && !byte1 {
        color_33
    }
    else if !byte0 && !byte1 {
        color_off
    }
    else {
        color_off
    }
}