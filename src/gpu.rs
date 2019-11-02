use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;

use log::error;

use sdl2;

use sdl2::rect::Rect;
use sdl2::rect::Point;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use sdl2::video::Window;
use sdl2::video::WindowContext;

use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;

use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;

use super::utils;
use super::memory;
use super::memory::GeneralMemory;
use super::emulator::InputEvent;


pub struct SpriteData {
    pub x: u8,
    pub y: u8,
    pub data: Texture,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl SpriteData {
    pub fn new(coords: (u8, u8), flip: (bool, bool), data: Texture) -> SpriteData {
        SpriteData {
            x: coords.0,
            y: coords.1,
            data: data,
            flip_x: flip.0,
            flip_y: flip.1,
        }
    }
}

pub struct GpuState {

    pub gpu_mode: u8,
    pub gpu_cycles: u16,
    pub line: u8,

    pub lcd_enabled: bool,

    pub scroll_x: u8,
    pub scroll_y: u8,

    pub background_tilemap: (u16, u16),
    pub background_enabled: bool,
    
    pub window_tilemap: (u16, u16),
    pub window_enabled: bool,
    pub window_x: u8,
    pub window_y: u8,

    pub tiles_area: (u16, u16),

    pub big_sprites: bool,
    pub sprites_enabled: bool,

    pub sprites: Vec<SpriteData>,
    pub tile_bank0: Vec<Vec<u8>>,
    pub tile_bank1: Vec<Vec<u8>>,
    pub background_points: Vec<u8>,
    pub window_points: Vec<u8>,

    pub tile_palette: Vec<Color>,
    pub sprites_palettes: Vec<Vec<Color>>,
    pub tile_palette_dirty: bool,
    pub sprite_palettes_dirty: bool,

    pub tiles_dirty_flags: u8,
    pub sprites_dirty_flags: u8,
    pub background_dirty_flags: u8,

