use std::thread;
use std::ops::Neg;
use std::sync::mpsc::{Sender, Receiver};

use log::info;

use sdl2::rect::Point;
use sdl2::event::Event;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::keyboard::Keycode;

use super::utils;
use super::emulator::InputEvent;
use super::memory::{MemoryOp, GpuResponse, MemoryAccess};


pub struct Tile {

    pub tile_colors: Vec<Color>,
}

pub struct BGPoint {

    pub point: Point,
    pub color: Color,
}

pub struct GpuState {

    pub mode: u8,
    pub mode_clock: u16,
    pub line: u8,
    pub all_tiles: Vec<Tile>,
    pub background_points: Vec<BGPoint>,

    pub bg_dirty: bool,
    pub tiles_dirty: bool,
}

pub fn start_gpu(cpu_cycles: Receiver<u16>, memory: (Sender<MemoryAccess>, Receiver<GpuResponse>, Sender<bool>), input: Sender<InputEvent>) {

    let initial_state = GpuState {
        mode: 0,
        mode_clock: 0,
        line: 0,
        all_tiles: Vec::new(),
        background_points: Vec::new(),

        bg_dirty: false,
        tiles_dirty: false,
    };
    
    thread::spawn(move || {
        
        let mut current_state = initial_state;
        let current_memory = (memory.0, memory.1);
        
        let sdl_ctx = sdl2::init().unwrap();
        let sdl_video = sdl_ctx.video().unwrap();
        let mut sdl_events = sdl_ctx.event_pump().unwrap();
        let emu_window = sdl_video.window("Rusty Boi", 160 * 3, 144 * 3).position_centered().build().unwrap();
        let mut emu_canvas = emu_window.into_canvas().present_vsync().build().unwrap();

        // TODO: Add a way to change scaling without having to change it from code.
        // Maybe as an argument, or request a scale multiplier after loading the ROMs.
        emu_canvas.set_scale(3.0, 3.0).unwrap();

        emu_canvas.set_draw_color(Color::RGB(255, 255, 255));
        emu_canvas.clear();
        emu_canvas.present();

        loop {

            for event in sdl_events.poll_iter() {

                match event {
                    Event::Quit {..} => { input.send(InputEvent::Quit).unwrap() },
                    Event::KeyDown { keycode: Some(Keycode::A), .. } => { input.send(InputEvent::APressed).unwrap() },
                    Event::KeyUp  { keycode: Some(Keycode::A), .. } => { input.send(InputEvent::AReleased).unwrap() },
                    Event::KeyDown { keycode: Some(Keycode::S), .. } => { input.send(InputEvent::BPressed).unwrap() },
                    Event::KeyUp  { keycode: Some(Keycode::S), .. } => { input.send(InputEvent::BReleased).unwrap() },
                    Event::KeyDown { keycode: Some(Keycode::Up), .. } => { input.send(InputEvent::UpPressed).unwrap() },
                    Event::KeyUp  { keycode: Some(Keycode::Up), .. } => { input.send(InputEvent::UpReleased).unwrap() },
                    Event::KeyDown { keycode: Some(Keycode::Left), .. } => { input.send(InputEvent::LeftPressed).unwrap() },
                    Event::KeyUp  { keycode: Some(Keycode::Left), .. } => { input.send(InputEvent::LeftReleased).unwrap() },
                    Event::KeyDown { keycode: Some(Keycode::Right), .. } => { input.send(InputEvent::RightPressed).unwrap() },
                    Event::KeyUp  { keycode: Some(Keycode::Right), .. } => { input.send(InputEvent::RightReleased).unwrap() },
                    Event::KeyDown { keycode: Some(Keycode::Down), .. } => { input.send(InputEvent::DownPressed).unwrap() },
                    Event::KeyUp  { keycode: Some(Keycode::Down), .. } => { input.send(InputEvent::DownReleased).unwrap() },
                    Event::KeyDown { keycode: Some(Keycode::Return), .. } => { input.send(InputEvent::StartPressed).unwrap() },
                    Event::KeyUp  { keycode: Some(Keycode::Return), .. } => { input.send(InputEvent::StartReleased).unwrap() },
                    Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => { input.send(InputEvent::SelectPressed).unwrap() },
                    Event::KeyUp  { keycode: Some(Keycode::Backspace), .. } => { input.send(InputEvent::SelectReleased).unwrap() },
                    _ => {}
                }
            }

            let display_enabled: bool;
            let mut if_value = memory_read(0xFF0F, &current_memory);

            // This one isn't using memory_read in purpose.
            // Having one full request on the beginning of the loop is useful so
            // we can get the status of caches.
            let mem_request = MemoryAccess {
                operation: MemoryOp::Read,
                address: 0xFF40,
                value: 0,
            };
            let response: GpuResponse;

            current_memory.0.send(mem_request).unwrap();
            response = current_memory.1.recv().unwrap();
            display_enabled = utils::check_bit(response.read_value, 7);

            current_state.bg_dirty = response.background_dirty;
            current_state.tiles_dirty = response.tiles_dirty;

            if display_enabled {

                current_state.mode_clock += cpu_cycles.recv().unwrap();

                match current_state.mode {

                    2 => {
                        if current_state.mode_clock >= 80 {
                            
                            let mut stat_value = memory_read(0xFF41, &current_memory);
                            
                            current_state.mode_clock = 0;
                            current_state.mode = 3;
                            
                            if utils::check_bit(stat_value, 5) {
                                if_value = utils::set_bit(if_value, 2);
                                memory_write(if_value, 0xFF0F, &current_memory);
                            }

                            stat_value = utils::set_bit(stat_value, 1);
                            stat_value = utils::reset_bit(stat_value, 0);
                            memory_write(stat_value, 0xFF41, &current_memory);
                        }
                    }

                    3 => {
                        if current_state.mode_clock >= 172 {
                            
                            let mut stat_value = memory_read(0xFF41, &current_memory);
                            
                            current_state.mode_clock = 0;
                            current_state.mode = 0;

                            stat_value = utils::set_bit(stat_value, 1);
                            stat_value = utils::set_bit(stat_value, 0);
                            memory_write(stat_value, 0xFF41, &current_memory);

                            if current_state.tiles_dirty {
                                make_tiles(&current_memory, &mut current_state);
                                current_state.tiles_dirty = false;
                            }

                            if current_state.bg_dirty {
                                make_background(&current_memory, &mut current_state);
                                current_state.bg_dirty = false;
                            }

                            memory.2.send(false).unwrap();
                        }
                    }

                    0 => {
                        if current_state.mode_clock >= 204 {
                
                            let mut stat_value = memory_read(0xFF41, &current_memory);
                            
                            current_state.mode_clock = 0;
                            current_state.line += 1;
                            memory_write(current_state.line, 0xFF44, &current_memory);
                            
                            if utils::check_bit(stat_value, 3) {
                                if_value = utils::set_bit(if_value, 2);
                                memory_write(if_value, 0xFF0F, &current_memory);
                            }

                            stat_value = utils::reset_bit(stat_value, 1);
                            stat_value = utils::reset_bit(stat_value, 0);
                            memory_write(stat_value, 0xFF41, &current_memory);

                            if current_state.all_tiles.len() >= 128 && current_state.background_points.len() >= 65536
                            {
                                draw(&mut current_state, &mut emu_canvas, &current_memory);
                            }

                            if current_state.line == 144 {
                                // Go into VBlank mode.
                                current_state.mode = 1;
                                // Send data to screen.
                                emu_canvas.present();
                            }
                        }
                    }
        
                    // VBlank mode
                    1 => {
                        if current_state.mode_clock >= 456 {

                            let mut stat_value = memory_read(0xFF41, &current_memory);
                            
                            current_state.mode_clock = 0;
                            current_state.line += 1;
                            memory_write(current_state.line, 0xFF44, &current_memory);
                            
                            if_value = utils::set_bit(if_value, 0);
                            memory_write(if_value, 0xFF0F, &current_memory);

                            stat_value = utils::reset_bit(stat_value, 1);
                            stat_value = utils::set_bit(stat_value, 0);
                            memory_write(stat_value, 0xFF41, &current_memory);

                            if current_state.line == 154 {
                            // End of the screen, restart.
                                current_state.mode = 2;
                                current_state.line = 1;
                                
                                memory_write(current_state.line, 0xFF44, &current_memory);
                                emu_canvas.clear();
                            }
                        }
                    }

                    _ => panic!("Invalid GPU Mode"),
                }
            }
        }
    });
}

