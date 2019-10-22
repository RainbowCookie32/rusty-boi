use std::ops::Neg;
use std::sync::{Arc, Mutex};

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

#[derive(Clone, Copy)]
pub struct SpriteData {
    pub y_position: u8,
    pub x_position: u8,
    pub tile_number: u8,

    pub priority: bool,
    // TODO: Actually use the X and Y flip when generating points.
    pub y_flip: bool,
    pub x_flip: bool,
}

impl SpriteData {
    pub fn new(bytes: Vec<u8>) -> SpriteData {
        SpriteData {
            y_position: bytes[0],
            x_position: bytes[1],
            tile_number: bytes[2],
            priority: utils::check_bit(bytes[3], 7),
            y_flip: utils::check_bit(bytes[3], 6),
            x_flip: utils::check_bit(bytes[3], 5),
        }
    }
}

pub struct BGPoint {

    pub point: Point,
    pub color: Color,
}

pub struct GpuState {

    pub gpu_mode: u8,
    pub gpu_cycles: u16,
    pub line: u8,
    pub tiles_0: Vec<Tile>,
    pub tiles_1: Vec<Tile>,
    pub sprites: Vec<SpriteData>,
    pub background_points: Vec<BGPoint>,

    pub bg_dirty: bool,
    pub tiles_dirty: bool,
}

pub fn init_gpu() -> GpuState {

    GpuState {
        gpu_mode: 0,
        gpu_cycles: 0,
        line: 0,
        tiles_0: Vec::new(),
        tiles_1: Vec::new(),
        sprites: Vec::new(),
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

    if state.tiles_dirty {
        make_tiles(state, memory);
        state.tiles_dirty = false;
        state.bg_dirty = true;
    }

    if state.tiles_0.len() != 0 {
        make_tiles(state, memory);
        make_sprites(state, memory);
        make_background(state, memory);
        add_sprites_to_background(state);
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

    let mut start_position = 0x8000;
    let mut end_position = 0x8FFF;
    let mut memory_position = start_position;
    let mut tiles_position = 0;
    let mut new_tiles:Vec<Tile> = Vec::new();

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

    state.tiles_0 = new_tiles;

    start_position = 0x8800;
    end_position = 0x97FF;
    memory_position = start_position;
    tiles_position = 0;
    new_tiles = Vec::new();

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

    state.tiles_1 = new_tiles;
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

fn make_sprites(state: &mut GpuState, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let mut index: usize = 0;
    let mut current_address = 0xFE00;
    let mut generated_sprites: Vec<SpriteData> = vec![SpriteData::new(vec![0; 4]); 40];

    while current_address < 0xFEA0 {

        let mut bytes: Vec<u8> = vec![0; 4];
        let mut loaded_bytes: usize = 0;

        while loaded_bytes < 4 {
            bytes[loaded_bytes] = memory::gpu_read(current_address, &memory);
            loaded_bytes += 1;
            current_address += 1;
        }

        generated_sprites[index] = SpriteData::new(bytes);
        index += 1;
    }

    state.sprites = generated_sprites;
}

fn make_background(state: &mut GpuState, memory: &(Arc<Mutex<CpuMemory>>, Arc<Mutex<GpuMemory>>)) {

    let mut generated_lines: u16 = 0;
    let mut new_points: Vec<BGPoint> = Vec::new();
    let mut current_background = if utils::check_bit(memory::gpu_read(0xFF40, memory), 3) {0x9C00} else {0x9800};

    let lcdc_value =  utils::check_bit(memory::gpu_read(0xFF40, memory), 4);
    let tile_bank = if lcdc_value {&state.tiles_0} else {&state.tiles_1};
        
    while generated_lines < 256 {

        let mut tiles: Vec<&Tile> = Vec::new();
        let mut tile_idx: usize = 0;

        // Loads tile indexes from memory, then gets the tile from GPU State and saves it to tiles.
        // 32 tiles is the maximum amount of tiles per line in the background.
        while tiles.len() < 32 {

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

fn add_sprites_to_background(state: &mut GpuState) {

    for sprite in state.sprites.iter() {

        if sprite.x_position > 0 && sprite.y_position > 0 {
            
            let used_tile = &state.tiles_0[sprite.tile_number as usize];
            let initial_x = sprite.x_position.wrapping_sub(8);
            let initial_y = sprite.y_position.wrapping_sub(16);

            let mut points = 0;
            let mut current_x = initial_x;
            let mut current_y = initial_y;

            while points < 64 {

                let background_index = (256 * current_y as u16) + (current_x as u16);
                let target_point = &state.background_points[background_index as usize];
                let new_point = BGPoint {
                    point: Point::new(current_x as i32, current_y as i32),
                    color: used_tile.tile_colors[points],
                };

                if !sprite.priority {
                    state.background_points[background_index as usize] = new_point;
                }
                else {
                    if target_point.color == Color::RGB(255, 255, 255) {
                        state.background_points[background_index as usize] = new_point;
                    }
                }

                if current_x == initial_x.wrapping_add(7) {
                    current_y = current_y.wrapping_add(1);
                    current_x = initial_x;
                }
                else {
                    current_x = current_x.wrapping_add(1);
                }

                points += 1;
            }
        }
    }
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