    pub frames: u16,
}

impl GpuState {
    pub fn new() -> GpuState {

        GpuState {
            gpu_mode: 0,
            gpu_cycles: 0,
            line: 0,

            lcd_enabled: false,
            scroll_x: 0,
            scroll_y: 0,

            background_tilemap: (0x9800, 0x9BFF),
            background_enabled: false,

            window_tilemap: (0x9800, 0x9BFF),
            window_enabled: false,
            window_x: 0,
            window_y: 0,

            tiles_area: (0x8800, 0x97FF),

            big_sprites: false,
            sprites_enabled: false,

            sprites: Vec::new(),
            tile_bank0: vec![vec![0; 64]; 256],
            tile_bank1: vec![vec![0; 64]; 256],
            background_points: vec![0; 65536],
            window_points: vec![0; 65536],

            tile_palette: vec![Color::RGBA(255, 255, 255, 0), Color::RGBA(192, 192, 192, 255), Color::RGBA(96, 96, 96, 255), 
            Color::RGBA(0, 0, 0, 255)],
            sprites_palettes: vec![vec![Color::RGBA(255, 255, 255, 0), Color::RGBA(192, 192, 192, 255), Color::RGBA(96, 96, 96, 255), 
            Color::RGBA(0, 0, 0, 255)]; 2],
            tile_palette_dirty: false,
            sprite_palettes_dirty: false,

            tiles_dirty_flags: 0,
            sprites_dirty_flags: 0,
            background_dirty_flags: 0,

            frames: 0,
        }
    }
}

pub fn start_gpu(cycles: Arc<AtomicU16>, memory: Arc<GeneralMemory>, input_tx: Sender<InputEvent>) {

    let mut gpu_state = GpuState::new();

    let sdl_context = sdl2::init().unwrap();
    let video_sys = sdl_context.video().unwrap();
    let game_window = video_sys.window("Rusty Boi - Game - FPS: 0", 160 * 4, 144 * 4).position_centered().opengl().resizable().build().unwrap();
    let mut game_canvas = game_window.into_canvas().present_vsync().build().unwrap();
    let creator = game_canvas.texture_creator();

    let mut event_pump = sdl_context.event_pump().unwrap();

    game_canvas.set_scale(4.0, 4.0).unwrap();
    game_canvas.set_draw_color(Color::RGB(255, 255, 255));
    game_canvas.clear();
    game_canvas.present();

    let mut fps_timer = std::time::Instant::now();

    loop {

        check_inputs(&mut event_pump, &input_tx);
        update_gpu_values(&mut gpu_state, &memory);
        gpu_state.gpu_cycles = gpu_state.gpu_cycles.overflowing_add(cycles.load(Ordering::Relaxed)).0;

        if gpu_state.lcd_enabled {

            if gpu_state.tile_palette_dirty {
                gpu_state.tile_palette = make_palette(memory::gpu_read(0xFF47, &memory));
                gpu_state.tile_palette_dirty = false;
            }
            if gpu_state.sprite_palettes_dirty {
                gpu_state.sprites_palettes[0] = make_palette(memory::gpu_read(0xFF48, &memory));
                gpu_state.sprites_palettes[1] = make_palette(memory::gpu_read(0xFF49, &memory));
                // Regenerate the sprites cache after modifying the palettes.
                gpu_state.sprites_dirty_flags = gpu_state.sprites_dirty_flags.wrapping_add(1);
                gpu_state.sprite_palettes_dirty = false;
            }

            if gpu_state.gpu_mode == 0 && gpu_state.gpu_cycles >= 204 {
                hblank_mode(&mut gpu_state, &mut game_canvas, &memory);
            }
            else if gpu_state.gpu_mode == 1 && gpu_state.gpu_cycles >= 456 {
                vblank_mode(&mut gpu_state, &mut game_canvas, &memory);
            }
            else if gpu_state.gpu_mode == 2 && gpu_state.gpu_cycles >= 80 {
                oam_scan_mode(&mut gpu_state, &creator, &memory);
            }
            else if gpu_state.gpu_mode == 3 && gpu_state.gpu_cycles >= 172 {
                lcd_transfer_mode(&mut gpu_state, &memory);
            }

            let lyc = memory::gpu_read(0xFF45, &memory);
            
            if lyc == gpu_state.line {
                let mut stat = memory::gpu_read(0xFF41, &memory);
                let mut if_value = memory::gpu_read(0xFF0F, &memory);

                if utils::check_bit(stat, 6) {
                    if_value = utils::set_bit(if_value, 1);
                    memory::gpu_write(0xFF0F, if_value, &memory);
                }

                stat = utils::set_bit(stat, 2);
                memory::gpu_write(0xFF41, stat, &memory);
            }
        }

        if fps_timer.elapsed() >= std::time::Duration::from_millis(1000) && gpu_state.frames > 0 {

            let framerate = format!("Rusty Boi - Game - FPS: {:#?}", gpu_state.frames as u64 / fps_timer.elapsed().as_secs());
            game_canvas.window_mut().set_title(&framerate).unwrap();
            fps_timer = std::time::Instant::now();
            gpu_state.frames = 0;
        }
    }
}

fn update_gpu_values(state: &mut GpuState, memory: &Arc<GeneralMemory>) {

    let lcdc = memory::gpu_read(0xFF40, memory);
    state.lcd_enabled = utils::check_bit(lcdc, 7);
    state.window_tilemap = if utils::check_bit(lcdc, 6) {(0x9C00, 0x9FFF)} else {(0x9800, 0x9BFF)};
    state.window_enabled = utils::check_bit(lcdc, 5);
    state.tiles_area = if utils::check_bit(lcdc, 4) {(0x8000, 0x8FFF)} else {(0x8800, 0x97FF)};
    state.background_tilemap = if utils::check_bit(lcdc, 3) {(0x9C00, 0x9FFF)} else {(0x9800, 0x9BFF)};
    state.big_sprites = utils::check_bit(lcdc, 2);
    state.sprites_enabled = utils::check_bit(lcdc, 1);
    state.background_enabled = utils::check_bit(lcdc, 0);

    state.scroll_y = memory::gpu_read(0xFF42, memory);
    state.scroll_x = memory::gpu_read(0xFF43, memory);
    state.window_y = memory::gpu_read(0xFF4A, memory);
    state.window_x = memory::gpu_read(0xFF4B, memory);

    state.tiles_dirty_flags = memory.tiles_dirty_flags.load(Ordering::Relaxed);
    state.sprites_dirty_flags = memory.sprites_dirty_flags.load(Ordering::Relaxed);
    state.background_dirty_flags = memory.background_dirty_flags.load(Ordering::Relaxed);
    state.tile_palette_dirty = memory.tile_palette_dirty.load(Ordering::Relaxed);
    state.sprite_palettes_dirty = memory.sprite_palettes_dirty.load(Ordering::Relaxed);

    memory.tile_palette_dirty.store(false, Ordering::Relaxed);
    memory.sprite_palettes_dirty.store(false, Ordering::Relaxed);
}

// GPU Modes

fn hblank_mode(state: &mut GpuState, canvas: &mut Canvas<Window>, memory: &Arc<GeneralMemory>) {

    let mut stat_value = memory::gpu_read(0xFF41, &memory);

    stat_value = utils::reset_bit(stat_value, 1);
    stat_value = utils::reset_bit(stat_value, 0);
    memory::gpu_write(0xFF41, stat_value, &memory);

    if state.background_enabled {draw_background(state, canvas)}
    if state.sprites_enabled {draw_sprites(state, canvas)}
    if state.window_enabled {draw_window(state, canvas)};

    state.gpu_cycles = 0;
    state.line += 1;
    memory::gpu_write(0xFF44, state.line, &memory);
    
    if state.line == 144 {
        state.gpu_mode = 1;
        state.frames += 1;
        canvas.present();
    }

    if utils::check_bit(stat_value, 3) {
        let if_value = utils::set_bit(memory::gpu_read(0xFF0F, memory), 1);
        memory::gpu_write(0xFF0F, if_value, &memory);
    }
}

fn vblank_mode(state: &mut GpuState, canvas: &mut Canvas<Window>, memory: &Arc<GeneralMemory>) {
    
    let mut if_value = memory::gpu_read(0xFF0F, memory);
    let mut stat_value = memory::gpu_read(0xFF41, memory);

    state.gpu_cycles = 0;
    state.line += 1;
    memory::gpu_write(0xFF44, state.line, &memory);
    
    if_value = utils::set_bit(if_value, 0);
    memory::gpu_write(0xFF0F, if_value, &memory);

    stat_value = utils::reset_bit(stat_value, 1);
    stat_value = utils::set_bit(stat_value, 0);
    memory::gpu_write(0xFF41, stat_value, &memory);

    if state.line == 154 {

        state.gpu_mode = 2;
        state.line = 0;

        canvas.clear();
        memory::gpu_write(0xFF44, 1, &memory);
    }
}

fn oam_scan_mode(state: &mut GpuState, creator: &TextureCreator<WindowContext>, memory: &Arc<GeneralMemory>) {

    let mut stat_value = memory::gpu_read(0xFF41, memory);

    state.gpu_cycles = 0;
    state.gpu_mode = 3;
    stat_value = utils::set_bit(stat_value, 1);
    stat_value = utils::reset_bit(stat_value, 0);
    memory::gpu_write(0xFF41, stat_value, &memory);
    
    if state.sprites_dirty_flags > 0 {
        make_sprites(state, creator, memory);
        state.sprites_dirty_flags -= 1;
        memory.sprites_dirty_flags.fetch_sub(1, Ordering::Relaxed);
    }

    if utils::check_bit(stat_value, 5) {

        let if_value = utils::set_bit(memory::gpu_read(0xFF0F, memory), 1);
        memory::gpu_write(0xFF0F, if_value, &memory);
    }
}

fn lcd_transfer_mode(state: &mut GpuState, memory: &Arc<GeneralMemory>) {

    let mut stat_value = memory::gpu_read(0xFF41, memory);

    stat_value = utils::set_bit(stat_value, 1);
    stat_value = utils::set_bit(stat_value, 0);
    memory::gpu_write(0xFF41, stat_value, &memory);

    state.gpu_cycles = 0;
    state.gpu_mode = 0;

    if state.tiles_dirty_flags > 0 {
        make_tiles(state, 0, memory);
        make_tiles(state, 1, memory);
        state.tiles_dirty_flags -= 1;
        memory.tiles_dirty_flags.fetch_sub(1, Ordering::Relaxed);
    }

    if state.background_dirty_flags > 1 {
        make_background(state, memory);
        make_window(state, memory);
        state.background_dirty_flags -= 1;
        memory.background_dirty_flags.fetch_sub(1, Ordering::Relaxed);
    }
}

// Drawing to screen.
fn draw_background(state: &mut GpuState, canvas: &mut Canvas<Window>) {

    let mut point_idx: u16 = 0;

    // Index offset for the points array in case the current line is not 0.
    point_idx += 256 * state.line as u16;

    // Draw a whole line from the background map.
    for point in 0..256 {
        
        let target_x = (point as u8).overflowing_sub(state.scroll_x).0;
        let target_y = state.line.overflowing_sub(state.scroll_y).0;
        let color = state.tile_palette[state.background_points[point_idx as usize] as usize];
        let final_point = Point::new(target_x as i32, target_y as i32);

        canvas.set_draw_color(color);
        canvas.draw_point(final_point).unwrap();
        point_idx += 1;
    }
}

fn draw_window(state: &mut GpuState, canvas: &mut Canvas<Window>) {

    if state.window_x < 166 && state.window_y < 143 {
        let mut point_idx: u16 = 0;

        // Index offset for the points array in case the current line is not 0.
        point_idx += 256 * state.line as u16;

        // Draw a whole line from the window.
        for point in 0..255 as u8 {
        
            let target_x = point.wrapping_add(state.window_x.wrapping_sub(7));
            let target_y = state.line.wrapping_add(state.window_y);
            let color = state.tile_palette[state.window_points[point_idx as usize] as usize];
            let final_point = Point::new(target_x as i32, target_y as i32);

            canvas.set_draw_color(color);
            canvas.draw_point(final_point).unwrap();
            point_idx += 1;
        }
    }
}

fn draw_sprites(state: &mut GpuState, canvas: &mut Canvas<Window>) {

    for sprite in state.sprites.iter() {

        let target_x = sprite.x.wrapping_sub(8) as i32;
        let target_y = sprite.y.wrapping_sub(16) as i32;
        let y_size = if state.big_sprites {16} else {8};
        canvas.copy_ex(&sprite.data, None, Rect::new(target_x, target_y, 8, y_size), 0.0, None, sprite.flip_x, sprite.flip_y).unwrap();
    }
}

// Tile, Sprites, and Background cache generation.

fn make_tiles(state: &mut GpuState, target_bank: u8, memory: &Arc<GeneralMemory>) {

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

fn make_sprites(state: &mut GpuState, creator: &TextureCreator<WindowContext>, memory: &Arc<GeneralMemory>) {

    let mut current_address = 0xFE00;
    let mut generated_sprites: usize = 0;
    let mut sprites_idx = 0;
    let mut sprites: Vec<SpriteData> = Vec::new();

    while generated_sprites < 40 {

        let mut sprite_bytes: Vec<u8> = vec![0; 4];
        let mut loaded_bytes: usize = 0;

        while loaded_bytes < 4 {
            sprite_bytes[loaded_bytes] = memory::gpu_read(current_address, memory);
            current_address += 1;
            loaded_bytes += 1;
        }

        // Ignore the sprite if it's outside of the screen.
        if sprite_bytes[0] > 8 && sprite_bytes[1] > 0 {
            let new_tile = make_sprite(state, creator, &sprite_bytes);
            sprites.insert(sprites_idx, new_tile);
            sprites_idx += 1;
        }

        generated_sprites += 1;
    }

    state.sprites = sprites;
}

fn make_sprite(state: &mut GpuState, creator: &TextureCreator<WindowContext>, bytes: &Vec<u8>) -> SpriteData {

    let position_x = bytes[1];
    let position_y = bytes[0];
    let tile_id = bytes[2];
    let _priority = utils::check_bit(bytes[3], 7);
    let flip_y = utils::check_bit(bytes[3], 6);
    let flip_x = utils::check_bit(bytes[3], 5);
    let palette_id = if utils::check_bit(bytes[3], 4) {1} else {0};
    let y_size = if state.big_sprites {16} else {8};

    let mut new_sprite: Texture = creator.create_texture_streaming(PixelFormatEnum::RGBA32, 8, y_size).unwrap();
    new_sprite.set_blend_mode(sdl2::render::BlendMode::Blend);

    if y_size == 16 {

        let mut tile = tile_id & 0xFE;
        let mut color_idx: usize = 0;
        let mut tile_data = &state.tile_bank0[tile as usize];
        let mut sprite_colors: Vec<Color> = vec![Color::RGB(255, 255, 255); 128];

        for color in tile_data.iter() {

            // Get the color from the palette used by the sprite.
            let sprite_color = state.sprites_palettes[palette_id][*color as usize];
            sprite_colors[color_idx] = sprite_color;
            color_idx += 1;
        }

        tile = tile_id | 0x01;
        tile_data = &state.tile_bank0[tile as usize];

        for color in tile_data.iter() {

            // Get the color from the palette used by the sprite.
            let sprite_color = state.sprites_palettes[palette_id][*color as usize];
            sprite_colors[color_idx] = sprite_color;
            color_idx += 1;
        }

        color_idx = 0;

        new_sprite.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..16 {
                for x in 0..8 {
                    let offset = y*pitch + x*4;
                    // Set each color channel for the sprite texture from the palette.
                    buffer[offset] = sprite_colors[color_idx].r;
                    buffer[offset + 1] = sprite_colors[color_idx].g;
                    buffer[offset + 2] = sprite_colors[color_idx].b;
                    buffer[offset + 3] = sprite_colors[color_idx].a;
                    color_idx += 1;
                }
            }
        }).unwrap();
    }
    else {
        
        let mut color_idx: usize = 0;
        let tile_data = &state.tile_bank0[tile_id as usize];
        let mut sprite_colors: Vec<Color> = vec![Color::RGB(255, 255, 255); 64];

        for color in tile_data.iter() {

            // Get the color from the palette used by the sprite.
            let sprite_color = state.sprites_palettes[palette_id][*color as usize];
            sprite_colors[color_idx] = sprite_color;
            color_idx += 1;
        }

        color_idx = 0;

        new_sprite.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..8 {
                for x in 0..8 {
                    let offset = y*pitch + x*4;
                    // Set each color channel for the sprite texture from the palette.
                    buffer[offset] = sprite_colors[color_idx].r;
                    buffer[offset + 1] = sprite_colors[color_idx].g;
                    buffer[offset + 2] = sprite_colors[color_idx].b;
                    buffer[offset + 3] = sprite_colors[color_idx].a;
                    color_idx += 1;
                }
            }
        }).unwrap();
    }

    SpriteData::new((position_x, position_y), (flip_x, flip_y), new_sprite)
}

