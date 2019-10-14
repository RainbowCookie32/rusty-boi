use sdl2;
use sdl2::event::Event;
use sdl2::pixels::Color;

use imgui::*;
use imgui_sdl2;
use imgui_opengl_renderer;

use log::error;

use std::io;
use std::fs;
use std::path::PathBuf;

use super::gpu;
use super::emulator;

struct State {
    pub emu_running: bool,
    pub booted_rom: PathBuf,
    pub game_scale: f32,
}

struct ImguiSys {
    pub context: imgui::Context,
    pub sdl_imgui: imgui_sdl2::ImguiSdl2,
    pub renderer: imgui_opengl_renderer::Renderer,
}

pub fn init_renderer() {

    let mut emu_state = State {
        emu_running: false,
        booted_rom: PathBuf::new(),
        game_scale: 1.0,
    };
    
    // Init SDL
    let sdl_context = sdl2::init().unwrap();
    let sdl_video = sdl_context.video().unwrap();
    let mut sdl_events = sdl_context.event_pump().unwrap();
    let main_window = sdl_video.window("Rusty Boi - Main Window", 600, 400).position_centered().opengl().build().unwrap();
    let _gl_context = main_window.gl_create_context().expect("Failed to create OpenGL context");
    gl::load_with(|s| sdl_video.gl_get_proc_address(s) as _);

    // Init IMGUI
    let mut imgui = imgui::Context::create();
    let sdl2_imgui = imgui_sdl2::ImguiSdl2::new(&mut imgui, &main_window);
    let imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| sdl_video.gl_get_proc_address(s) as _);
    
    let mut imgui_sys = ImguiSys {
        context: imgui,
        sdl_imgui: sdl2_imgui,
        renderer: imgui_renderer
    };

    let all_roms = get_all_roms();

    'render_loop: loop {

        if emu_state.emu_running {

            let mut gpu_state = gpu::init_gpu();
            let game_window = sdl_video.window("Rusty Boi - Game Window", 160 * emu_state.game_scale as u32, 144 * emu_state.game_scale as u32)
            .position_centered().build().unwrap();
            let mut game_canvas = game_window.into_canvas().present_vsync().build().unwrap();

            let emulator_locks = emulator::initialize(&emu_state.booted_rom);

            game_canvas.set_scale(emu_state.game_scale, emu_state.game_scale).unwrap();
            game_canvas.set_draw_color(Color::RGB(255, 255, 255));
            game_canvas.clear();
            game_canvas.present();

            emulator::start_emulation(&emulator_locks);

            'game_loop: loop {

                for event in sdl_events.poll_iter() {

                    imgui_sys.sdl_imgui.handle_event(&mut imgui_sys.context, &event);
                    match event {
                        Event::Quit {..} => { emu_state.emu_running = false }
                        _ => {}
                    }
                }

                gpu::gpu_loop(&emulator_locks.cycles_arc, &mut gpu_state, &mut game_canvas, &emulator_locks.gpu);
                ui_loop(&mut imgui_sys, &main_window, &sdl_events.mouse_state(), &all_roms, &mut emu_state);
                if !emu_state.emu_running {break 'game_loop}
            }
        }
        else {

            loop {

                for event in sdl_events.poll_iter() {

                    imgui_sys.sdl_imgui.handle_event(&mut imgui_sys.context, &event);
                    match event {
                        Event::Quit {..} => { break 'render_loop }
                        _ => {}
                    }
                }
                ui_loop(&mut imgui_sys, &main_window, &sdl_events.mouse_state(), &all_roms, &mut emu_state);
                if emu_state.emu_running {break;}
            }
        }
    }

}

fn ui_loop(sys: &mut ImguiSys, window: &sdl2::video::Window, mouse_state: &sdl2::mouse::MouseState, all_roms: &Vec<fs::DirEntry>, emu: &mut State) {

    sys.sdl_imgui.prepare_frame(sys.context.io_mut(), window, mouse_state);
    let imgui_ui = sys.context.frame();

    Window::new(im_str!("Rusty Boi - Main Window"))
    .size([300.0, 350.0], Condition::Always)
    .build(&imgui_ui, || {
        if let Some(menu) = imgui_ui.begin_menu(im_str!("Detected ROMs"), true) {
            if all_roms.len() > 0 && !emu.emu_running {

                for file in all_roms.iter() {
                    let filename = ImString::new(file.file_name().into_string().unwrap());

                    if MenuItem::new(&filename).build_with_ref(&imgui_ui, &mut false) { 

                        if PathBuf::from("Bootrom.gb").exists() && !emu.emu_running {
                            emu.emu_running = true;
                            emu.booted_rom = file.path();
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
        if PathBuf::from("Bootrom.gb").exists() {
            imgui_ui.text_colored([0.0, 1.0, 0.0, 1.0], im_str!("Bootrom located, everything's ready"));
        }
        else {
            imgui_ui.text_colored([1.0, 0.0, 0.0, 1.0], im_str!("Can't locate Bootrom!"));
        }
        imgui_ui.separator();
        Slider::new(im_str!("Scale factor"), 1.0 ..= 10.0).display_format(im_str!("%.0f")).build(&imgui_ui, &mut emu.game_scale);
    });

    unsafe {
        gl::ClearColor(0.2, 0.2, 0.2, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    sys.sdl_imgui.prepare_render(&imgui_ui, &window);
    sys.renderer.render(imgui_ui);
    window.gl_swap_window();
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