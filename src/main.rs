mod cpu;
mod cart;
mod video;
mod timer;
mod memory;
mod instructions;

use std::fs;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::borrow::Cow;

use cpu::{UiObject, Status};
use memory::EmulatedMemory;

use log::info;

use imgui::*;

use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use glium::glutin;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::{Display, Surface};

use glium::Texture2d;
use glium::backend::Facade;
use glium::texture::{ClientFormat, UncompressedFloatFormat, MipmapsOption, RawImage2d};

pub static GLOBAL_CYCLE_COUNTER: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);

pub enum InputEvent {
    A,
    B,
    Start,
    Select,

    Up,
    Down,
    Left,
    Right
}

struct EmuState {
    show_cpu_debugger: bool,
    show_cpu_breakpoints: bool,
    show_video_debugger: bool,
    show_io_regs_debugger: bool,
    show_memory_disassembler: bool,

    cpu_breakpoint_value: i32,
    selected_cpu_breakpoint: i32,
    selected_memory_entry: i32,

    scale_factor: i32,
    shared_object: Arc<Mutex<UiObject>>,
    shared_memory: Arc<EmulatedMemory>,

    cart_tx: mpsc::Sender<Vec<u8>>,
    input_tx: mpsc::Sender<InputEvent>,
    mem_rx: mpsc::Receiver<Arc<EmulatedMemory>>,

    started: bool,
    selected_rom_title: String,

    screen_tex_id: Option<TextureId>,
    window_tex_id: Option<TextureId>,
    background_tex_id: Option<TextureId>,
    sprite_tex_ids: Option<Vec<TextureId>>,
}

impl EmuState {
    pub fn new(ui: Arc<Mutex<UiObject>>, mem: Arc<EmulatedMemory>, 
        input_tx: mpsc::Sender<InputEvent>, cart_tx: mpsc::Sender<Vec<u8>>, 
        mem_rx: mpsc::Receiver<Arc<EmulatedMemory>>) -> EmuState {

        EmuState {
            show_cpu_debugger: false,
            show_cpu_breakpoints: false,
            show_video_debugger: false,
            show_io_regs_debugger: false,
            show_memory_disassembler: false,

            cpu_breakpoint_value: 0,
            selected_cpu_breakpoint: 0,
            selected_memory_entry: 0,

            scale_factor: 3,
            shared_object: ui,
            shared_memory: mem,

            cart_tx: cart_tx,
            input_tx: input_tx,
            mem_rx: mem_rx,

            started: false,
            selected_rom_title: String::from("None"),

            screen_tex_id: None,
            window_tex_id: None,
            background_tex_id: None,
            sprite_tex_ids: None,
        }
    }
}

struct ImguiSystem {
    pub event_loop: EventLoop<()>,
    pub display: glium::Display,
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
}

