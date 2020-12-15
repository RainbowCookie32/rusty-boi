use std::sync::Arc;
use std::time::{Duration, Instant};

use log::info;

use sdl2::rect::Point;
use sdl2::event::Event;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::keyboard::Scancode;

use super::memory::Memory;

const LCD_CONTROL: u16 = 0xFF40;
const LCD_STATUS: u16 = 0xFF41;
const SCROLL_Y: u16 = 0xFF42;
const SCROLL_X: u16 = 0xFF43;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF45;
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B;

pub enum VideoMode {
    Vblank,
    Hblank,
    OamSearch,
    LcdTransfer,
}

enum VideoInterrupt {
    ModeSwitch,
    LycCoincidence,
}

pub struct ColorPalette {
    value: u8,
    palette: Vec<Color>,
    base_palette: Vec<Color>,
}

impl ColorPalette {
    pub fn default() -> ColorPalette {
        ColorPalette {
            value: 0,
            palette: vec![Color::RGBA(255, 255, 255, 0), Color::RGBA(192, 192, 192, 255), Color::RGBA(96, 96, 96, 255),
            Color::RGBA(0, 0, 0, 255)],
            base_palette: vec![Color::RGBA(255, 255, 255, 0), Color::RGBA(192, 192, 192, 255), Color::RGBA(96, 96, 96, 255),
            Color::RGBA(0, 0, 0, 255)],
        }
    }

    pub fn update(&mut self, new_value: u8) {
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

    pub fn get_color(&self, index: u8) -> Color {
        self.palette[index as usize]
    }
}

pub struct VideoChip {
    mode: VideoMode,
    memory: Arc<Memory>,

    current_line: u8,
    rendered_frames: u32,
    display_enabled: bool,

    bg_enabled: bool,
    window_enabled: bool,
    sprites_enabled: bool,

    tiles_signed: Vec<Vec<u8>>,
    tiles_unsigned: Vec<Vec<u8>>,
    
    tiles_signed_hash: u64,
    tiles_unsigned_hash: u64,
    tiles_signed_dirty: bool,
    tiles_unsigned_dirty: bool,
    
    sprite_palette_0: ColorPalette,
    sprite_palette_1: ColorPalette,
    background_palette: ColorPalette,

    event_pump: sdl2::EventPump,
    window_canvas: Canvas<Window>,
}

impl VideoChip {
    pub fn new(memory: Arc<Memory>) -> VideoChip {

        let sdl_context = sdl2::init().unwrap();
        let sdl_video = sdl_context.video().unwrap();

        let game_window = sdl_video.window("Rusty Boi - Game - FPS: 0", 160 * 4, 144 * 4).position_centered().build().unwrap();
        let mut game_canvas = game_window.into_canvas().present_vsync().build().unwrap();

        let pump = sdl_context.event_pump().unwrap();

        game_canvas.set_scale(4.0, 4.0).unwrap();
        game_canvas.set_draw_color(Color::RGB(255, 255, 255));
        game_canvas.clear();
        game_canvas.present();

        VideoChip {
            mode: VideoMode::Hblank,
            memory: memory,

            current_line: 0,
            rendered_frames: 0,
            display_enabled: false,

            bg_enabled: false,
            window_enabled: false,
            sprites_enabled: false,

            tiles_signed: vec![vec![0; 64]; 256],
            tiles_unsigned: vec![vec![0; 64]; 256],
            
            tiles_signed_hash: 0,
            tiles_unsigned_hash: 0,
            tiles_signed_dirty: false,
            tiles_unsigned_dirty: false,

            sprite_palette_0: ColorPalette::default(),
            sprite_palette_1: ColorPalette::default(),
            background_palette: ColorPalette::default(),

            event_pump: pump,
            window_canvas: game_canvas,
        }
    }

