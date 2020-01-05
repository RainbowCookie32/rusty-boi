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

use super::memory::Memory;
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

pub struct Gpu {

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

    pub tiles_dirty_flags: u8,
    pub sprites_dirty_flags: u8,
    pub background_dirty_flags: u8,

    pub frames: u16,
    pub total_cycles: Arc<AtomicU16>,

    pub memory: Arc<Memory>,

    pub event_pump: sdl2::EventPump,
    pub input_tx: Sender<InputEvent>,

    pub game_canvas: Canvas<Window>,
    pub texture_creator: TextureCreator<WindowContext>,
}

impl Gpu {
    pub fn new(cycles: Arc<AtomicU16>, mem: Arc<Memory>, tx: Sender<InputEvent>) -> Gpu {

        let sdl_ctx = sdl2::init().unwrap();
        let sdl_video = sdl_ctx.video().unwrap();

        let game_window = sdl_video.window("Rusty Boi - Game - FPS: 0", 160 * 4, 144 * 4).position_centered().build().unwrap();
        let mut game_canvas = game_window.into_canvas().present_vsync().build().unwrap();
        let texture_creator = game_canvas.texture_creator();

        game_canvas.set_scale(4.0, 4.0).unwrap();
        game_canvas.set_draw_color(Color::RGB(255, 255, 255));
        game_canvas.clear();
        game_canvas.present();
        
        Gpu {
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

            tiles_dirty_flags: 0,
            sprites_dirty_flags: 0,
            background_dirty_flags: 0,

            frames: 0,
            total_cycles: cycles,

            memory: mem,
            
            event_pump: sdl_ctx.event_pump().unwrap(),
            input_tx: tx,

            game_canvas: game_canvas,
            texture_creator: texture_creator,
        }
    }

    pub fn execution_loop(&mut self) {

        let mut fps_timer = std::time::Instant::now();
        
        loop {
            self.update_inputs();
            self.update_gpu_values();

            if self.lcd_enabled {

                self.tile_palette = self.make_palette(self.memory.read(0xFF47));
                self.sprites_palettes[0] = self.make_palette(self.memory.read(0xFF48));
                self.sprites_palettes[1] = self.make_palette(self.memory.read(0xFF49));

                if self.gpu_mode == 0 && self.gpu_cycles >= 204 {
                    self.hblank_mode();
                }
                else if self.gpu_mode == 1 && self.gpu_cycles >= 456 {
                    self.vblank_mode();
                }
                else if self.gpu_mode == 2 && self.gpu_cycles >= 80 {
                    self.oam_scan_mode();
                }
                else if self.gpu_mode == 3 && self.gpu_cycles >= 172 {
                    self.lcd_transfer_mode();
                }

                let lyc_value = self.memory.read(0xFF45);

                if lyc_value == self.memory.read(0xFF44) {
                    let stat_value = self.memory.read(0xFF41) | 2;
                    let mut if_value = self.memory.read(0xFF0F);

                    self.memory.write(0xFF41, stat_value, false);

                    if ((stat_value >> 6) & 1) == 1 {
                        if_value |= 2;
                        self.memory.write(0xFF0F, if_value, false);
                    }
                }
            }

            if fps_timer.elapsed() >= std::time::Duration::from_millis(1000) && self.frames > 0 {
                let framerate = format!("Rusty Boi - Game - FPS: {:#?}", self.frames as u64 / fps_timer.elapsed().as_secs());
                self.game_canvas.window_mut().set_title(&framerate).unwrap();
                fps_timer = std::time::Instant::now();
                self.frames = 0;
            }
        }
    }

    fn hblank_mode(&mut self) {

        let stat_value = self.memory.read(0xFF41);
        self.memory.write(0xFF41, stat_value & 0xFC, false);

        if self.background_enabled {self.draw_background()}
        if self.sprites_enabled {self.draw_sprites()}
        if self.window_enabled {self.draw_window()}

        self.gpu_cycles = 0;
        self.line += 1;
        self.memory.write(0xFF44, self.line, false);

        if self.line == 144 {
            self.gpu_mode = 1;
            self.frames += 1;
            self.game_canvas.present();
        }

        if ((stat_value >> 3) & 1) == 1 {
            let if_value = self.memory.read(0xFF0F);
            self.memory.write(0xFF0F, if_value | 2, false);
        }
    }

    fn vblank_mode(&mut self) {
        let if_value = self.memory.read(0xFF0F) | 1;
        let stat_value = (self.memory.read(0xFF41) & 0xFD) | 1;

        self.gpu_cycles = 0;
        self.line += 1;

        self.memory.write(0xFF41, stat_value, false);
        self.memory.write(0xFF44, self.line, false);
        self.memory.write(0xFF0F, if_value, false);

        if self.line == 154 {
            self.gpu_mode = 2;
            self.line = 0;

            self.game_canvas.clear();
            self.memory.write(0xFF44, 1, false);
        }
    }

