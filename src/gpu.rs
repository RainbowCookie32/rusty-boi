use std::sync::Arc;


use sdl2;

use sdl2::rect::Point;
use sdl2::pixels::Color;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;

use sdl2::video::Window;
use sdl2::render::Canvas;

use super::memory::Memory;


#[derive(Clone, Copy, PartialEq)]
enum InterruptType {
    Hblank = 3,
    Vblank = 4,
    Oam = 5,
    Lyc = 6,
}

enum GpuMode {
    Hblank = 0,
    Vblank = 1,
    Oam = 2,
    Lcd = 3,
}

pub struct Gpu {

    // Internal GPU values.
    gpu_mode: u8,
    line: u8,

    // Background scrolling values.
    scroll_x: u8,
    scroll_y: u8,

    // The position of the window in X and Y.
    window_x: u8,
    window_y: u8,

    // LCDC values.
    lcd_enabled: bool,
    window_tilemap: bool,
    window_enabled: bool,
    bg_win_tile_data: bool,
    bg_tilemap: bool,
    sprite_size: bool,
    sprites_enabled: bool,
    bg_enabled: bool,

    // Color paletttes.
    base_palette: Vec<Color>,
    tile_palette: Vec<Color>,
    sprites_palettes: Vec<Vec<Color>>,

    // Rendering related variables, sdl memes.
    frames: u16,
    game_canvas: Canvas<Window>,

    memory: Arc<Memory>,

    event_pump: sdl2::EventPump,
}

impl Gpu {
    pub fn new(mem: Arc<Memory>) -> Gpu {

        let sdl_ctx = sdl2::init().unwrap();
        let sdl_video = sdl_ctx.video().unwrap();

        let game_window = sdl_video.window("Rusty Boi - Game - FPS: 0", 160 * 4, 144 * 4).position_centered().build().unwrap();
        let mut game_canvas = game_window.into_canvas().present_vsync().build().unwrap();

        game_canvas.set_scale(4.0, 4.0).unwrap();
        game_canvas.set_draw_color(Color::RGB(255, 255, 255));
        game_canvas.clear();
        game_canvas.present();
        
        Gpu {
            gpu_mode: 0,
            line: 0,

            scroll_x: 0,
            scroll_y: 0,

            window_x: 0,
            window_y: 0,

            lcd_enabled: false,
            window_tilemap: false,
            window_enabled: false,
            bg_win_tile_data: false,
            bg_tilemap: false,
            sprite_size: false,
            sprites_enabled: false,
            bg_enabled: false,

            base_palette: vec![Color::RGBA(255, 255, 255, 0), Color::RGBA(192, 192, 192, 255), Color::RGBA(96, 96, 96, 255),
            Color::RGBA(0, 0, 0, 255)],
            tile_palette: vec![Color::RGBA(255, 255, 255, 0), Color::RGBA(192, 192, 192, 255), Color::RGBA(96, 96, 96, 255),
            Color::RGBA(0, 0, 0, 255)],
            sprites_palettes: vec![vec![Color::RGBA(255, 255, 255, 0), Color::RGBA(192, 192, 192, 255), Color::RGBA(96, 96, 96, 255), 
            Color::RGBA(0, 0, 0, 255)]; 2],

            frames: 0,
            memory: mem,
            event_pump: sdl_ctx.event_pump().unwrap(),
            game_canvas: game_canvas,
        }
    }

    pub fn execution_loop(&mut self) {
        let mut fps_timer = std::time::Instant::now();
        
        loop {
            if self.update_inputs() {
                break;
            }

            self.update_gpu_values();

            if self.lcd_enabled {
                if self.gpu_mode == 0 {
                    self.hblank_mode();
                }
                else if self.gpu_mode == 1 {
                    self.vblank_mode();
                }
                else if self.gpu_mode == 2 {
                    self.oam_scan_mode();
                }
                else if self.gpu_mode == 3 {
                    self.lcd_transfer_mode();
                }

                let lyc_value = self.memory.read(0xFF45);

                if lyc_value == self.memory.read(0xFF44) {
                    let stat_value = self.memory.read(0xFF41);
                    self.memory.write(0xFF41, stat_value | 4, false);
                    self.request_interrupt(InterruptType::Lyc);
                }
            }

            if fps_timer.elapsed() >= std::time::Duration::from_secs(1) {
                let framerate = format!("Rusty Boi - Game - FPS: {:#?}", self.frames as f64 / fps_timer.elapsed().as_secs() as f64);
                self.game_canvas.window_mut().set_title(&framerate).unwrap();
                fps_timer = std::time::Instant::now();
                self.frames = 0;
            }
        }
    }