    pub fn execution_loop(&mut self) {
        let mut frames_timer = Instant::now();

        loop {
            self.update_video_values();

            if self.display_enabled {

                if self.tiles_signed_dirty {
                    self.cache_signed();
                }
                if self.tiles_unsigned_dirty {
                    self.cache_unsigned();
                }

                match self.mode {
                    VideoMode::Hblank => {
                        self.hblank_mode();
                    },
                    VideoMode::Vblank => {
                        self.vblank_mode();
                    },
                    VideoMode::OamSearch => {
                        self.oam_search_mode();
                    },
                    VideoMode::LcdTransfer => {
                        self.lcd_transfer_mode();
                    }
                }

                let lyc = self.memory.video_read(LYC);
                if lyc == self.current_line {
                    let stat = self.memory.video_read(LCD_STATUS) | 4;
                    self.memory.video_write(LCD_STATUS, stat);
                    self.request_video_interrupt(VideoInterrupt::LycCoincidence);
                }
            }

            if frames_timer.elapsed() >= Duration::from_secs(1) {
                let framerate = format!("Rusty Boi - Game - FPS: {:#?}", self.rendered_frames as f64 / frames_timer.elapsed().as_secs() as f64);
                self.window_canvas.window_mut().set_title(&framerate).unwrap();
                frames_timer = Instant::now();
                self.rendered_frames = 0;
            }

            if self.handle_sdl_events() {
                break;
            }
        }
    }

    fn update_video_values(&mut self) {
        let lcdc = self.memory.video_read(LCD_CONTROL);
        let bg_status = (lcdc & 1) != 0;

        self.display_enabled = (lcdc & 0x80) != 0;
        // For the window to be displayed, it needs both its bit and the background one to be enabled.
        self.window_enabled = ((lcdc & 0x20) != 0) && bg_status;
        self.sprites_enabled = (lcdc & 0x02) != 0;
        self.bg_enabled = bg_status;

        self.current_line = self.memory.video_read(LY);

        let signed_hash = self.memory.get_signed_hash();
        let unsigned_hash = self.memory.get_unsigned_hash();

        if self.tiles_signed_hash != signed_hash {
            self.tiles_signed_dirty = true;
            self.tiles_signed_hash = signed_hash;
        }

        if self.tiles_unsigned_hash != unsigned_hash {
            self.tiles_unsigned_dirty = true;
            self.tiles_unsigned_hash = unsigned_hash;
        }

        self.background_palette.update(self.memory.video_read(0xFF47));
        self.sprite_palette_0.update(self.memory.video_read(0xFF48));
        self.sprite_palette_1.update(self.memory.video_read(0xFF49));
    }
    
    fn hblank_mode(&mut self) {
        if self.current_line >= 144 {
            self.mode = VideoMode::Vblank;
            self.update_video_mode();
            self.request_video_interrupt(VideoInterrupt::ModeSwitch);
            self.window_canvas.present();
            return;
        }

        self.mode = VideoMode::Hblank;
        self.update_video_mode();
        self.request_video_interrupt(VideoInterrupt::ModeSwitch);

        if self.bg_enabled {
            self.draw_background_line();
        }

        if self.window_enabled {
            self.draw_window_line();
        }

        self.current_line += 1;
        self.memory.video_write(LY, self.current_line);
    }

    fn vblank_mode(&mut self) {
        self.mode = VideoMode::Vblank;
        self.update_video_mode();
        self.request_video_interrupt(VideoInterrupt::ModeSwitch);

        self.current_line += 1;

        if self.current_line == 154 {
            self.current_line = 0;
            self.mode = VideoMode::OamSearch;
            self.update_video_mode();
            self.request_video_interrupt(VideoInterrupt::ModeSwitch);
            self.window_canvas.clear();
            self.rendered_frames += 1;
        }
        
        self.memory.video_write(LY, self.current_line);
    }

    fn oam_search_mode(&mut self) {
        self.mode = VideoMode::OamSearch;
        self.update_video_mode();
        self.request_video_interrupt(VideoInterrupt::ModeSwitch);

        self.mode = VideoMode::LcdTransfer;
    }