    fn oam_scan_mode(&mut self) {
        
        let stat_value = self.memory.read(0xFF41);

        self.gpu_cycles = 0;
        self.gpu_mode = 3;

        self.memory.write(0xFF41, (stat_value | 2) & 0xFE, false);

        if self.sprites_dirty_flags > 0 {
            self.make_sprites();
            self.sprites_dirty_flags -= 1;
            self.memory.sprites_dirty_flags.fetch_sub(1, Ordering::Relaxed);
        }

        if ((stat_value >> 5) & 1) == 1 {
            let if_value = self.memory.read(0xFF0F) | 2;
            self.memory.write(0xFF0F, if_value, false);
        }
    }

    fn lcd_transfer_mode(&mut self) {
        let stat_value = self.memory.read(0xFF41) | 3;

        self.memory.write(0xFF41, stat_value, false);
        
        self.gpu_cycles = 0;
        self.gpu_mode = 0;

        if self.tiles_dirty_flags > 0 {
            self.make_tiles(0);
            self.make_tiles(1);
            self.tiles_dirty_flags -= 1;
            self.memory.tiles_dirty_flags.fetch_sub(1, Ordering::Relaxed);
        }

        if self.background_dirty_flags > 0 {
            self.make_background();
            self.make_window();
            self.background_dirty_flags -= 1;
            self.memory.background_dirty_flags.fetch_sub(1, Ordering::Relaxed);
        }
    }

    fn draw_background(&mut self) {
        let mut point_idx: u16 = 0;

        // Index offset for the points array in case the current line is not 0.
        point_idx += 256 * self.line as u16;

        // Draw a whole line from the background map.
        for point in 0..256 {
        
            let target_x = (point as u8).overflowing_sub(self.scroll_x).0;
            let target_y = self.line.overflowing_sub(self.scroll_y).0;
            let color = self.tile_palette[self.background_points[point_idx as usize] as usize];
            let final_point = Point::new(target_x as i32, target_y as i32);

            self.game_canvas.set_draw_color(color);
            self.game_canvas.draw_point(final_point).unwrap();
            point_idx += 1;
        }
    }

    fn draw_window(&mut self) {
        if self.window_x < 166 && self.window_y < 143 {
            let mut point_idx: u16 = 0;
    
            // Index offset for the points array in case the current line is not 0.
            point_idx += 256 * self.line as u16;
    
            // Draw a whole line from the window.
            for point in 0..255 as u8 {
            
                let target_x = point.wrapping_add(self.window_x.wrapping_sub(7));
                let target_y = self.line.wrapping_add(self.window_y);
                let color = self.tile_palette[self.window_points[point_idx as usize] as usize];
                let final_point = Point::new(target_x as i32, target_y as i32);
    
                self.game_canvas.set_draw_color(color);
                self.game_canvas.draw_point(final_point).unwrap();
                point_idx += 1;
            }
        }
    }

    fn draw_sprites(&mut self) {
        for sprite in self.sprites.iter() {

            let target_x = sprite.x.wrapping_sub(8) as i32;
            let target_y = sprite.y.wrapping_sub(16) as i32;
            let y_size = if self.big_sprites {16} else {8};
            self.game_canvas.copy_ex(&sprite.data, None, Rect::new(target_x, target_y, 8, y_size), 0.0, None, sprite.flip_x, sprite.flip_y).unwrap();
        }
    }

    fn make_tiles(&mut self, target_bank: u8) {
        let start_position = if target_bank == 0 {0x8000} else {0x8800};
        let end_position = if target_bank == 0 {0x8FFF} else {0x97FF};
        let mut memory_position = start_position;
        let mut tiles_position = 0;

        while memory_position < end_position {

            let mut loaded_bytes = 0;
            let mut tile_bytes: Vec<u8> = vec![0; 16];

            while loaded_bytes < 16 {

                tile_bytes[loaded_bytes] = self.memory.read(memory_position);
                memory_position += 1;
                loaded_bytes += 1;
            }

            if target_bank == 0 {
                self.tile_bank0[tiles_position as usize] = self.make_tile(&tile_bytes);
            }
            else {
                self.tile_bank1[tiles_position as usize] = self.make_tile(&tile_bytes);
            }

            tiles_position += 1;
        }
    }