impl ImguiSystem {
    pub fn main_loop(self) {
        let ImguiSystem {
            event_loop,
            display,
            mut imgui,
            mut platform,
            mut renderer,
        } = self;

        let (fb_tx, fb_rx) = mpsc::channel();
        let (mem_tx, mem_rx) = mpsc::channel();
        let (cart_tx, cart_rx) = mpsc::channel();
        let (input_tx, input_rx) = mpsc::channel();

        let shared_object = Arc::new(Mutex::new(UiObject::new()));
        let cpu_object = shared_object.clone();

        let memory = Arc::new(memory::EmulatedMemory::new(load_bootrom()));
        let mut emu_state = EmuState::new(shared_object, memory, input_tx, cart_tx, mem_rx);

        let _emulator_thread = std::thread::Builder::new().name("emulator_thread".to_string()).spawn(move || {
            // Emulated components.
            let mut emu_memory = memory::EmulatedMemory::new(load_bootrom());

            loop {
                let res = cart_rx.try_recv();

                if res.is_ok() {
                    emu_memory.set_cart_data(res.unwrap());
                    break;
                }
            }

            let emu_memory = Arc::new(emu_memory);

            let mut cpu = cpu::Cpu::new(cpu_object, emu_memory.clone(), input_rx);
            let mut video = video::VideoChip::new(emu_memory.clone(), fb_tx);

            mem_tx.send(emu_memory).unwrap();

            loop {
                if let Status::Running{paused, breakpoint: _, step, error: _} = cpu.cpu_status {
                    if paused {
                        if step {
                            cpu.step();
                            video.step();
                        }
                    }
                    else {
                        cpu.step();
                        video.step();
                    }
                }

                cpu.update_ui_object();
            }
        }).unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                platform.prepare_frame(imgui.io_mut(), &gl_window.window()).unwrap();
                gl_window.window().request_redraw();
            },
            Event::RedrawRequested(_) => {
                let ui = imgui.frame();

                ImguiSystem::general_window(&ui, &mut emu_state);
                ImguiSystem::cpu_debugger_window(&ui, &mut emu_state);
                ImguiSystem::cpu_breakpoints_window(&ui, &mut emu_state);
                ImguiSystem::memory_disassembly_window(&ui, &mut emu_state);
                ImguiSystem::io_registers_window(&ui, &mut emu_state);
                ImguiSystem::video_debugger_window(&ui, &mut emu_state);

                if emu_state.started {
                    let scale_factor = emu_state.scale_factor;
                    let received_data = fb_rx.try_iter();
                    let final_texture = Texture2d::empty_with_format(&display, UncompressedFloatFormat::U8U8U8,
                        MipmapsOption::NoMipmap, 160 * scale_factor as u32, 144 * scale_factor as u32).unwrap();
                    let mut full_bg_texture = Texture2d::empty_with_format(&display, UncompressedFloatFormat::U8U8U8,
                        MipmapsOption::NoMipmap, 256, 256).unwrap();
                    let mut full_window_texture = Texture2d::empty_with_format(&display, UncompressedFloatFormat::U8U8U8,
                        MipmapsOption::NoMipmap, 256, 256).unwrap();

                    final_texture.as_surface().clear_color(255.0, 255.0, 255.0, 1.0);
                    full_bg_texture.as_surface().clear_color(255.0, 255.0, 255.0, 1.0);
                    full_window_texture.as_surface().clear_color(255.0, 255.0, 255.0, 1.0);

                    let most_recent_data = received_data.last();

                    if most_recent_data.is_some() {
                        let video_data = most_recent_data.unwrap();

                        let mut bg_data = Vec::with_capacity(256*256);
                        let mut window_data = Vec::with_capacity(256*256);

                        for y in 0..256 {
                            let y_offset = 256 * y;
                            for x in 0..256 {
                                let index = x + y_offset;
                                let color = video_data.background[index];
                                bg_data.push(color);
                                bg_data.push(color);
                                bg_data.push(color);
                            }
                        }

                        for y in 0..256 {
                            let y_offset = 256 * y;
                            for x in 0..256 {
                                let index = x + y_offset;
                                let color = video_data.window[index];
                                window_data.push(color);
                                window_data.push(color);
                                window_data.push(color);
                            }
                        }

                        let raw_bg = RawImage2d {
                            data: Cow::Owned(bg_data),
                            width: 256,
                            height: 256,
                            format: ClientFormat::U8U8U8
                        };

                        let raw_window = RawImage2d {
                            data: Cow::Owned(window_data),
                            width: 256,
                            height: 256,
                            format: ClientFormat::U8U8U8
                        };

                        full_bg_texture = Texture2d::new(display.get_context(), raw_bg).unwrap();
                        full_window_texture = Texture2d::new(display.get_context(), raw_window).unwrap();

                        let bg_blit_target = glium::BlitTarget {
                            left: 0 as u32,
                            bottom: 0 as u32,
                            width: 256 * scale_factor,
                            height: 256 * scale_factor,
                        };

                        let window_blit_target = glium::BlitTarget {
                            left: (video_data.wx.wrapping_sub(7)) as u32,
                            bottom: video_data.wy as u32,
                            width: 256 * scale_factor,
                            height: 256 * scale_factor,
                        };

                        full_bg_texture.as_surface().blit_whole_color_to(&final_texture.as_surface(), &bg_blit_target, 
                            glium::uniforms::MagnifySamplerFilter::Nearest);

                        if video_data.window_enabled {
                            full_window_texture.as_surface().blit_whole_color_to(&final_texture.as_surface(), &window_blit_target, 
                                glium::uniforms::MagnifySamplerFilter::Nearest);
                        }
                    }

                    if emu_state.screen_tex_id.is_some() {
                        renderer.textures().replace(emu_state.screen_tex_id.unwrap(), Rc::new(final_texture));
                    }
                    else {
                        emu_state.screen_tex_id = Some(renderer.textures().insert(Rc::new(final_texture)));
                    }

                    if emu_state.background_tex_id.is_some() {
                        renderer.textures().replace(emu_state.background_tex_id.unwrap(), Rc::new(full_bg_texture));
                    }
                    else {
                        emu_state.background_tex_id = Some(renderer.textures().insert(Rc::new(full_bg_texture)));
                    }

                    if emu_state.window_tex_id.is_some() {
                        renderer.textures().replace(emu_state.window_tex_id.unwrap(), Rc::new(full_window_texture));
                    }
                    else {
                        emu_state.window_tex_id = Some(renderer.textures().insert(Rc::new(full_window_texture)));
                    }

                    ImguiSystem::screen_window(&ui, &mut emu_state);
                    ImguiSystem::controls_window(&ui, &mut emu_state);
                }

                let gl_window = display.gl_window();
                let mut target = display.draw();

                target.clear_color_srgb(0.2, 0.2, 0.2, 1.0);
                platform.prepare_render(&ui, gl_window.window());

                let draw_data = ui.render();
                renderer.render(&mut target, draw_data).unwrap();
                target.finish().unwrap();
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            event => {
                let gl_window = display.gl_window();
                platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
            }
        });
    }

    fn general_window(ui: &Ui, emu_state: &mut EmuState) {
        Window::new(im_str!("Rusty Boi - Controls")).build(&ui, || {
            ui.bullet_text(im_str!("ROM Selection"));
            ui.separator();

            let all_roms = get_all_roms();
            ImguiSystem::create_rom_list(&ui, emu_state, &all_roms);

            ui.text(format!("ROM Title: {}", emu_state.selected_rom_title));
            ui.separator();

            ui.bullet_text(im_str!("Resolution"));
            ui.input_int(im_str!("Resolution Scale"), &mut emu_state.scale_factor).build();
            ui.separator();

            ui.bullet_text(im_str!("Emulation Controls"));
            if ui.button(im_str!("Start/Resume"), [120.0, 20.0]) {
                let mut lock = emu_state.shared_object.lock().unwrap();
                
                emu_state.started = true;
                lock.cpu_step = Some(false);
                lock.cpu_paused = Some(false);
            }
            ui.same_line(140.0);
            if ui.button(im_str!("Pause"), [120.0, 20.0]) {
                let mut lock = emu_state.shared_object.lock().unwrap();
                
                lock.cpu_paused = Some(true);
            }
            ui.separator();

            ui.checkbox(im_str!("Show CPU debugger"), &mut emu_state.show_cpu_debugger);
            ui.checkbox(im_str!("Show Video debugger"), &mut emu_state.show_video_debugger);
        });
    }

    fn screen_window(ui: &Ui, emu_state: &mut EmuState) {
        Window::new(im_str!("Rusty Boi - Screen")).build(&ui, || {
            ui.bullet_text(im_str!("Screen Output:"));
            let size_x = 160.0 * emu_state.scale_factor as f32;
            let size_y = 144.0 * emu_state.scale_factor as f32;
            Image::new(emu_state.screen_tex_id.unwrap(), [size_x, size_y]).build(&ui);
        });
    }

    fn controls_window(ui: &Ui, emu_state: &mut EmuState) {
        Window::new(im_str!("Rusty Boi - Game Controls")).build(&ui, || {
            ui.bullet_text(im_str!("Controls:"));
            ui.separator();

            if ui.button(im_str!("A"), [50.0, 20.0]) {
                emu_state.input_tx.send(InputEvent::A).unwrap();
            }
            ui.same_line(65.0);
            if ui.button(im_str!("B"), [50.0, 20.0]) {
                emu_state.input_tx.send(InputEvent::B).unwrap();
            }
            ui.same_line(120.0);
            if ui.button(im_str!("Start"), [50.0, 20.0]) {
                emu_state.input_tx.send(InputEvent::Start).unwrap();
            }
            ui.same_line(180.0);
            if ui.button(im_str!("Select"), [50.0, 20.0]) {
                emu_state.input_tx.send(InputEvent::Select).unwrap();
            }

            if ui.button(im_str!("Up"), [50.0, 20.0]) {
                emu_state.input_tx.send(InputEvent::Up).unwrap();
            }
            ui.same_line(65.0);
            if ui.button(im_str!("Down"), [50.0, 20.0]) {
                emu_state.input_tx.send(InputEvent::Down).unwrap();
            }
            ui.same_line(120.0);
            if ui.button(im_str!("Left"), [50.0, 20.0]) {
                emu_state.input_tx.send(InputEvent::Left).unwrap();
            }
            ui.same_line(180.0);
            if ui.button(im_str!("Right"), [50.0, 20.0]) {
                emu_state.input_tx.send(InputEvent::Right).unwrap();
            }
        });
    }

    fn cpu_debugger_window(ui: &Ui, emu_state: &mut EmuState) {
        if emu_state.show_cpu_debugger {
            Window::new(im_str!("Rusty Boi - CPU Debugger")).build(&ui, || {
                let mut lock = emu_state.shared_object.lock().unwrap();
                
                ui.bullet_text(im_str!("CPU Registers"));
                ui.separator();

                ui.text(format!("AF: {:04X}", lock.registers[0]));
                ui.same_line(80.0);
                ui.text(format!("BC: {:04X}", lock.registers[1]));

                ui.text(format!("DE: {:04X}", lock.registers[2]));
                ui.same_line(80.0);
                ui.text(format!("HL: {:04X}", lock.registers[3]));

                ui.text(format!("SP: {:04X}", lock.registers[4]));
                ui.same_line(80.0);
                ui.text(format!("PC: {:04X}", lock.pc));
                ui.text(format!("Current instruction: {}", instructions::get_instruction_disassembly(
                    &mut lock.pc.clone(), &emu_state.shared_memory).split('-').last().unwrap().trim()));
                ui.separator();

                ui.bullet_text(im_str!("CPU State and Controls"));
                ui.separator();

                let cpu_status = lock.cpu_status;

                match cpu_status {
                    Status::Running {paused, breakpoint, step: _, error} => {
                        if paused {
                            ui.text_colored([1.0, 0.5, 1.0, 1.0], "Status: Paused by debugger");
                        }
                        else if breakpoint {
                            ui.text_colored([0.3, 1.0, 1.0, 1.0], "Status: Breakpoint hit");
                        }
                        else if error {
                            ui.text_colored([1.0, 0.0, 0.0, 1.0], "Status: Error");
                        }
                        else if lock.halted {
                            ui.text_colored([0.7, 1.0, 1.0, 1.0], "Status: CPU Halted");
                        }
                        else {
                            ui.text_colored([0.0, 1.0, 0.0, 1.0], "Status: Running");
                        }
                    },
                    Status::NotReady => {
                        ui.text_colored([1.0, 0.5, 1.0, 1.0], "Status: CPU not ready.");
                    }
                }

                ui.separator();

                if ui.button(im_str!("Break"), [50.0, 20.0]) {
                    lock.cpu_paused = Some(true);
                }
                ui.same_line(65.0);
                if ui.button(im_str!("Step"), [50.0, 20.0]) {
                    if emu_state.started {
                        lock.cpu_step = Some(true);
                        lock.cpu_paused = Some(true);
                    }
                }
                ui.same_line(122.0);
                if ui.button(im_str!("Resume"), [50.0, 20.0]) {
                    if emu_state.started {
                        lock.cpu_step = Some(false);
                        lock.cpu_paused = Some(false);
                    }
                }
                ui.separator();

                ui.checkbox(im_str!("Show breakpoints"), &mut emu_state.show_cpu_breakpoints);
                ui.checkbox(im_str!("Show interrupts state"), &mut emu_state.show_io_regs_debugger);
                ui.checkbox(im_str!("Show memory disassembler"), &mut emu_state.show_memory_disassembler);
            });
        }
    }

    fn cpu_breakpoints_window(ui: &Ui, emu_state: &mut EmuState) {
        if emu_state.show_cpu_breakpoints {
            Window::new(im_str!("Rusty Boi - CPU Breakpoints")).build(&ui, || {
                let mut lock = emu_state.shared_object.lock().unwrap();
                let mut all_breakpoints = Vec::new();

                for set_breakpoint in lock.breakpoints.iter() {
                    all_breakpoints.push(ImString::from(format!("{:04X}", set_breakpoint)))
                }

                let strings: Vec<&ImStr> = all_breakpoints.iter().map(|s| s.as_ref()).collect();
                ui.list_box(im_str!("CPU Breakpoints"), &mut emu_state.selected_cpu_breakpoint, 
                &strings[..], 10);

                if ui.input_int(im_str!("Breakpoint address: "), &mut emu_state.cpu_breakpoint_value)
                .chars_hexadecimal(true)
                .chars_noblank(true)
                .enter_returns_true(true)
                .build() {
                    lock.breakpoints.push(emu_state.cpu_breakpoint_value as u16);
                }

                if ui.button(im_str!("Remove"), [50.0, 20.0]) {
                    if lock.breakpoints.len() > 0 {
                        lock.breakpoints.remove(emu_state.selected_cpu_breakpoint as usize);
                    }
                }
            });
            
        }
    }

    fn memory_disassembly_window(ui: &Ui, emu_state: &mut EmuState) {
        if emu_state.show_memory_disassembler {
            Window::new(im_str!("Rusty Boi - Memory Disassembler")).build(&ui, || {
                let mut address = 0;
                let mut all_entries = Vec::new();
    
                while address < 0xFF80 {
                    let lock = emu_state.shared_object.lock().unwrap();
                    let mut result = String::new();

                    if address == lock.pc {
                        result.insert(0, '>');
                    }

                    result += &instructions::get_instruction_disassembly(&mut address, &emu_state.shared_memory);
                    all_entries.push(ImString::from(result));
                }
    
                // Get $FFFF in there as well.
                // Overflows address, since it gets increased by 1.
                /*all_entries.push(ImString::from(instructions::get_instruction_disassembly(&mut 0xFFFF, 
                    &emu_state.shared_memory)));*/
    
                let strings: Vec<&ImStr> = all_entries.iter().map(|s| s.as_ref()).collect();
                ui.list_box(im_str!("Memory"), &mut emu_state.selected_memory_entry, &strings[..], 20);
            });
        }
    }

    fn video_debugger_window(ui: &Ui, emu_state: &mut EmuState) {
        if emu_state.show_video_debugger {
            Window::new(im_str!("Rusty Boi - Video Debugger")).build(&ui, || {
                let ly = emu_state.shared_memory.read(0xFF44);
                let lyc = emu_state.shared_memory.read(0xFF45);
                let wx = emu_state.shared_memory.read(0xFF4A);
                let wy = emu_state.shared_memory.read(0xFF4B);
                let lcd_stat = emu_state.shared_memory.read(0xFF41);
                let lcd_control = emu_state.shared_memory.read(0xFF40);

                ui.bullet_text(im_str!("LCD Control"));
                ui.separator();
                ui.text(format!("LCD Enabled: {}", (lcd_control & 0x80) != 0));
                ui.text(format!("Window Tilemap: {}", if (lcd_control & 0x40) != 0 {"0x9C00"} else {"0x9800"}));
                ui.text(format!("Window Enabled: {}", (lcd_control & 0x20) != 0));
                ui.text(format!("Window and BG Tile Data: {}", 
                    if (lcd_control & 0x10) != 0 {"0x8800"} else {"0x8000"}));
                ui.text(format!("BG Tilemap: {}", if (lcd_control & 0x08) != 0 {"0x9C00"} else {"0x9800"}));
                ui.text(format!("Sprite Size: {}", if (lcd_control & 0x04) != 0 {"8x16"} else {"8x8"}));
                ui.text(format!("Sprites Enabled: {}", (lcd_control & 0x02) != 0));
                ui.text(format!("BG Enabled: {}", (lcd_control & 0x01) != 0));
                ui.separator();

                ui.bullet_text(im_str!("LCD Status"));
                ui.separator();
                ui.text(format!("LYC Interrupt: {}", (lcd_stat & 0x40) != 0));
                ui.text(format!("OAM Mode Interrupt: {}", (lcd_stat & 0x20) != 0));
                ui.text(format!("VBlank Mode Interrupt: {}", (lcd_stat & 0x10) != 0));
                ui.text(format!("HBlank Mode Interrupt: {}", (lcd_stat & 0x08) != 0));
                ui.text(format!("Coincidence Flag: {}", (lcd_stat & 0x04) != 0));
                ui.text(format!("Current mode: {}", lcd_stat & 0x03));
                ui.separator();

                ui.bullet_text(im_str!("Various"));
                ui.separator();
                ui.text(format!("LY: {}", ly));
                ui.text(format!("LYC: {}", lyc));
                ui.text(format!("WX: {}", wx));
                ui.text(format!("WY: {}", wy));
                ui.separator();

                ui.bullet_text(im_str!("Full Background:"));
                ui.same_line(300.0);
                ui.bullet_text(im_str!("Full Window:"));

                if emu_state.started {
                    if emu_state.background_tex_id.is_some() {
                        Image::new(emu_state.background_tex_id.unwrap(), [256.0, 256.0]).build(&ui);
                    }
                    
                    ui.same_line(300.0);

                    if emu_state.window_tex_id.is_some() {
                        Image::new(emu_state.window_tex_id.unwrap(), [256.0, 256.0]).build(&ui);
                    }
                }
            });
        }
    }

    fn io_registers_window(ui: &Ui, emu_state: &mut EmuState) {
        if emu_state.show_io_regs_debugger {
            Window::new(im_str!("Rusty Boi - Interrupts")).build(&ui, || {
                let if_value = emu_state.shared_memory.read(0xFF0F);
                let ie_value = emu_state.shared_memory.read(0xFFFF);

                ui.bullet_text(im_str!("Enabled Interrupts"));
                ui.separator();
                ui.text(format!("V-Blank: {}", ie_value & 1));
                ui.text(format!("LCD STAT: {}", (ie_value >> 1) & 1));
                ui.text(format!("Timer: {}", (ie_value >> 2) & 1));
                ui.text(format!("Serial: {}", (ie_value >> 3) & 1));
                ui.text(format!("Joypad: {}", (ie_value >> 4) & 1));
                ui.separator();

                ui.bullet_text(im_str!("Requested Interrupts"));
                ui.separator();
                ui.text(format!("V-Blank: {}", if_value & 1));
                ui.text(format!("LCD STAT: {}", (if_value >> 1) & 1));
                ui.text(format!("Timer: {}", (if_value >> 2) & 1));
                ui.text(format!("Serial: {}", (if_value >> 3) & 1));
                ui.text(format!("Joypad: {}", (if_value >> 4) & 1));
                ui.separator();
            });
        }
    }

    fn create_rom_list(ui: &Ui, emu_state: &mut EmuState, paths: &Vec<PathBuf>) {
        if let Some(menu) = ui.begin_menu(im_str!("Detected ROMs"), paths.len() > 0 && !emu_state.started) {
            for entry in paths {
                ImguiSystem::create_rom_list_entry(ui, emu_state, &entry);
            }

            menu.end(&ui);
        }
    }

    fn create_rom_list_entry(ui: &Ui, emu_state: &mut EmuState, path: &PathBuf) {
        if path.is_dir() {
            let mut files: Vec<_> = fs::read_dir(path).unwrap().map(|r| r.unwrap()).collect();
            files.sort_by_key(|dir| dir.path().to_str().unwrap().to_lowercase());

            if let Some(menu) = ui.begin_menu(&ImString::from(String::from(path.file_name().unwrap().to_str().unwrap())), 
                !emu_state.started) {
                for entry in files {
                    ImguiSystem::create_rom_list_entry(ui, emu_state, &entry.path());
                }
    
                menu.end(&ui);
            }
        }
        else {
            let filename = ImString::from(String::from(path.file_name().unwrap().to_str().unwrap()));

            if MenuItem::new(&filename).build_with_ref(&ui, &mut false) {
                let read_data = fs::read(path).unwrap();
                emu_state.selected_rom_title.clear();

                for idx in 0x0134..0x143 {
                    let value = read_data[idx];

                    if value != 0 {
                        emu_state.selected_rom_title.push(value as char);
                    }
                }

                emu_state.cart_tx.send(read_data).unwrap();
                emu_state.shared_memory = emu_state.mem_rx.recv().unwrap();
            }
        }
    }
}