    fn lcd_transfer_mode(&mut self) {
        self.mode = VideoMode::LcdTransfer;
        self.update_video_mode();

        self.mode = VideoMode::Hblank;
    }

    fn draw_background_line(&mut self) {
        let lcd_control = self.memory.video_read(LCD_CONTROL);
        let use_signed_tiles = (lcd_control & 0x10) == 0;
        let background_address = (if (lcd_control & 0x08) == 0 {0x9800} else {0x9C00}) + (32 * (self.current_line / 8) as u16);

        let tile_y_offset = self.current_line % 8;

        let mut drawn_tiles = 0;
        let mut color_idx: u8 = 0;

        let target_y = self.current_line.wrapping_sub(self.memory.video_read(SCROLL_Y));

        // One draw pass for each color, avoids moving values around too frequently and the draw color switches.
        while color_idx < 4 {
            let mut target_x = self.memory.video_read(SCROLL_X);

            let color = self.background_palette.get_color(color_idx);
            self.window_canvas.set_draw_color(color);

            while drawn_tiles < 32 {
                let tile: &Vec<u8>;
                let tile_idx = self.memory.video_read(background_address + drawn_tiles);
                let mut draw_idx = 8 * tile_y_offset;
                let mut drawn_pixels = 0;

                if use_signed_tiles {
                    tile = &self.tiles_signed[(tile_idx  as i8 as i16 + 128) as usize];
                }
                else {
                    tile = &self.tiles_unsigned[tile_idx as usize];
                }
                
                while drawn_pixels < 8 {
                    if tile[draw_idx as usize] == color_idx {
                        self.window_canvas.draw_point(Point::new(target_x as i32, target_y as i32)).unwrap();
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

    fn draw_window_line(&mut self) {
        let lcd_control = self.memory.video_read(LCD_CONTROL);
        let use_signed_tiles = (lcd_control & 0x10) == 0;
        let background_address = (if (lcd_control & 0x40) == 0 {0x9800} else {0x9C00}) + (32 * (self.current_line / 8) as u16);

        let tile_y_offset = self.current_line % 8;

        let mut drawn_tiles = 0;
        let mut color_idx: u8 = 0;

        let target_y = self.current_line.wrapping_sub(self.memory.video_read(WY));

        // One draw pass for each color, avoids moving values around too frequently and the draw color switches.
        while color_idx < 4 {
            let mut target_x = self.memory.video_read(WX).wrapping_sub(7);

            let color = self.background_palette.get_color(color_idx);
            self.window_canvas.set_draw_color(color);

            while drawn_tiles < 32 {
                let tile: &Vec<u8>;
                let tile_idx = self.memory.video_read(background_address + drawn_tiles);
                let mut draw_idx = 8 * tile_y_offset;
                let mut drawn_pixels = 0;

                if use_signed_tiles {
                    tile = &self.tiles_signed[(tile_idx  as i8 as i16 + 128) as usize];
                }
                else {
                    tile = &self.tiles_unsigned[tile_idx as usize];
                }
                
                while drawn_pixels < 8 {
                    if tile[draw_idx as usize] == color_idx {
                        self.window_canvas.draw_point(Point::new(target_x as i32, target_y as i32)).unwrap();
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

    fn cache_signed(&mut self) {
        let cache_time = Instant::now();
        let mut current_addr = 0x87FF;
        let mut data: Vec<u8> = Vec::with_capacity(3072);

        info!("Video: Cache for signed tile bank invalidated, regenerating...");

        while current_addr < 0x97FF {
            data.push(self.memory.video_read(current_addr));
            current_addr += 1;
        }

        self.tiles_signed = self.cache_tiles(data);
        self.tiles_signed_dirty = false;

        info!("Video: Cache for signed tiles re-built in {:#?}", cache_time.elapsed());
    }

    fn cache_unsigned(&mut self) {
        let cache_time = Instant::now();
        let mut current_addr = 0x8000;
        let mut data: Vec<u8> = Vec::with_capacity(3072);

        info!("Video: Cache for unsigned tile bank invalidated, regenerating...");

        while current_addr < 0x9000 {
            data.push(self.memory.video_read(current_addr));
            current_addr += 1;
        }

        self.tiles_unsigned = self.cache_tiles(data);
        self.tiles_unsigned_dirty = false;

        info!("Video: Cache for unsigned tiles re-built in {:#?}", cache_time.elapsed());
    }

    fn cache_tiles(&mut self, data: Vec<u8>) -> Vec<Vec<u8>> {
        let mut result: Vec<Vec<u8>> = Vec::with_capacity(256);

        let mut byte_idx = 0;
        let mut generated_tiles = 0;

        while generated_tiles < 256 {
            let mut processed_bytes = 0;
            let mut tile: Vec<u8> = Vec::with_capacity(64);

            while processed_bytes < 16 {
                let mut bit_offset = 0;
                let bytes = (data[byte_idx], data[byte_idx + 1]);

                while bit_offset < 8 {
                    let bit1 = (bytes.0 >> (7 - bit_offset)) & 1;
                    let bit2 = (bytes.1 >> (7 - bit_offset)) & 1;

                    bit_offset += 1;
                    tile.push((bit2 << 1) | (bit1));
                }

                byte_idx += 2;
                processed_bytes += 2;
            }

            generated_tiles += 1;
            result.push(tile);
        }

        result
    }

    fn update_video_mode(&mut self) {
        let mut stat = self.memory.video_read(LCD_STATUS);

        match self.mode {
            VideoMode::OamSearch => {
                stat |= 2;
            },
            VideoMode::Vblank => {
                stat |= 1;
            },
            VideoMode::Hblank => {
                stat &= 0xFC;
            },
            VideoMode::LcdTransfer => {
                stat |= 3;
            }
        }

        self.memory.video_write(LCD_STATUS, stat);
    }

    fn request_video_interrupt(&mut self, int_type: VideoInterrupt) {
        let stat = self.memory.video_read(LCD_STATUS);
        let mut if_value = self.memory.video_read(0xFF0F);

        match int_type {
            VideoInterrupt::LycCoincidence => {
                if (stat & 0x40) != 0 {
                    if_value |= 2;
                }
            },
            VideoInterrupt::ModeSwitch => {
                match self.mode {
                    VideoMode::OamSearch => {
                        if (stat & 0x20) != 0 {
                            if_value |= 2;
                        }
                    },
                    VideoMode::Vblank => {
                        if_value |= 1;
                        if (stat & 0x10) != 0 {
                            if_value |= 2;
                        }
                    },
                    VideoMode::Hblank => {
                        if (stat & 0x08) != 0 {
                            if_value |= 2;
                        }
                    },
                    _ => unreachable!(),
                }
            }
        }

        self.memory.video_write(0xFF0F, if_value);
    }

    fn handle_sdl_events(&mut self) -> bool {
        let input_reg = self.memory.video_read(0xFF00);
        let mut result = 0b1111;

        let targets_dpad = (input_reg & 0x10) == 0;
        let targets_buttons = (input_reg & 0x20) == 0;

        for event in self.event_pump.poll_event() {
            match event {
                Event::Quit{..} => {
                    return true;
                },
                _ => {}
            }
        }

        if (input_reg & 0x30) == 0 {
            return false;
        }

        if targets_dpad {
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Down) {
                result &= 0x07;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Up) {
                result &= 0x0B;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Left) {
                result &= 0x0D;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Right) {
                result &= 0x0E;
            }
        }
        else if targets_buttons {
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Return) {
                result &= 0x07;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::RShift) {
                result &= 0x0B;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::S) {
                result &= 0x0D;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::A) {
                result &= 0x0E;
            }
        }

        self.memory.video_write(0xFF00, result | 0xC0);
        false
    }
}