fn draw(state: &mut GpuState, canvas: &mut Canvas<Window>, memory: &(Sender<MemoryAccess>, Receiver<GpuResponse>)) {

    let scroll_x = (memory_read(0xFF43, memory) as i32).neg();
    let scroll_y = (memory_read(0xFF42, memory) as i32).neg();
    let mut point_idx: u16 = 0;
    let mut drawn_pixels: u16 = 0;

    // Index offset for the points array in case the current line is not 0.
    if state.line > 0 {
        point_idx += 256 * state.line as u16;
    }

    // Draw a whole line from the background map, skipping points that are outside the screen.
    while drawn_pixels < 256 {

        let current_point = &state.background_points[point_idx as usize];
        let final_point = current_point.point.offset(scroll_x, scroll_y);

        canvas.set_draw_color(current_point.color);
        canvas.draw_point(final_point).unwrap();
        point_idx += 1;
        drawn_pixels += 1;
    }
}

fn make_tiles(memory: &(Sender<MemoryAccess>, Receiver<GpuResponse>), state: &mut GpuState) {

    let mut memory_position = 0x8000;
    let mut tiles_position = 0;
    let mut new_tiles:Vec<Tile> = Vec::new();

    info!("GPU: Regenerating tile cache");

    while memory_position < 0x9000 {

        let mut loaded_bytes = 0;
        let mut tile_bytes: Vec<u8> = vec![0; 16];

        while loaded_bytes < tile_bytes.len() {

            tile_bytes[loaded_bytes] = memory_read(memory_position, memory);
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
        if false /*tile_bytes[0] == 0 && tile_bytes[1] == 0*/ {
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

fn make_background(memory: &(Sender<MemoryAccess>, Receiver<GpuResponse>), state: &mut GpuState) {

    let mut new_points: Vec<BGPoint> = Vec::new();
    let mut current_background = 0x9800;
    let mut generated_lines: u16 = 0;

    info!("GPU: Regenerating background cache");
    
    while generated_lines < 256 {

        let mut tiles: Vec<&Tile> = Vec::new();
        let mut tile_idx: usize = 0;

        // Loads tile indexes from memory, then gets the tile from GPU State and saves it to tiles.
        // 32 tiles is the maximum amount of tiles per line in the background.
        while tiles.len() < 32 {

            let target_tile = memory_read(current_background, memory);

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

fn memory_read(addr: u16, memory: &(Sender<MemoryAccess>, Receiver<GpuResponse>)) -> u8 {
    
    let mem_request = MemoryAccess {
        operation: MemoryOp::Read,
        address: addr,
        value: 0,
    };
            
    memory.0.send(mem_request).unwrap();
    memory.1.recv().unwrap().read_value
}

fn memory_write(value: u8, addr: u16, memory: &(Sender<MemoryAccess>, Receiver<GpuResponse>)) {

    let mem_request = MemoryAccess {
        operation: MemoryOp::Write,
        address: addr,
        value: value,
    };
            
    memory.0.send(mem_request).unwrap();
}