    fn make_tile(&mut self, bytes: &Vec<u8>) -> Vec<u8> {
        let mut tile_index = 0;
        let mut processed_bytes = 0;
        let mut generated_tile: Vec<u8> = vec![0; 64];
    
        while processed_bytes < 16 {
    
            let mut current_bit = 8;
            let bytes_to_check = (bytes[processed_bytes], bytes[processed_bytes + 1]);
            processed_bytes += 2;
    
            while current_bit != 0 {
    
                current_bit -= 1;
                let bits = (((bytes_to_check.0 >> current_bit) & 1), ((bytes_to_check.1 >> current_bit) & 1));
                let color = (bits.0 << 1) | (bits.1);
                generated_tile[tile_index] = color;
                tile_index += 1;
            }
        }
    
        generated_tile
    }

    fn make_sprites(&mut self) {
        let mut current_address = 0xFE00;
        let mut generated_sprites: usize = 0;
        let mut sprites_idx = 0;
        let mut sprites: Vec<SpriteData> = Vec::new();
    
        while generated_sprites < 40 {
    
            let mut sprite_bytes: Vec<u8> = vec![0; 4];
            let mut loaded_bytes: usize = 0;
    
            while loaded_bytes < 4 {
                sprite_bytes[loaded_bytes] = self.memory.read(current_address);
                current_address += 1;
                loaded_bytes += 1;
            }
    
            // Ignore the sprite if it's outside of the screen.
            if sprite_bytes[0] > 8 && sprite_bytes[1] > 0 {
                let new_tile = self.make_sprite(&sprite_bytes);
                sprites.insert(sprites_idx, new_tile);
                sprites_idx += 1;
            }
    
            generated_sprites += 1;
        }
    
        self.sprites = sprites;
    }

