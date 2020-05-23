use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::sync::atomic::Ordering;

use log::error;

use sdl2;
use sdl2::EventPump;
use sdl2::rect::Rect;
use sdl2::rect::Point;
use sdl2::video::Window;
use sdl2::video::WindowContext;
use sdl2::render::Canvas;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use sdl2::render::Texture;
use sdl2::render::TextureCreator;

use super::memory::SharedMemory;
use super::emulator::InputEvent;


const LCD_CONTROL: u16 = 0xFF40;
const LCD_STATUS: u16 = 0xFF41;
const SCROLL_Y: u16 = 0xFF42;
const SCROLL_X: u16 = 0xFF43;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF45;
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B;

#[derive(PartialEq)]
pub enum VideoMode {
    Hblank,
    Vblank,
    OamSearch,
    LcdTransfer,
    LyCoincidence,
}

pub struct ColorPalette {
    value: u8,
    palette: Vec<Color>,
    base_palette: Vec<Color>,
}

impl ColorPalette {
    pub fn new() -> ColorPalette {
        ColorPalette {
            value: 0,
            palette: vec![Color::RGBA(255, 255, 255, 0), Color::RGBA(192, 192, 192, 255), Color::RGBA(96, 96, 96, 255),
                Color::RGBA(0, 0, 0, 255)],
            base_palette: vec![Color::RGBA(255, 255, 255, 0), Color::RGBA(192, 192, 192, 255), Color::RGBA(96, 96, 96, 255),
                Color::RGBA(0, 0, 0, 255)],
        }
    }

    pub fn update_palette(&mut self, new_value: u8) {
        if new_value == self.value {
            return;
        }

        let color_0 = self.base_palette[(new_value & 3) as usize];
        let color_1 = self.base_palette[((new_value >> 2) & 3) as usize];
        let color_2 = self.base_palette[((new_value >> 4) & 3) as usize];
        let color_3 = self.base_palette[(new_value >> 6) as usize];

        self.value = new_value;
        self.palette = vec![color_0, color_1, color_2, color_3];
    }

    pub fn get_color(&self, idx: u8) -> Color {
        self.palette[idx as usize]
    }
}

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

pub struct VideoChip {
    mode: VideoMode,
    current_cycles: u16,
    display_enabled: bool,

    tbank_0: Vec<Vec<u8>>,
    tbank_1: Vec<Vec<u8>>,
    sprites: Vec<SpriteData>,

    tile_palette: ColorPalette,
    sprite_palettes: Vec<ColorPalette>,

    oam_state: (u64, bool),
    t0_state: (u64, bool),
    t1_state: (u64, bool),

    frames: u16,
    pump: EventPump,
    sdl_canvas: Canvas<Window>,
    memory: Arc<SharedMemory>,
    input_tx: Sender<InputEvent>,
    creator: TextureCreator<WindowContext>,
}

impl VideoChip {
    pub fn new(memory: Arc<SharedMemory>, input_tx: Sender<InputEvent>) -> VideoChip {
        let sdl_context = sdl2::init().unwrap();
        let sdl_video = sdl_context.video().unwrap();
        let window = sdl_video.window("Rusty Boi - Game - FPS: 0.0", 160 * 4, 144 * 4).position_centered().opengl().resizable().build().unwrap();
        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        let creator = canvas.texture_creator();
        let pump = sdl_context.event_pump().unwrap();

        canvas.set_scale(4.0, 4.0).unwrap();
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();
        canvas.present();

        VideoChip {
            mode: VideoMode::Hblank,
            current_cycles: 0,
            display_enabled: false,

            tbank_0: vec![vec![0; 64]; 256],
            tbank_1: vec![vec![0; 64]; 256],
            sprites: Vec::new(),

            tile_palette: ColorPalette::new(),
            sprite_palettes: vec![ColorPalette::new(), ColorPalette::new()],

            oam_state: (0, false),
            t0_state: (0, false),
            t1_state: (0, false),

            frames: 0,
            pump: pump,
            sdl_canvas: canvas,
            memory: memory,
            input_tx: input_tx,
            creator: creator,
        }
    }

