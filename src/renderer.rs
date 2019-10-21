use sdl2;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::video;

use imgui::*;
use imgui_sdl2;
use imgui_opengl_renderer;

use log::error;

use std::io;
use std::io::Read;
use std::fs;
use std::fs::File;
use std::sync::mpsc;
use std::path::PathBuf;

use super::gpu;
use super::emulator;
use super::emulator::InputEvent;

struct State {
    pub emu_running: bool,
    pub rom_selected: bool,
    pub selected_rom: PathBuf,
    pub game_scale: f32,
    pub header_data: HeaderData,
}

struct ImguiSys {
    pub context: imgui::Context,
    pub sdl_imgui: imgui_sdl2::ImguiSdl2,
    pub renderer: imgui_opengl_renderer::Renderer,
}

struct HeaderData {
    title: String,
    publisher: String,
    cart_type: String,
    rom_size: String,
    ram_size: String,
}

pub fn init_renderer() {

    let mut emu_state = State {
        emu_running: false,
        rom_selected: false,
        selected_rom: PathBuf::new(),
        game_scale: 1.0,
        header_data: HeaderData {
            title: String::from(""),
            publisher: String::from(""),
            cart_type: String::from(""),
            rom_size: String::from(""),
            ram_size: String::from(""),
        }
    };
    
    // Init SDL
    let sdl_context = sdl2::init().unwrap();
    let sdl_video = sdl_context.video().unwrap();
    let mut sdl_events = sdl_context.event_pump().unwrap();
    let main_window = sdl_video.window("Rusty Boi - Main Window", 650, 450).position_centered().opengl().resizable().build().unwrap();
    let _gl_context = main_window.gl_create_context().expect("Failed to create OpenGL context");
    gl::load_with(|s| sdl_video.gl_get_proc_address(s) as _);
    sdl_video.gl_set_swap_interval(0).unwrap();

    // Init IMGUI
    let mut imgui = imgui::Context::create();
    let sdl2_imgui = imgui_sdl2::ImguiSdl2::new(&mut imgui, &main_window);
    let imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| sdl_video.gl_get_proc_address(s) as _);
    
    let mut imgui_sys = ImguiSys {
        context: imgui,
        sdl_imgui: sdl2_imgui,
        renderer: imgui_renderer,
    };

    let all_roms = get_all_roms();

    'render_loop: loop {

        if emu_state.emu_running {

            let mut gpu_state = gpu::init_gpu();
            let game_window = sdl_video.window("Rusty Boi - Game Window", 160 * emu_state.game_scale as u32, 144 * emu_state.game_scale as u32)
            .position_centered().vulkan().resizable().build().unwrap();
            let mut game_canvas = game_window.into_canvas().build().unwrap();

            let (input_tx, input_rx) = mpsc::channel();
            let emulator_locks = emulator::initialize(&emu_state.selected_rom);
            let mut update_ui = false;

            game_canvas.set_scale(emu_state.game_scale, emu_state.game_scale).unwrap();
            game_canvas.set_draw_color(Color::RGB(255, 255, 255));
            game_canvas.clear();
            game_canvas.present();

            emulator::start_emulation(&emulator_locks, input_rx);

            'game_loop: loop {

                for event in sdl_events.poll_iter() {

                    imgui_sys.sdl_imgui.handle_event(&mut imgui_sys.context, &event);
                    match event {
                        Event::Window { timestamp: _, window_id, win_event} => {
                            match win_event {
                                WindowEvent::Close => {
                                    emu_state.emu_running = false;
                                    if window_id == game_canvas.window().id() { 
                                        input_tx.send(InputEvent::Quit).unwrap();
                                        break 'game_loop 
                                    } 
                                    else {
                                        break 'render_loop
                                    }
                                },
                                WindowEvent::FocusGained => {
                                    if window_id == game_canvas.window().id() {
                                        update_ui = false;
                                    }
                                    else {
                                        update_ui = true;
                                    }
                                },
                                _ => {},
                            }
                        }
                        Event::KeyDown { keycode: Some(Keycode::A), .. } => { input_tx.send(InputEvent::APressed).unwrap() },
                        Event::KeyDown { keycode: Some(Keycode::S), .. } => { input_tx.send(InputEvent::BPressed).unwrap() },
                        Event::KeyDown { keycode: Some(Keycode::Up), .. } => { input_tx.send(InputEvent::UpPressed).unwrap() },
                        Event::KeyDown { keycode: Some(Keycode::Left), .. } => { input_tx.send(InputEvent::LeftPressed).unwrap() },
                        Event::KeyDown { keycode: Some(Keycode::Right), .. } => { input_tx.send(InputEvent::RightPressed).unwrap() },
                        Event::KeyDown { keycode: Some(Keycode::Down), .. } => { input_tx.send(InputEvent::DownPressed).unwrap() },
                        Event::KeyDown { keycode: Some(Keycode::Return), .. } => { input_tx.send(InputEvent::StartPressed).unwrap() },
                        Event::KeyDown { keycode: Some(Keycode::RShift), .. } => { input_tx.send(InputEvent::SelectPressed).unwrap() },
                        Event::Quit {..} => { emu_state.emu_running = false }
                        _ => {}
                    }
                }

                gpu::gpu_loop(&emulator_locks.cycles_arc, &mut gpu_state, &mut game_canvas, &emulator_locks.gpu);
                if update_ui { ui_loop(&mut imgui_sys, &main_window, &sdl_events.mouse_state(), &all_roms, &mut emu_state) }
                if !emu_state.emu_running { 
                    input_tx.send(InputEvent::Quit).unwrap();
                    break 'game_loop;
                }
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

fn ui_loop(sys: &mut ImguiSys, window: &video::Window, mouse_state: &sdl2::mouse::MouseState, all_roms: &Vec<fs::DirEntry>, emu: &mut State) {

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

                        emu.header_data = parse_header(&file.path());
                        emu.rom_selected = true;
                        emu.selected_rom = file.path();
                    }
                }
            }
            else {
                MenuItem::new(im_str!("No ROMs detected.")).build_with_ref(&imgui_ui, &mut false);
            }
            menu.end(&imgui_ui);
        }
        imgui_ui.separator();

        imgui_ui.text(format!("ROM Title: {}", &emu.header_data.title));
        imgui_ui.text(format!("Publisher: {}", &emu.header_data.publisher));
        imgui_ui.text(format!("Cart Type: {}", &emu.header_data.cart_type));
        imgui_ui.text(format!("ROM Size: {}", &emu.header_data.rom_size));
        imgui_ui.text(format!("RAM Size: {}", &emu.header_data.ram_size));

        imgui_ui.separator();
        if PathBuf::from("Bootrom.gb").exists() {
            imgui_ui.text_colored([0.0, 1.0, 0.0, 1.0], im_str!("Bootrom located, everything's ready"));
            if emu.rom_selected {
                if imgui_ui.button(im_str!("Boot ROM"), [90.0, 20.0]) {
                    emu.rom_selected = false;
                    emu.emu_running = true;
                }
            }
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

    sys.renderer.render(imgui_ui);
    window.gl_swap_window();
}


