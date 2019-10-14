mod cpu;
mod gpu;
mod timer;
mod memory;
mod register;

mod utils;
mod emulator;

mod opcodes;
mod opcodes_prefixed;

use emulator::init_emu;

use log::info;
use log::error;

use sdl2::event::Event;

use imgui::*;
use imgui_sdl2;
use imgui_opengl_renderer;

use std::io;
use std::fs;
use std::path::Path;


fn main() {

    simple_logger::init_with_level(log::Level::Info).unwrap();
    info!("Rusty Boi");

    let sdl_ctx = sdl2::init().unwrap();
    let sdl_video = sdl_ctx.video().unwrap();
    let main_window = sdl_video.window("Rusty Boi", 600, 400).position_centered().opengl().build().unwrap();
    let _gl_context = main_window.gl_create_context().expect("Failed to create OpenGL context");
    gl::load_with(|s| sdl_video.gl_get_proc_address(s) as _);

    let mut imgui = imgui::Context::create();
    let mut sdl2_imgui = imgui_sdl2::ImguiSdl2::new(&mut imgui, &main_window);
    let imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| sdl_video.gl_get_proc_address(s) as _);
    let mut sdl_events = sdl_ctx.event_pump().unwrap();

    let mut scale_factor = 1.0;
    let all_roms = get_all_roms();
    let mut _game_running = false;

    'main: loop {

        for event in sdl_events.poll_iter() {
            
            sdl2_imgui.handle_event(&mut imgui, &event);
            if !sdl2_imgui.ignore_event(&event) {continue};

            match event {
                Event::Quit {..} => { break 'main },/*
                Event::KeyDown { keycode: Some(Keycode::A), .. } => { input_tx.send(InputEvent::APressed).unwrap() },
                Event::KeyUp  { keycode: Some(Keycode::A), .. } => { input_tx.send(InputEvent::AReleased).unwrap() },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => { input_tx.send(InputEvent::BPressed).unwrap() },
                Event::KeyUp  { keycode: Some(Keycode::S), .. } => { input_tx.send(InputEvent::BReleased).unwrap() },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => { input_tx.send(InputEvent::UpPressed).unwrap() },
                Event::KeyUp  { keycode: Some(Keycode::Up), .. } => { input_tx.send(InputEvent::UpReleased).unwrap() },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => { input_tx.send(InputEvent::LeftPressed).unwrap() },
                Event::KeyUp  { keycode: Some(Keycode::Left), .. } => { input_tx.send(InputEvent::LeftReleased).unwrap() },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => { input_tx.send(InputEvent::RightPressed).unwrap() },
                Event::KeyUp  { keycode: Some(Keycode::Right), .. } => { input_tx.send(InputEvent::RightReleased).unwrap() },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => { input_tx.send(InputEvent::DownPressed).unwrap() },
                Event::KeyUp  { keycode: Some(Keycode::Down), .. } => { input_tx.send(InputEvent::DownReleased).unwrap() },
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => { input_tx.send(InputEvent::StartPressed).unwrap() },
                Event::KeyUp  { keycode: Some(Keycode::Return), .. } => { input_tx.send(InputEvent::StartReleased).unwrap() },
                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => { input_tx.send(InputEvent::SelectPressed).unwrap() },
                Event::KeyUp  { keycode: Some(Keycode::Backspace), .. } => { input_tx.send(InputEvent::SelectReleased).unwrap() },*/
                _ => {}
            }
        }

        sdl2_imgui.prepare_frame(imgui.io_mut(), &main_window, &sdl_events.mouse_state());

        let imgui_ui = imgui.frame();

        Window::new(im_str!("Main Window"))
        .size([300.0, 300.0], Condition::Always)
        .build(&imgui_ui, || {
            if let Some(menu) = imgui_ui.begin_menu(im_str!("Installed ROMs"), true) {
                if all_roms.len() > 0 {

                    for file in all_roms.iter() {
                        let filename = ImString::new(file.file_name().into_string().unwrap());

                        if MenuItem::new(&filename).build_with_ref(&imgui_ui, &mut false) { 

                            if Path::new("Bootrom.gb").exists() {
                                let game_window = sdl_video.window("Rusty Boi - Game Window", 160 * scale_factor as u32, 144 * scale_factor as u32)
                                .position_centered().build().unwrap();
                                let mut game_canvas = game_window.into_canvas().present_vsync().build().unwrap();
                                game_canvas.set_scale(scale_factor, scale_factor).unwrap();
                                init_emu(file);
                            }
                        };
                    }
                }
                else {
                    MenuItem::new(im_str!("No ROMs detected.")).build_with_ref(&imgui_ui, &mut false);
                }
                menu.end(&imgui_ui);
            }
            imgui_ui.separator();
            if Path::new("Bootrom.gb").exists() {
                imgui_ui.text_colored([0.0, 1.0, 0.0, 1.0], im_str!("Bootrom located, everything's ready"));
            }
            else {
                imgui_ui.text_colored([1.0, 0.0, 0.0, 1.0], im_str!("Can't locate Bootrom!"));
            }
            imgui_ui.separator();
            Slider::new(im_str!("Scale factor"), 1.0 ..= 10.0).display_format(im_str!("%.0f")).build(&imgui_ui, &mut scale_factor);
        });

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        sdl2_imgui.prepare_render(&imgui_ui, &main_window);
        imgui_renderer.render(imgui_ui);
        main_window.gl_swap_window();
    }
}

fn get_all_roms() -> Vec<fs::DirEntry> {

    init_dirs();
    let mut all_roms: Vec<fs::DirEntry> = Vec::new();
    
    for entry in fs::read_dir("roms").unwrap() {
        
        let file = entry.unwrap();
        let file_name = file.file_name().into_string().unwrap();
        
        if file_name.contains(".gb") {
            all_roms.push(file);
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
                io::ErrorKind::PermissionDenied => {error!("Failed to create ROMs directory: Permission Denied")},
                _ => {},
            }
        }
    }
}