    pub fn execution_loop(&mut self) {
        let mut fps_timer = std::time::Instant::now();

        loop {
            self.update_input_events();
            self.update_video_values();

            self.current_cycles = self.current_cycles.wrapping_add(super::emulator::GLOBAL_CYCLE_COUNTER.load(Ordering::Relaxed));

            if self.display_enabled {
                self.tile_palette.update_palette(self.memory.read(0xFF47));
                self.sprite_palettes[0].update_palette(self.memory.read(0xFF48));
                self.sprite_palettes[1].update_palette(self.memory.read(0xFF49));

                if self.t0_state.1 {
                    self.make_tiles(0);
                    self.t0_state.1 = false;
                }
                if self.t1_state.1 {
                    self.make_tiles(1);
                    self.t1_state.1 = false;
                }
                if self.oam_state.1 {
                    self.make_sprites();
                    self.oam_state.1 = false;
                }

                if self.mode == VideoMode::Hblank && self.current_cycles >= 204 {
                    self.hblank_mode();
                }
                else if self.mode == VideoMode::Vblank && self.current_cycles >= 456 {
                    self.vblank_mode();
                }
                else if self.mode == VideoMode::OamSearch && self.current_cycles >= 80 {
                    self.oam_scan_mode();
                }
                else if self.mode == VideoMode::LcdTransfer && self.current_cycles >= 172 {
                    self.lcd_transfer_mode();
                }

                let ly = self.memory.read(LY);
                let lyc = self.memory.read(LYC);

                if ly == lyc {
                    self.update_video_mode(VideoMode::LyCoincidence);
                }
            }

            if fps_timer.elapsed() >= std::time::Duration::from_secs(1) {
                let new_title = format!("Rusty Boi - Game - FPS: {}", self.frames as f32 / fps_timer.elapsed().as_secs() as f32);
                self.sdl_canvas.window_mut().set_title(&new_title).unwrap();
                fps_timer = std::time::Instant::now();
                self.frames = 0;
            }
        }
    }

    fn update_video_values(&mut self) {
        let lcdc = self.memory.read(LCD_CONTROL);

        self.display_enabled = ((lcdc >> 7) & 1) != 0;

        let oam_hash = self.memory.get_oam_hash();
        let t0_hash = self.memory.get_t0_hash();
        let t1_hash = self.memory.get_t1_hash();

        if !self.oam_state.1 {
            self.oam_state.1 = self.oam_state.0 != oam_hash;
            self.oam_state.0 = oam_hash;
        }

        if !self.t0_state.1 {
            self.t0_state.1 = self.t0_state.0 != t0_hash;
            self.t0_state.0 = t0_hash;
        }

        if !self.t1_state.1 {
            self.t1_state.1 = self.t1_state.0 != t1_hash;
            self.t1_state.0 = t1_hash;
        }
    }

    fn update_video_mode(&mut self, new_mode: VideoMode) {
        let mut stat_value = self.memory.read(LCD_STATUS);
        let mut if_value = self.memory.read(0xFF0F);

        match new_mode {
            VideoMode::LcdTransfer => {
                stat_value &= 0xFE;
                stat_value |= 3;
            },
            VideoMode::Hblank => {
                stat_value &= 0xFE;
                if ((stat_value >> 3) & 1) != 0 {
                    if_value |= 2;
                }
            },
            VideoMode::Vblank => {
                stat_value &= 0xFE;
                stat_value |= 1;
                if ((stat_value >> 4) & 1) != 0 {
                    if_value |= 2;
                }
            },
            VideoMode::OamSearch => {
                stat_value &= 0xFE;
                stat_value |= 2;
                if ((stat_value >> 5) & 1) != 0 {
                    if_value |= 2;
                }
    
                if_value |= 1;
            },
            VideoMode::LyCoincidence => {
                stat_value |= 4;
                if ((stat_value >> 6) & 1) != 0 {
                    if_value |= 2;
                }
            }
        }

        self.memory.write(LCD_STATUS, stat_value, false);
        self.memory.write(0xFF0F, if_value, false);
    }