fn main() {
    // Initialize the logger
    simple_logger::init_with_level(log::Level::Info).unwrap();
    info!("Rusty Boi");

    let imgui_system = init_imgui();
    imgui_system.main_loop();
}

fn init_imgui() -> ImguiSystem {
    let event_loop = EventLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let builder = WindowBuilder::new().with_title("Rusty Boi").with_inner_size(glutin::dpi::LogicalSize::new(1600, 720));
    let display = Display::new(builder, context, &event_loop).expect("Failed to init display");

    let mut imgui = Context::create();
    let mut platform = WinitPlatform::init(&mut imgui);

    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
    }

    let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    ImguiSystem {
        event_loop,
        display,
        imgui,
        platform,
        renderer,
    }
}

fn get_all_roms() -> Vec<PathBuf> {
    init_dirs();
    get_paths_in_dir(PathBuf::from("roms"))
}

fn get_paths_in_dir(path: PathBuf) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut read_files: Vec<_> = fs::read_dir(path).unwrap().map(|r| r.unwrap()).collect();

    read_files.sort_by_key(|dir| dir.path().to_str().unwrap().to_lowercase());
    for entry in read_files {
        result.push(entry.path());
    }

    result
}

fn init_dirs() {
    let roms_dir = fs::create_dir("roms");
    match roms_dir {

        Ok(_result) => {},
        Err(error) => {
            match error.kind() {
                io::ErrorKind::AlreadyExists => {},
                io::ErrorKind::PermissionDenied => { log::error!("Failed to create ROMs directory: Permission Denied") },
                _ => {},
            }
        }
    }
}

fn load_bootrom() -> Option<Vec<u8>> {
    let data = fs::read("Bootrom.gb");
    
    if data.is_ok() {
        Some(data.unwrap())
    }
    else {
        None
    }
}