    fn make_sprite(&mut self, bytes: &Vec<u8>) -> SpriteData {
        let position_x = bytes[1];
        let position_y = bytes[0];
        let tile_id = bytes[2];
        let _priority = ((bytes[3] >> 7) & 1) == 1;
        let flip_y = ((bytes[3] >> 6) & 1) == 1;
        let flip_x = ((bytes[3] >> 5) & 1) == 1;
        let palette_id = if ((bytes[3] >> 4) & 1) == 1 {1} else {0};
        let y_size = if self.big_sprites {16} else {8};
    
        let mut new_sprite: Texture = self.texture_creator.create_texture_streaming(PixelFormatEnum::RGBA32, 8, y_size).unwrap();
        new_sprite.set_blend_mode(sdl2::render::BlendMode::Blend);
    
        if y_size == 16 {
    
            let mut tile = tile_id & 0xFE;
            let mut color_idx: usize = 0;
            let mut tile_data = &self.tile_bank0[tile as usize];
            let mut sprite_colors: Vec<Color> = vec![Color::RGB(255, 255, 255); 128];
    
            for color in tile_data.iter() {
    
                // Get the color from the palette used by the sprite.
                let sprite_color = self.sprites_palettes[palette_id][*color as usize];
                sprite_colors[color_idx] = sprite_color;
                color_idx += 1;
            }
    
            tile = tile_id | 0x01;
            tile_data = &self.tile_bank0[tile as usize];
    
            for color in tile_data.iter() {
    
                // Get the color from the palette used by the sprite.
                let sprite_color = self.sprites_palettes[palette_id][*color as usize];
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
            let tile_data = &self.tile_bank0[tile_id as usize];
            let mut sprite_colors: Vec<Color> = vec![Color::RGB(255, 255, 255); 64];
    
            for color in tile_data.iter() {
    
                // Get the color from the palette used by the sprite.
                let sprite_color = self.sprites_palettes[palette_id][*color as usize];
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

    fn make_window(&mut self) {
        let mut generated_lines: u16 = 0;
        let mut current_address = self.window_tilemap.0;
    
        let lcdc_value =  ((self.memory.read(0xFF40) >> 4) & 1) == 1;
        let tile_bank = if lcdc_value {&self.tile_bank0} else {&self.tile_bank1};
    
        let mut window_index: usize = 0;
            
        while generated_lines < 256 {
    
            let mut tiles: Vec<&Vec<u8>> = Vec::new();
            let mut tile_idx: usize = 0;
    
            // Loads tile indexes from memory, then gets the tile from GPU State and saves it to tiles.
            // 32 tiles is the maximum amount of tiles per line.
            while tile_idx < 32 {
    
                let tile_id = self.memory.read(current_address);
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
    
                let line = Gpu::make_background_line(&tiles, tile_line);
                for point in line.into_iter() {
                    self.window_points[window_index] = point;
                    window_index += 1;
                }
                tile_line += 1;
                generated_lines += 1;
            }
        }
    
    }

    fn make_background(&mut self) {
        let mut generated_lines: u16 = 0;
        
        let mut current_background = if ((self.memory.read(0xFF40) >> 3) & 1) == 1 {0x9C00} else {0x9800};
    
        let lcdc_value =  ((self.memory.read(0xFF40) >> 4) & 1) == 1;
        let tile_bank = if lcdc_value {&self.tile_bank0} else {&self.tile_bank1};
    
        let mut background_idx: usize = 0;
            
        while generated_lines < 256 {
    
            let mut tiles: Vec<&Vec<u8>> = Vec::new();
            let mut tile_idx: usize = 0;
    
            // Loads tile indexes from memory, then gets the tile from GPU State and saves it to tiles.
            // 32 tiles is the maximum amount of tiles per line in the background.
            while tile_idx < 32 {
    
                let bg_value = self.memory.read(current_background);
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
    
                let line = Gpu::make_background_line(&tiles, tile_line);
                for point in line.into_iter() {
                    self.background_points[background_idx] = point;
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

    fn make_palette(&mut self, value: u8) -> Vec<Color> {
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
            _ => result[1] = Color::RGBA(0, 0, 0, 255),
        };
    
        match color_2 {
            0 => result[2] = Color::RGBA(255, 255, 255, 0),
            1 => result[2] = Color::RGBA(192, 192, 192, 255),
            2 => result[2] = Color::RGBA(96, 96, 96, 255),
            3 => result[2] = Color::RGBA(0, 0, 0, 255),
            _ => result[2] = Color::RGBA(0, 0, 0, 255),
        };
    
        match color_3 {
            0 => result[3] = Color::RGBA(255, 255, 255, 0),
            1 => result[3] = Color::RGBA(192, 192, 192, 255),
            2 => result[3] = Color::RGBA(96, 96, 96, 255),
            3 => result[3] = Color::RGBA(0, 0, 0, 255),
            _ => result[3] = Color::RGBA(0, 0, 0, 255),
        };
    
        result
    
    }

    fn update_gpu_values(&mut self) {
        let lcdc_value = self.memory.read(0xFF40);

        self.lcd_enabled = ((lcdc_value >> 7) & 1) == 1;

        self.window_tilemap = if ((lcdc_value >> 6) & 1) == 1 {(0x9C00, 0x9FFF)} else {(0x8800, 0x97FF)};
        self.window_enabled = ((lcdc_value >> 5) & 1) == 1;
        
        self.tiles_area = if ((lcdc_value >> 4) & 1) == 1 {(0x8000, 0x8FFF)} else {(0x8800, 0x97FF)};
        self.background_tilemap = if ((lcdc_value >> 3) & 1) == 1 {(0x9C00, 0x9FFF)} else {(0x9800, 0x9BFF)};

        self.big_sprites = ((lcdc_value >> 2) & 1) == 1;
        self.sprites_enabled = ((lcdc_value >> 1) & 1) == 1;

        self.background_enabled = (lcdc_value & 1) == 1;

        self.scroll_y = self.memory.read(0xFF42);
        self.scroll_x = self.memory.read(0xFF43);
        
        self.window_y = self.memory.read(0xFF4A);
        self.window_x = self.memory.read(0xFF4B);

        self.tiles_dirty_flags = self.memory.tiles_dirty_flags.load(Ordering::Relaxed);
        self.sprites_dirty_flags = self.memory.sprites_dirty_flags.load(Ordering::Relaxed);
        self.background_dirty_flags = self.memory.background_dirty_flags.load(Ordering::Relaxed);
        self.gpu_cycles = self.total_cycles.load(Ordering::Relaxed);
    }

    fn update_inputs(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit{..} => {
                    self.input_tx.send(InputEvent::Quit).unwrap();
                }
                Event::KeyDown{keycode: Some(Keycode::A), ..} => {
                    let mut count = 5;
                    while count > 0 {
                        let result = self.input_tx.send(InputEvent::APressed);
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
                        let result = self.input_tx.send(InputEvent::BPressed);
                        match result {
                            Ok(_) => {},
                            Err(error) => {error!("Input: Failed to send event to CPU, error {}", error); count = 0},
                        }
                        count -= 1;
                    }
                },
                Event::KeyDown{keycode: Some(Keycode::Return), ..} => {
                    let mut count = 15;
                    while count > 0 {
                        let result = self.input_tx.send(InputEvent::StartPressed);
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
                        let result = self.input_tx.send(InputEvent::SelectPressed);
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
                        let result = self.input_tx.send(InputEvent::UpPressed);
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
                        let result = self.input_tx.send(InputEvent::DownPressed);
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
                        let result = self.input_tx.send(InputEvent::LeftPressed);
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
                        let result = self.input_tx.send(InputEvent::RightPressed);
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
}