    fn hblank_mode(&mut self) {
        let lcdc = self.memory.read(LCD_CONTROL);
        let bg_enabled = (lcdc & 1) != 0;
        let window_enabled = bg_enabled && ((lcdc >> 5) & 1) != 0;

        if bg_enabled {
            self.draw_background();
        }

        if window_enabled {
            self.draw_window();
        }

        let ly_value = self.memory.read(LY) + 1;
        self.memory.write(LY, ly_value, false);

        self.current_cycles = 0;
        self.mode = VideoMode::Hblank;
        self.update_video_mode(VideoMode::Hblank);

        if ly_value == 144 {
            if ((lcdc >> 1) & 1) != 0 {
                self.draw_sprites();
            }
            self.mode = VideoMode::Vblank;
            self.frames += 1;
            self.sdl_canvas.present();
        }
    }

    fn vblank_mode(&mut self) {
        self.current_cycles = 0;
        let ly_value = self.memory.read(LY) + 1;
        self.memory.write(LY, ly_value, false);

        self.update_video_mode(VideoMode::Vblank);

        if ly_value == 154 {
            self.mode = VideoMode::OamSearch;
            self.update_video_mode(VideoMode::OamSearch);
            self.memory.write(LY, 0, false);

            self.sdl_canvas.set_draw_color(Color::WHITE);
            self.sdl_canvas.clear();
        }
    }

    fn oam_scan_mode(&mut self) {
        self.current_cycles = 0;
        self.mode = VideoMode::LcdTransfer;
        self.update_video_mode(VideoMode::LcdTransfer);
    }

    fn lcd_transfer_mode(&mut self) {
        self.current_cycles = 0;
        self.mode = VideoMode::Hblank;
        self.update_video_mode(VideoMode::Hblank);
    }

    // Drawing to screen.
    fn draw_background(&mut self) {
        let line = self.memory.read(LY);
        let lcd_control = self.memory.read(LCD_CONTROL);
        let use_signed_tiles = (lcd_control & 0x10) == 0;
        let background_address = (if (lcd_control & 0x08) == 0 {0x9800} else {0x9C00}) + (32 * (line / 8) as u16);

        let tile_y_offset = line % 8;

        let mut drawn_tiles = 0;
        let mut color_idx: u8 = 0;

        // One draw pass for each color, avoids moving values around too frequently and the draw color switches.
        while color_idx < 4 {
            let mut target_x: u8 = 0;
            let target_y = line.wrapping_sub(self.memory.read(SCROLL_Y));
            target_x = target_x.wrapping_sub(self.memory.read(SCROLL_X));

            let color = self.tile_palette.get_color(color_idx);
            self.sdl_canvas.set_draw_color(color);

            while drawn_tiles < 32 {
                let tile: &Vec<u8>;
                let tile_idx = self.memory.read(background_address + drawn_tiles);
                let mut draw_idx = 8 * tile_y_offset;
                let mut drawn_pixels = 0;

                if use_signed_tiles {
                    tile = &self.tbank_1[(tile_idx  as i8 as i16 + 128) as usize];
                }
                else {
                    tile = &self.tbank_0[tile_idx as usize];
                }
                
                while drawn_pixels < 8 {
                    if tile[draw_idx as usize] == color_idx {
                        self.sdl_canvas.draw_point(Point::new(target_x as i32, target_y as i32)).unwrap();
                    }

                    target_x = target_x.wrapping_add(1);
                    draw_idx += 1;
                    drawn_pixels += 1;
                }

                drawn_tiles += 1;
            }

            color_idx += 1;
            drawn_tiles = 0;
        }
    }