fn make_window(state: &mut GpuState, memory: &Arc<GeneralMemory>) {

    let mut generated_lines: u16 = 0;
    let mut current_address = state.window_tilemap.0;

    let lcdc_value =  utils::check_bit(memory::gpu_read(0xFF40, memory), 4);
    let tile_bank = if lcdc_value {&state.tile_bank0} else {&state.tile_bank1};

    let mut window_index: usize = 0;
        
    while generated_lines < 256 {

        let mut tiles: Vec<&Vec<u8>> = Vec::new();
        let mut tile_idx: usize = 0;

        // Loads tile indexes from memory, then gets the tile from GPU State and saves it to tiles.
        // 32 tiles is the maximum amount of tiles per line.
        while tile_idx < 32 {

            let tile_id = memory::gpu_read(current_address, memory);
            if lcdc_value {
                let target_tile = tile_id;
                tiles.insert(tile_idx, &tile_bank[target_tile as usize]);
                tile_idx += 1;
                current_address += 1;
            }
            else {
                let target_tile = (tile_id as i8 as i16 + 128) as u16;
                tiles.insert(tile_idx, &tile_bank[target_tile as usize]);
                tile_idx += 1;
                current_address += 1;
            }
        }

        let mut tile_line = 0;

        while tile_line < 8 {

            let line = make_background_line(&tiles, tile_line);
            for point in line.into_iter() {
                state.window_points[window_index] = point;
                window_index += 1;
            }
            tile_line += 1;
            generated_lines += 1;
        }
    }
}

