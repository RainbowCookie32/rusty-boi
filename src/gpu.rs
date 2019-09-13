use super::cpu;
use super::cpu::Memory;

use super::utils;

use log;
use log::info;
use log::trace;

use sdl2::rect::Point;
use sdl2::pixels::Color;
use sdl2::video::Window;
use sdl2::render::Canvas;

pub struct Tile
{
    pub tile_colors: Vec<Color>,
}

pub struct BGPoint
{
    pub point: Point,
    pub color: Color,
}

pub struct GpuState
{
    pub mode: u8,
    pub mode_clock: u32,
    pub line: u8,
    pub last_bg: u16,
    pub all_tiles: Vec<Tile>,
    pub background_points: Vec<BGPoint>,
}

pub fn init_gpu() -> GpuState {

    let initial_state = GpuState{
        mode: 0,
        mode_clock: 0,
        line: 0,
        last_bg: 0x9800,
        all_tiles: Vec::new(),

        background_points: Vec::new(),
    };

    initial_state
}

pub fn gpu_tick(canvas: &mut Canvas<Window>, state: &mut GpuState, memory: &mut Memory, cycles: &u32) {

    let display_enabled = utils::check_bit(cpu::memory_read_u8(&0xFF40, memory), 7);
    
    if display_enabled {

        state.mode_clock += *cycles;
        match state.mode {

            // OAM Read mode
            2 => {
                if state.mode_clock >= 80 {

                    state.mode_clock = 0;
                    state.mode = 3;
                }
            }

            // VRAM Read mode
            3 => {
                if state.mode_clock >= 172 {

                    state.mode_clock = 0;
                    state.mode = 0;

                    // Cache all the tiles if something new was written to that VRAM area.
                    if memory.tiles_dirty {
                        info!("GPU: Regenerating tile cache");
                        make_tiles(memory, state);
                        memory.tiles_dirty = false;
                    }

                    // Cache the background if something new was written to that VRAM area and the tiles are already cached.
                    if memory.background_dirty && state.all_tiles.len() == 384 {
                        info!("GPU: Regenerating background cache");
                        make_background(memory, state);
                        memory.background_dirty = false;
                    }
                }
            }

            // HBlank mode
            0 => {
                if state.mode_clock >= 204 {
                
                    state.mode_clock = 0;
                    state.line += 1;
                    cpu::memory_write(0xFF44, state.line, memory);

                    if state.all_tiles.len() == 384
                    {
                        draw(state, canvas, memory);
                    }

                    if state.line == 144 {
                        trace!("GPU: Presenting framebuffer to SDL canvas");
                        // Go into VBlank mode.
                        state.mode = 1;
                        // Send data to screen.
                        canvas.present();
                    }
                }
            }
        
            // VBlank mode
            1 => {
                if state.mode_clock >= 456 {

                    state.mode_clock = 0;
                    state.line += 1;
                    cpu::memory_write(0xFF44, state.line, memory);

                    if state.line == 154 {

                        // End of the screen, restart.
                        state.mode = 2;
                        state.line = 1;
                        cpu::memory_write(0xFF44, state.line, memory);
                        canvas.clear();
                    }
                }
            }

            _ => panic!("Invalid GPU Mode"),
        }

    }
}


fn draw(state: &mut GpuState, canvas: &mut Canvas<Window>, memory: &mut Memory) 
{
    let scroll_x = cpu::memory_read_u8(&0xFF43, memory);
    let scroll_y = cpu::memory_read_u8(&0xFF42, memory);
    let mut point_idx: u16 = 0;
    let mut drawn_pixels: u16 = 0;

    // Index offset for the points array in case the current line is not 0.
    if state.line > 0 {
        point_idx += 256 * state.line as u16;
    }

    // Draw a whole line from the background map, skipping points that are outside the screen.
    while drawn_pixels < 256 {

        let current_point = &state.background_points[point_idx as usize];
        let mut should_draw = true;

        // If the point is outside of the screen bounds, just skip drawing it.
        // The scroll registers should keep everything important on screen.
        if current_point.point.x() + scroll_x as i32 > 160 || current_point.point.y() + scroll_y as i32 > 144 {
            should_draw = false;
            trace!("GPU: Discarding out of bounds point: X {}, Y {}", current_point.point.x() + scroll_x as i32, current_point.point.y() + scroll_y as i32);
        }

        if should_draw {
            let final_point = current_point.point.offset(scroll_x as i32, scroll_y as i32);
            trace!("GPU: Drawing at X: {} and Y {}", final_point.x(), final_point.y());
            canvas.set_draw_color(current_point.color);
            canvas.draw_point(final_point).unwrap();
        }

        point_idx += 1;
        drawn_pixels += 1;
    }
}

fn make_tiles(memory: &mut Memory, state: &mut GpuState) {

    let mut memory_position = 0;
    let mut tiles_position = 0;
    let mut new_tiles:Vec<Tile> = Vec::new();

    while memory_position < memory.char_ram.len() {

        let mut loaded_bytes = 0;
        let mut tile_bytes: Vec<u8> = vec![0; 16];

        while loaded_bytes < tile_bytes.len() {

            tile_bytes[loaded_bytes] = memory.char_ram[memory_position];
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
    
    while generated_colors < colors.len() {

        let mut bit_counter = 8;
        let tile_bytes = vec![bytes[current_byte], bytes[current_byte + 1]];

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

fn make_background(memory: &mut Memory, state: &mut GpuState) {

    let mut new_points: Vec<BGPoint> = Vec::new();
    let mut current_background = 0x9800;
    let mut generated_lines: u16 = 0;
    
    while generated_lines < 256 {

        let mut tiles: Vec<&Tile> = Vec::new();
        let mut tile_idx: usize = 0;

        // Loads tile indexes from memory, then gets the tile from GPU State and saves it to tiles.
        // 32 tiles is the maximum amount of tiles per line in the background.
        while tiles.len() < 32 {

            let target_tile = cpu::memory_read_u8(&current_background, memory);

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

    let start_idx = vec![0, 8, 15, 23, 31, 39, 47, 55];
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
    let _color_33 = Color::RGB(192, 192, 192);
    let color_66 = Color::RGB(96, 96, 96);
    let color_on = Color::RGB(0, 0, 0);

    let byte0 = utils::check_bit(bytes[0], bit);
    let byte1 = utils::check_bit(bytes[1], bit);

    if  byte0 && byte1 {
        color_on
    }
    else if !byte0 && byte1 {
        color_66
    }
    else if byte0 && !byte1 {
        color_on//color_33
    }
    else if !byte0 && !byte1 {
        color_off
    }
    else {
        color_off
    }
}