    fn draw_window(&mut self) {
        let line = self.memory.read(LY);
        let lcd_control = self.memory.read(LCD_CONTROL);
        let use_signed_tiles = (lcd_control & 0x10) == 0;
        let background_address = (if (lcd_control & 0x40) == 0 {0x9800} else {0x9C00}) + (32 * (line / 8) as u16);

        let tile_y_offset = line % 8;

        let mut drawn_tiles = 0;
        let mut color_idx: u8 = 0;

        let target_y = line.wrapping_sub(self.memory.read(WY));

        // One draw pass for each color, avoids moving values around too frequently and the draw color switches.
        while color_idx < 4 {
            let mut target_x = self.memory.read(WX).wrapping_sub(7);

            let color = self.tile_palette.get_color(color_idx);
            self.sdl_canvas.set_draw_color(color);

            while drawn_tiles < 32 {
                let tile: &Vec<u8>;
                let tile_idx = self.memory.read(background_address + drawn_tiles);
                let mut draw_idx = 8 * tile_y_offset;
                let mut drawn_pixels = 0;

                if use_signed_tiles {
                    tile = &self.tbank_1[(tile_idx  as i8 as i16 + 128) as usize];
                }
                else {
                    tile = &self.tbank_0[tile_idx as usize];
                }
                
                while drawn_pixels < 8 {
                    if tile[draw_idx as usize] == color_idx {
                        self.sdl_canvas.draw_point(Point::new(target_x as i32, target_y as i32)).unwrap();
                    }

                    target_x = target_x.wrapping_add(1);
                    draw_idx += 1;
                    drawn_pixels += 1;
                }

                drawn_tiles += 1;
            }

            color_idx += 1;
            drawn_tiles = 0;
        }
    }

    fn draw_sprites(&mut self) {
        let big_sprites = ((self.memory.read(LCD_CONTROL) >> 2) & 1) == 1;
        for sprite in self.sprites.iter() {
            let target_x = sprite.x.wrapping_sub(8) as i32;
            let target_y = sprite.y.wrapping_sub(16) as i32;
            let y_size = if big_sprites {16} else {8};
            self.sdl_canvas.copy_ex(&sprite.data, None, Rect::new(target_x, target_y, 8, y_size), 0.0, None, sprite.flip_x, sprite.flip_y).unwrap();
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
                self.tbank_0[tiles_position as usize] = VideoChip::make_tile(&tile_bytes);
            }
            else {
                self.tbank_1[tiles_position as usize] = VideoChip::make_tile(&tile_bytes);
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
                generated_tile[tile_index] = ((bytes_to_check.0 >> current_bit) & 1) | (((bytes_to_check.1 >> current_bit) & 1) << 1);
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
        let _priority = ((bytes[3] >> 7) & 1) != 0;
        let flip_y = ((bytes[3] >> 6) & 1) != 0;
        let flip_x = ((bytes[3] >> 5) & 1) != 0;
        let palette_id = if ((bytes[3] >> 4) & 1) != 0 {1} else {0};
        let y_size = if ((self.memory.read(LCD_CONTROL) >> 2) & 1) == 1 {16} else {8};
    
        let mut new_sprite: Texture = self.creator.create_texture_streaming(PixelFormatEnum::RGBA32, 8, y_size).unwrap();
        new_sprite.set_blend_mode(sdl2::render::BlendMode::Blend);
    
        if y_size == 16 {
            let mut tile = tile_id & 0xFE;
            let mut color_idx: usize = 0;
            let mut tile_data = &self.tbank_0[tile as usize];
            let mut sprite_colors: Vec<Color> = vec![Color::RGB(255, 255, 255); 128];
    
            for color in tile_data.iter() {
    
                // Get the color from the palette used by the sprite.
                let sprite_color = self.sprite_palettes[palette_id].get_color(*color);
                sprite_colors[color_idx] = sprite_color;
                color_idx += 1;
            }
    
            tile = tile_id | 0x01;
            tile_data = &self.tbank_0[tile as usize];
    
            for color in tile_data.iter() {
                // Get the color from the palette used by the sprite.
                let sprite_color = self.sprite_palettes[palette_id].get_color(*color);
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
            let tile_data = &self.tbank_0[tile_id as usize];
            let mut sprite_colors: Vec<Color> = vec![Color::RGB(255, 255, 255); 64];
    
            for color in tile_data.iter() {
                // Get the color from the palette used by the sprite.
                let sprite_color = self.sprite_palettes[palette_id].get_color(*color);
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

    fn update_input_events(&mut self) {
        for event in self.pump.poll_iter() {
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
                    let mut count = 5;
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