fn make_background(state: &mut GpuState, memory: &Arc<GeneralMemory>) {

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

fn make_palette(value: u8) -> Vec<Color> {

    let mut result = vec![Color::RGB(255, 255, 255), Color::RGB(192, 192, 192), Color::RGB(96, 96, 96), Color::RGB(0, 0, 0)];
    let color_0 = value & 3;
    let color_1 = (value & 0x0C) >> 2;
    let color_2 = (value & 0x30) >> 4;
    let color_3 = (value & 0xC0) >> 6;

    match color_0 {
        0 => result[0] = Color::RGBA(255, 255, 255, 0),
        1 => result[0] = Color::RGBA(192, 192, 192, 255),
        2 => result[0] = Color::RGBA(96, 96, 96, 255),
        3 => result[0] = Color::RGBA(0, 0, 0, 255),
        _ => result[0] = Color::RGBA(0, 0, 0, 255),
    };

    match color_1 {
        0 => result[1] = Color::RGBA(255, 255, 255, 0),
        1 => result[1] = Color::RGBA(192, 192, 192, 255),
        2 => result[1] = Color::RGBA(96, 96, 96, 255),
        3 => result[1] = Color::RGBA(0, 0, 0, 255),
        _ => result[0] = Color::RGBA(0, 0, 0, 255),
    };

    match color_2 {
        0 => result[2] = Color::RGBA(255, 255, 255, 0),
        1 => result[2] = Color::RGBA(192, 192, 192, 255),
        2 => result[2] = Color::RGBA(96, 96, 96, 255),
        3 => result[2] = Color::RGBA(0, 0, 0, 255),
        _ => result[0] = Color::RGBA(0, 0, 0, 255),
    };

    match color_3 {
        0 => result[3] = Color::RGBA(255, 255, 255, 0),
        1 => result[3] = Color::RGBA(192, 192, 192, 255),
        2 => result[3] = Color::RGBA(96, 96, 96, 255),
        3 => result[3] = Color::RGBA(0, 0, 0, 255),
        _ => result[0] = Color::RGBA(0, 0, 0, 255),
    };

    result
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