    fn hblank_mode(&mut self) {

        self.set_gpu_mode(GpuMode::Hblank);

        if self.bg_enabled {self.draw_background()}
        if self.window_enabled {self.draw_window()}

        self.line += 1;
        self.memory.write(0xFF44, self.line, false);

        if self.line == 144 {
            self.gpu_mode = 1;
            self.frames += 1;
            self.game_canvas.present();
        }

        self.request_interrupt(InterruptType::Hblank);
    }

    fn vblank_mode(&mut self) {

        self.set_gpu_mode(GpuMode::Vblank);
        self.line += 1;
        self.memory.write(0xFF44, self.line, false);

        if self.line == 154 {
            self.gpu_mode = 2;
            self.line = 0;
            self.game_canvas.clear();
            self.memory.write(0xFF44, 1, false);
        }
        
        self.request_interrupt(InterruptType::Vblank);
    }

    fn oam_scan_mode(&mut self) {

        self.set_gpu_mode(GpuMode::Oam);
        self.gpu_mode = 3;
        
        self.request_interrupt(InterruptType::Oam);
    }

    fn lcd_transfer_mode(&mut self) {

        self.set_gpu_mode(GpuMode::Lcd);
        self.gpu_mode = 0;
    }

    fn draw_background(&mut self) {
        let tileset_addr = if self.bg_win_tile_data {0x8000} else {0x8800};
        let tilemap_addr = if !self.bg_tilemap {0x9800} else {0x9C00};

        let final_y: u16 = self.line as u16;

        for position_x in 0..256 {
            let final_x: u16 = position_x as u16;

            let bg_x = final_x % 256;
            let bg_y = final_y % 256;

            let tile_pixel_x = bg_x % 8;
            let tile_pixel_y = bg_y % 8;

            let tile_idx =  (bg_y / 8) * 32 + (bg_x / 8);
            let tile_addr = tilemap_addr + tile_idx;

            let tile_id = self.memory.read(tile_addr);
            
            let tile_mem_offset = if self.bg_win_tile_data {tile_id as u16 * 16} else {(tile_id as i8 as i16 + 128) as u16 * 16};
            let tile_start_addr = tileset_addr + tile_mem_offset as u16 + (tile_pixel_y * 2);

            let pixel_1 = self.memory.read(tile_start_addr);
            let pixel_2 = self.memory.read(tile_start_addr + 1);

            let color_idx = {
                let bit1 = (pixel_1 >> 7 - tile_pixel_x) & 1;
                let bit2 = (pixel_2 >> 7 - tile_pixel_x) & 1;

                (bit2 << 1) | (bit1)
            };

            let color = self.tile_palette[color_idx as usize];
            self.game_canvas.set_draw_color(color);
            self.game_canvas.draw_point(Point::new(final_x.wrapping_sub(self.scroll_x as u16) as i32, final_y.wrapping_sub(self.scroll_y as u16) as i32)).unwrap();
        }
    }

    fn draw_window(&mut self) {
        let tileset_addr = if self.bg_win_tile_data {0x8000} else {0x8800};
        let tilemap_addr = if !self.window_tilemap {0x9800} else {0x9C00};

        let final_y: u16 = self.line as u16 + self.memory.read(0xFF4A) as u16;

        for position_x in 0..256 {
            let final_x: u16 = position_x as u16 + self.memory.read(0xFF4B).wrapping_sub(7) as u16;

            let bg_x = final_x % 256;
            let bg_y = final_y % 256;

            let tile_x = bg_x / 8;
            let tile_y = bg_y / 8;

            let tile_pixel_x = bg_x % 8;
            let tile_pixel_y = bg_y % 8;

            let tile_idx =  tile_y * 32 + tile_x;
            let tile_addr = tilemap_addr + tile_idx;

            let tile_id = self.memory.read(tile_addr);
            
            let tile_mem_offset = if self.bg_win_tile_data {tile_id as u16 * 16} else {(tile_id as i8 as i16 + 128) as u16 * 16};
            let tile_line_offset = tile_pixel_y * 2;

            let tile_start_addr = tileset_addr + tile_mem_offset as u16 + tile_line_offset;

            let pixel_1 = self.memory.read(tile_start_addr);
            let pixel_2 = self.memory.read(tile_start_addr + 1);

            let color_idx = {
                let bit1 = (pixel_1 >> 7 - tile_pixel_x) & 1;
                let bit2 = (pixel_2 >> 7 - tile_pixel_x) & 1;

                (bit2 << 1) | (bit1)
            };

            let color = self.tile_palette[color_idx as usize];
            self.game_canvas.set_draw_color(color);
            self.game_canvas.draw_point(Point::new(final_x.wrapping_sub(self.scroll_x as u16) as i32, final_y.wrapping_sub(self.scroll_y as u16) as i32)).unwrap();
        }
    }