fn parse_header(file_path: &PathBuf) -> HeaderData {

    let header: HeaderData;
    let mut file = File::open(file_path).unwrap();
    let mut header_buffer = [0; 335];
    file.read(&mut header_buffer).unwrap();

    let game_title = (String::from_utf8(header_buffer[308..323].to_vec()).unwrap().trim_matches(char::from(0))).to_string();

    // TODO: This code can also be in 0144-0145 depending on the release
    // date of the cartridge.
    let lic_code = match header_buffer[331] {

        0x00 => String::from("None"),
        0x01 => String::from("Nintendo R&D 1"),
        0x08 => String::from("Capcom"),
        0x13 => String::from("Electronic Arts"),
        0x18 => String::from("Hudson Soft"),
        0x19 => String::from("b-ai"),
        0x20 => String::from("kss"),
        0x22 => String::from("pow"),
        0x24 => String::from("PCM Complete"),
        0x25 => String::from("san-z"),
        0x28 => String::from("Kemco Japan"),
        0x29 => String::from("seta"),
        0x30 => String::from("Viacom"),
        0x31 => String::from("Nintendo"),
        0x32 => String::from("Bandai"),
        // On 014B, it shows that the code is on 0144. On 0x144 it's Ocean/Acclaim
        0x33 => String::from("New Licensee"),
        0x34 => String::from("Konami"),
        0x35 => String::from("Hector"),
        0x37 => String::from("Taito"),
        0x38 => String::from("Hudson"),
        0x39 => String::from("Banpresto"),
        0x41 => String::from("Ubi Soft"),
        0x42 => String::from("Atlus"),
        0x44 => String::from("Malibu"),
        0x46 => String::from("angel"),
        0x47 => String::from("Bullet-Proof"),
        0x49 => String::from("irem"),
        0x50 => String::from("Absolute"),
        0x51 => String::from("Acclaim"),
        0x52 => String::from("Activision"),
        0x53 => String::from("American sammy"),
        0x54 => String::from("Konami"),
        0x55 => String::from("Hi tech entertainment"),
        0x56 => String::from("LJN"),
        0x57 => String::from("Matchbox"),
        0x58 => String::from("Mattel"),
        0x59 => String::from("Milton Bradley"),
        0x60 => String::from("Titus"),
        0x61 => String::from("Virgin"),
        0x64 => String::from("LucasArts"),
        0x67 => String::from("Ocean"),
        0x69 => String::from("Electronic Arts"),
        0x70 => String::from("Infogrames"),
        0x71 => String::from("Interplay"),
        0x72 => String::from("Broderbund"),
        0x73 => String::from("sculptured"),
        0x75 => String::from("sci"),
        0x78 => String::from("THQ"),
        0x79 => String::from("Accolade"),
        0x80 => String::from("misawa"),
        0x83 => String::from("Iozc"),
        0x86 => String::from("tokuma shoten i*"),
        0x87 => String::from("tsukuda ori*"),
        0x91 => String::from("Chunsoft"),
        0x92 => String::from("Video system"),
        0x93 => String::from("Ocean/Acclaim"),
        0x95 => String::from("Varie"),
        0x96 => String::from("Yonezawa/s'pal"),
        0x97 => String::from("Kaneko"),
        0x99 => String::from("Pack in soft"),
        0xA4 => String::from("Konami (Yu-Gi-Oh!)"),
        _ => String::from("Unknown"),
    };

    let mbc_type = match header_buffer[327] {
        0x00 => String::from("No MBC"),
        0x01 => String::from("MBC1"),
        0x02 => String::from("MBC1 with RAM"),
        0x03 => String::from("MBC1 with RAM and battery"),
        0x05 => String::from("MBC2"),
        0x06 => String::from("MBC2 with battery"),
        0x08 => String::from("ROM with RAM"),
        0x09 => String::from("ROM with RAM and battery"),
        0x0B => String::from("MMM01"),
        0x0C => String::from("MMM01 with RAM"),
        0x0D => String::from("MMM01 with RAM and battery"),
        0x0F => String::from("MBC3 with timer and battery"),
        0x11 => String::from("MBC3"),
        0x12 => String::from("MBC3 with RAM"),
        0x13 => String::from("MBC3 with RAM and battery"),
        _ => String::from("Unknown"),
    };

    let rom_size = format!("{}KB", 32 << header_buffer[328]);
    let ram_size = match header_buffer[329] {
        0x00 => String::from("None"),
        0x01 => String::from("2KB"),
        0x02 => String::from("8KB"),
        0x03 => String::from("32KB"),
        0x04 => String::from("128KB"),
        0x05 => String::from("64KB"),
        _ => String::from("Unknown"),
    };

    header = HeaderData {
        title: game_title,
        publisher: lic_code,
        cart_type: mbc_type,
        rom_size: rom_size,
        ram_size: ram_size,
    };
    header
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
                io::ErrorKind::PermissionDenied => {error!("Failed to create ROMs directory: Permission Denied")},
                _ => {},
            }
        }
    }
}