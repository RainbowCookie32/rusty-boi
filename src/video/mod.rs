use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::sync::atomic::Ordering;

use super::memory::SharedMemory;

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
    NotLyCoincidence,
}

#[derive(Clone)]
pub struct VideoData {
    pub background: Vec<u8>,
    pub bg_scx: u8,
    pub bg_scy: u8,

    pub wx: u8,
    pub wy: u8,
    pub window: Vec<u8>,
    pub window_enabled: bool,

    pub sprites: Vec<u8>,
}

impl VideoData {
    pub fn new(bg: Vec<u8>, scx: u8, scy: u8, window: Vec<u8>, wenabled: bool, sprites: Vec<u8>) -> VideoData {
        VideoData {
            background: bg,
            bg_scx: scx,
            bg_scy: scy,

            wx: 0,
            wy: 0,
            window: window,
            window_enabled: wenabled,

            sprites: sprites
        }
    }
}

pub struct ColorPalette {
    value: u8,
    palette: Vec<u8>,
    base_palette: Vec<u8>,
}

impl ColorPalette {
    pub fn new() -> ColorPalette {
        ColorPalette {
            value: 0,
            palette: vec![255, 192, 96, 0],
            base_palette: vec![255, 192, 96, 0],
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

    pub fn get_color(&self, idx: u8) -> u8 {
        self.palette[idx as usize]
    }
}

pub struct VideoChip {
    mode: VideoMode,
    current_cycles: u16,
    display_enabled: bool,

    tbank_0: Vec<Vec<u8>>,
    tbank_1: Vec<Vec<u8>>,
    sprites: Vec<u8>,

    tile_palette: ColorPalette,
    sprite_palettes: Vec<ColorPalette>,

    oam_state: (u64, bool),
    t0_state: (u64, bool),
    t1_state: (u64, bool),

    render_data: VideoData,
    sender: Sender<VideoData>,
    memory: Arc<SharedMemory>,
}

impl VideoChip {
    pub fn new(sender: Sender<VideoData>, memory: Arc<SharedMemory>) -> VideoChip {
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

            render_data: VideoData::new(vec![0; 256*256], 0, 0, vec![0; 256*256], false, Vec::new()),
            sender: sender,
            memory: memory,
        }
    }

    pub fn step(&mut self) {
        self.update_video_values();

        self.current_cycles = self.current_cycles.wrapping_add(super::GLOBAL_CYCLE_COUNTER.load(Ordering::Relaxed));

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

            if self.mode == VideoMode::Hblank {
                if self.current_cycles >= 204 {
                    self.current_cycles = self.current_cycles % 204;
                    self.hblank_mode();
                }
            }
            else if self.mode == VideoMode::Vblank {
                if self.current_cycles >= 456 {
                    self.current_cycles = self.current_cycles % 456;
                    self.vblank_mode();
                }
                
            }
            else if self.mode == VideoMode::OamSearch {
                if self.current_cycles >= 80 {
                    self.current_cycles = self.current_cycles % 80;
                    self.oam_scan_mode();
                }
            }
            else if self.mode == VideoMode::LcdTransfer {
                if self.current_cycles >= 172 {
                    self.current_cycles = self.current_cycles % 172;
                    self.lcd_transfer_mode();
                }
            }

            let ly = self.memory.read(LY);
            let lyc = self.memory.read(LYC);

            if ly == lyc {
                self.update_video_mode(VideoMode::LyCoincidence);
            }
            else {
                self.update_video_mode(VideoMode::NotLyCoincidence);
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
                stat_value &= 0xFC;
                stat_value |= 3;
            },
            VideoMode::Hblank => {
                stat_value &= 0xFC;
                if ((stat_value >> 3) & 1) != 0 {
                    if_value |= 2;
                }
            },
            VideoMode::Vblank => {
                stat_value &= 0xFC;
                stat_value |= 1;
                if ((stat_value >> 4) & 1) != 0 {
                    if_value |= 2;
                }
            },
            VideoMode::OamSearch => {
                stat_value &= 0xFC;
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
            },
            VideoMode::NotLyCoincidence => {
                stat_value &= 0xFB;
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
            self.render_data.window_enabled = true;
        }
        else {
            self.render_data.window_enabled = false;
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
        }
        else {
            self.mode = VideoMode::OamSearch;
        }
    }

    fn vblank_mode(&mut self) {
        self.current_cycles = 0;
        let ly_value = self.memory.read(LY) + 1;
        self.memory.write(LY, ly_value, false);

        self.update_video_mode(VideoMode::Vblank);
        self.draw_background();

        if ly_value == 154 {
            self.mode = VideoMode::OamSearch;
            self.update_video_mode(VideoMode::OamSearch);
            self.memory.write(LY, 0, false);

            let _result = self.sender.send(self.render_data.clone());
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
        // Scrolling is not working properly right now, since it doesn't account for new scrolling
        // values being set while drawing the frame. A possible solution to this would be to have a
        // line struct that is fed with the bytes for that line, and then constructs a vector with the
        // scrolling value for that line applied. Initialize a vector with 256 items, then assign initial index to
        // the value of scroll_x and add one 256 times. wrapping_add should give me proper wrapping.
        let line = self.memory.read(LY) as u32;
        let lcd_control = self.memory.read(LCD_CONTROL);
        let use_signed_tiles = (lcd_control & 0x10) == 0;
        let background_address = (if (lcd_control & 0x08) == 0 {0x9800} else {0x9C00}) + (32 * (line / 8) as u16);

        let tile_y_offset = line % 8;

        let mut drawn_tiles = 0;

        let scy = self.memory.read(SCROLL_Y);
        let scx = self.memory.read(SCROLL_X);

        let mut target_idx = 256 * line;
        
        while drawn_tiles < 32 {
            let tile: &Vec<u8>;
            let tile_idx = self.memory.read(background_address + drawn_tiles);
            let mut drawn_pixels = 0;
            let mut draw_idx = 8 * tile_y_offset;

            if use_signed_tiles {
                tile = &self.tbank_1[(tile_idx  as i8 as i16 + 128) as usize];
            }
            else {
                tile = &self.tbank_0[tile_idx as usize];
            }
                
            while drawn_pixels < 8 {
                let color = self.tile_palette.get_color(tile[draw_idx as usize]);
                self.render_data.background[target_idx as usize] = color;
                target_idx += 1;
                draw_idx += 1;
                drawn_pixels += 1;
            }

            drawn_tiles += 1;
        }

        self.render_data.bg_scx = scx;
        self.render_data.bg_scy = scy;
    }

    fn draw_window(&mut self) {
        let line = self.memory.read(LY);
        let lcd_control = self.memory.read(LCD_CONTROL);
        let use_signed_tiles = (lcd_control & 0x10) == 0;
        let background_address = (if (lcd_control & 0x40) == 0 {0x9800} else {0x9C00}) + (32 * (line / 8) as u16);

        let tile_y_offset = line % 8;

        let mut drawn_tiles = 0;
        let mut target_idx = 256 * line as u32;

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
                let color = self.tile_palette.get_color(tile[draw_idx as usize]);
                self.render_data.window[target_idx as usize] = color;
                target_idx += 1;
                draw_idx += 1;
                drawn_pixels += 1;
            }

            drawn_tiles += 1;
        }

        self.render_data.wx = self.memory.read(WX);
        self.render_data.wy = self.memory.read(WY);
    }

    fn draw_sprites(&mut self) {
        
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
        
    }
}