mod cpu;
mod cart;
mod video;
mod timer;
mod memory;

use std::fs;
use std::io;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::borrow::Cow;

use cpu::UiObject;

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
        let (input_tx, input_rx) = mpsc::channel();

        let ui_object = Arc::new(Mutex::new(UiObject::new()));
        let cpu_object = ui_object.clone();

        // Debugging.
        let mut show_debugger = false;
        let mut show_video_debugger = false;
        let mut show_interrupts_state = false;

        let mut scale_factor = 3;
        let mut emu_started = false;
        let mut selected_rom: Option<Vec<u8>> = None;
        let mut selected_rom_title = String::from("None");

        let mut framebuffer_id: Option<TextureId> = None;
        let mut background_id: Option<TextureId> = None;
        let mut window_id: Option<TextureId> = None;

        let _emulator_thread = std::thread::Builder::new().name("emulator_thread".to_string()).spawn(move || {
            // Emulated GB memory. CPU memory is a placeholder until a ROM is loaded.
            let mut emu_memory = memory::EmulatedMemory::new(load_bootrom());

            // Emulated components.
            let mut cpu = cpu::Cpu::new(cpu_object, input_rx);
            let mut video = video::VideoChip::new(fb_tx);

            loop {
                if !cpu.cpu_paused {
                    cpu.step(&mut emu_memory);
                    video.step(&mut emu_memory);
                }
                else {
                    if cpu.cpu_step {
                        cpu.step(&mut emu_memory);
                        video.step(&mut emu_memory);
                        cpu.cpu_step = false;
                    }
                    else {
                        cpu.update_ui_object(&mut emu_memory);
                    }
                }
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

                Window::new(im_str!("Rusty Boi - Controls")).build(&ui, || {
                    ui.bullet_text(im_str!("ROM Selection"));
                    ui.separator();

                    let all_roms = get_all_roms();
                    
                    if let Some(menu) = ui.begin_menu(im_str!("Detected ROMs"), all_roms.len() > 0 && !emu_started) {
                        for file in all_roms {
                            let filename = ImString::from(file.file_name().into_string().unwrap());

                            if MenuItem::new(&filename).build_with_ref(&ui, &mut false) {
                                let mut lock = ui_object.lock().unwrap();
                                let read_data = fs::read(file.path()).unwrap();
                                
                                selected_rom_title.clear();

                                for idx in 0x0134..0x143 {
                                    let value = read_data[idx];

                                    if value != 0 {
                                        selected_rom_title.push(value as char);
                                    }
                                }

                                selected_rom = Some(read_data.clone());
                                lock.update_cart = true;
                                lock.new_cart_data = read_data;
                            }
                        }

                        menu.end(&ui);
                    }

                    ui.text(format!("ROM Title: {}", selected_rom_title));
                    ui.separator();

                    ui.bullet_text(im_str!("Resolution"));
                    ui.input_int(im_str!("Resolution Scale"), &mut scale_factor).build();
                    ui.separator();

                    ui.bullet_text(im_str!("Emulation Controls"));
                    if ui.button(im_str!("Start/Resume"), [120.0, 20.0]) {
                        if !emu_started {
                            if selected_rom.is_some() {
                                emu_started = true;
                                ui_object.lock().unwrap().cpu_paused = false;
                            }
                        }
                        else {
                            ui_object.lock().unwrap().cpu_paused = false;
                        }
                    }
                    ui.same_line(140.0);
                    if ui.button(im_str!("Pause"), [120.0, 20.0]) {
                        ui_object.lock().unwrap().cpu_paused = true;
                    }
                    ui.separator();

                    ui.checkbox(im_str!("Show CPU debugger"), &mut show_debugger);
                    ui.checkbox(im_str!("Show Video debugger"), &mut show_video_debugger);
                });

                if emu_started {
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
                            left: (video_data.bg_scx as i8 * -1) as u32,
                            bottom: (video_data.bg_scy as i8 * -1) as u32,
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

                    if framebuffer_id.is_some() {
                        renderer.textures().replace(framebuffer_id.unwrap(), Rc::new(final_texture));
                    }
                    else {
                        framebuffer_id = Some(renderer.textures().insert(Rc::new(final_texture)));
                    }

                    if background_id.is_some() {
                        renderer.textures().replace(background_id.unwrap(), Rc::new(full_bg_texture));
                    }
                    else {
                        background_id = Some(renderer.textures().insert(Rc::new(full_bg_texture)));
                    }

                    if window_id.is_some() {
                        renderer.textures().replace(window_id.unwrap(), Rc::new(full_window_texture));
                    }
                    else {
                        window_id = Some(renderer.textures().insert(Rc::new(full_window_texture)));
                    }

                    Window::new(im_str!("Rusty Boi - Screen")).build(&ui, || {
                        ui.bullet_text(im_str!("Screen Output:"));
                        let size_x = 160.0 * scale_factor as f32;
                        let size_y = 144.0 * scale_factor as f32;
                        Image::new(framebuffer_id.unwrap(), [size_x, size_y]).build(&ui);
                    });

                    Window::new(im_str!("Rusty Boi - Game Controls")).build(&ui, || {
                        ui.bullet_text(im_str!("Controls:"));
                        ui.separator();

                        if ui.button(im_str!("A"), [50.0, 20.0]) {
                            input_tx.send(InputEvent::A).unwrap();
                        }
                        ui.same_line(45.0);
                        if ui.button(im_str!("B"), [50.0, 20.0]) {
                            input_tx.send(InputEvent::B).unwrap();
                        }
                        ui.same_line(90.0);
                        if ui.button(im_str!("Start"), [50.0, 20.0]) {
                            input_tx.send(InputEvent::Start).unwrap();
                        }
                        ui.same_line(135.0);
                        if ui.button(im_str!("Select"), [50.0, 20.0]) {
                            input_tx.send(InputEvent::Select).unwrap();
                        }

                        if ui.button(im_str!("Up"), [50.0, 20.0]) {
                            input_tx.send(InputEvent::Up).unwrap();
                        }
                        ui.same_line(45.0);
                        if ui.button(im_str!("Down"), [50.0, 20.0]) {
                            input_tx.send(InputEvent::Down).unwrap();
                        }
                        ui.same_line(90.0);
                        if ui.button(im_str!("Left"), [50.0, 20.0]) {
                            input_tx.send(InputEvent::Left).unwrap();
                        }
                        ui.same_line(135.0);
                        if ui.button(im_str!("Right"), [50.0, 20.0]) {
                            input_tx.send(InputEvent::Right).unwrap();
                        }
                    });
                }

                if show_debugger {
                    Window::new(im_str!("Rusty Boi - CPU Debugger")).build(&ui, || {
                        let mut lock = ui_object.lock().unwrap();
                        
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
                        ui.text(format!("Current instruction: {:02X}", lock.opcode));
                        ui.separator();

                        ui.bullet_text(im_str!("CPU State and Controls"));
                        ui.separator();

                        if lock.cpu_paused {
                            ui.text_colored([1.0, 0.5, 1.0, 1.0], "Status: Paused by debugger");
                        }
                        else if lock.breakpoint_hit {
                            ui.text_colored([0.3, 1.0, 1.0, 1.0], "Status: Breakpoint hit");
                        }
                        else if lock.halted {
                            ui.text_colored([0.7, 1.0, 1.0, 1.0], "Status: CPU Halted");
                        }
                        else {
                            ui.text_colored([0.0, 1.0, 0.0, 1.0], "Status: Running");
                        }
                        ui.separator();

                        if ui.button(im_str!("Break"), [50.0, 20.0]) {
                            lock.cpu_paused = true;
                        }
                        ui.same_line(65.0);
                        if ui.button(im_str!("Step"), [50.0, 20.0]) {
                            if emu_started {
                                lock.cpu_paused = true;
                                lock.cpu_should_step = true;
                            }
                        }
                        ui.same_line(122.0);
                        if ui.button(im_str!("Resume"), [50.0, 20.0]) {
                            if emu_started {
                                lock.cpu_paused = false;
                                lock.cpu_should_step = false;
                            }
                        }
                        ui.separator();

                        ui.checkbox(im_str!("Show interrupts state"), &mut show_interrupts_state);
                    });

                    if show_interrupts_state {
                        Window::new(im_str!("Rusty Boi - Interrupts")).build(&ui, || {
                            let lock = ui_object.lock().unwrap();

                            ui.bullet_text(im_str!("Enabled Interrupts"));
                            ui.separator();
                            ui.text(format!("V-Blank: {}", lock.ie_value & 1));
                            ui.text(format!("LCD STAT: {}", (lock.ie_value >> 1) & 1));
                            ui.text(format!("Timer: {}", (lock.ie_value >> 2) & 1));
                            ui.text(format!("Serial: {}", (lock.ie_value >> 3) & 1));
                            ui.text(format!("Joypad: {}", (lock.ie_value >> 4) & 1));
                            ui.separator();

                            ui.bullet_text(im_str!("Requested Interrupts"));
                            ui.separator();
                            ui.text(format!("V-Blank: {}", lock.if_value & 1));
                            ui.text(format!("LCD STAT: {}", (lock.if_value >> 1) & 1));
                            ui.text(format!("Timer: {}", (lock.if_value >> 2) & 1));
                            ui.text(format!("Serial: {}", (lock.if_value >> 3) & 1));
                            ui.text(format!("Joypad: {}", (lock.if_value >> 4) & 1));
                            ui.separator();
                        });
                    }
                }

                if show_video_debugger {
                    Window::new(im_str!("Rusty Boi - Video Debugger")).build(&ui, || {
                        let lock = ui_object.lock().unwrap();

                        ui.bullet_text(im_str!("LCD Control"));
                        ui.separator();
                        ui.text(format!("LCD Enabled: {}", (lock.lcd_control >> 7) != 0));
                        ui.text(format!("Window Tilemap: {}", if (lock.lcd_control >> 6) != 0 {"0x9C00"} else {"0x9800"}));
                        ui.text(format!("Window Enabled: {}", (lock.lcd_control >> 5) != 0));
                        ui.text(format!("Window and BG Tile Data: {}", 
                            if (lock.lcd_control >> 4) != 0 {"0x8800"} else {"0x8000"}));
                        ui.text(format!("BG Tilemap: {}", if (lock.lcd_control >> 3) != 0 {"0x9C00"} else {"0x9800"}));
                        ui.text(format!("Sprite Size: {}", if (lock.lcd_control >> 2) != 0 {"8x16"} else {"8x8"}));
                        ui.text(format!("Sprites Enabled: {}", (lock.lcd_control >> 1) != 0));
                        ui.text(format!("BG Enabled: {}", (lock.lcd_control & 1) != 0));
                        ui.separator();

                        ui.bullet_text(im_str!("LCD Status"));
                        ui.separator();
                        ui.text(format!("LYC Interrupt: {}", (lock.lcd_stat >> 7) != 0));
                        ui.text(format!("OAM Mode Interrupt: {}", (lock.lcd_stat >> 6) != 0));
                        ui.text(format!("VBlank Mode Interrupt: {}", (lock.lcd_stat >> 5) != 0));
                        ui.text(format!("HBlank Mode Interrupt: {}", (lock.lcd_stat >> 4) != 0));
                        ui.text(format!("Coincidence Flag: {}", (lock.lcd_stat >> 3) != 0));
                        ui.text(format!("Current mode: {}", lock.lcd_stat & 3));
                        ui.separator();

                        ui.bullet_text(im_str!("Various"));
                        ui.separator();
                        ui.text(format!("LY: {}", lock.ly));
                        ui.text(format!("LYC: {}", lock.lyc));
                        ui.separator();

                        ui.bullet_text(im_str!("Full Background:"));
                        ui.same_line(300.0);
                        ui.bullet_text(im_str!("Full Window:"));

                        if emu_started {
                            Image::new(background_id.unwrap(), [256.0, 256.0]).build(&ui);
                            ui.same_line(300.0);
                            Image::new(window_id.unwrap(), [256.0, 256.0]).build(&ui);
                        }
                    });
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

fn get_all_roms() -> Vec<fs::DirEntry> {

    init_dirs();
    let mut all_roms: Vec<fs::DirEntry> = Vec::new();
    let mut read_files: Vec<_> = fs::read_dir("roms").unwrap().map(|r| r.unwrap()).collect();
    read_files.sort_by_key(|dir| dir.path().to_str().unwrap().to_lowercase());
    
    for entry in read_files {
        
        let file_name = entry.file_name().into_string().unwrap();
        
        if file_name.contains(".gb") {
            all_roms.push(entry);
        }
    }

    all_roms
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