    fn make_palette(&mut self, palette: u8) -> Vec<Color> {
        let color_0 = self.base_palette[(palette & 3) as usize];
        let color_1 = self.base_palette[((palette >> 2) & 3) as usize];
        let color_2 = self.base_palette[((palette >> 4) & 3) as usize];
        let color_3 = self.base_palette[(palette >> 6) as usize];

        vec![color_0, color_1, color_2, color_3]
    }

    fn update_gpu_values(&mut self) {
        let lcdc_value = self.memory.read(0xFF40);

        self.lcd_enabled = ((lcdc_value >> 7) & 1) == 1;
        self.window_tilemap = ((lcdc_value >> 6) & 1) == 1;
        self.window_enabled = ((lcdc_value >> 5) & 1) == 1;
        self.bg_win_tile_data = ((lcdc_value >> 4) & 1) == 1;
        self.bg_tilemap = ((lcdc_value >> 3) & 1) == 1;
        self.sprite_size = ((lcdc_value >> 2) & 1) == 1;
        self.sprites_enabled = ((lcdc_value >> 1) & 1) == 1;
        self.bg_enabled = (lcdc_value & 1) == 1;

        self.scroll_y = self.memory.read(0xFF42);
        self.scroll_x = self.memory.read(0xFF43);
        
        self.window_y = self.memory.read(0xFF4A);
        self.window_x = self.memory.read(0xFF4B);

        self.tile_palette = self.make_palette(self.memory.read(0xFF47));
        self.sprites_palettes[0] = self.make_palette(self.memory.read(0xFF48));
        self.sprites_palettes[1] = self.make_palette(self.memory.read(0xFF49));
    }

    fn update_inputs(&mut self) -> bool {
        let input_reg = self.memory.read(0xFF00);
        let mut result = input_reg | 0xCF;

        if (input_reg & 0x20) != 0 {
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Down) {
                result &= 0xF7;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Up) {
                result &= 0xFB;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Left) {
                result &= 0xFD;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Right) {
                result &= 0xFE;
            }
        }
        else if (input_reg & 0x10) != 0 {
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Return) {
                result &= 0xF7;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::RShift) {
                result &= 0xFB;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::S) {
                result &= 0xFD;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::A) {
                result &= 0xFE;
            }
        }
        else {
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Down) || self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Return) {
                result &= 0xF7;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Up) || self.event_pump.keyboard_state().is_scancode_pressed(Scancode::RShift) {
                result &= 0xFB;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Left) || self.event_pump.keyboard_state().is_scancode_pressed(Scancode::S){
                result &= 0xFD;
            }
            if self.event_pump.keyboard_state().is_scancode_pressed(Scancode::Right) || self.event_pump.keyboard_state().is_scancode_pressed(Scancode::A) {
                result &= 0xFE;
            }
        }

        self.memory.write(0xFF00, result, false);

        let mut should_quit = false;

        for event in self.event_pump.poll_event() {
            match event {
                Event::Quit{..} => should_quit = true,
                _ => should_quit = false,
            }
        }

        should_quit
    }

    fn request_interrupt(&self, interrupt: InterruptType) {
        let mut if_value = self.memory.read(0xFF0F);

        // Vblank gets its own interrupt flag.
        if interrupt == InterruptType::Vblank {
            if_value |= 1;
        }
        // But it can also trigger a STAT interrupt, so also check that.
        if self.is_interrupt_enabled(&interrupt) {
            if_value |= 2;
        }

        self.memory.write(0xFF0F, if_value, false);
    }

    fn is_interrupt_enabled(&self, interrupt: &InterruptType) -> bool {
        let bit = *interrupt as u8;
        let stat = self.memory.read(0xFF41);

        ((stat >> bit) & 1) != 0
    }

    fn set_gpu_mode(&self, mode: GpuMode) {
        let stat = self.memory.read(0xFF41) & 0xFC;
        self.memory.write(0xFF41, stat | mode as u8, false);
    }
}