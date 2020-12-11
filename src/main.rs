mod audio;
mod ui;

use glium::{glutin, Surface};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    // Load music
    // =============================================================================================
    let audio = audio::Audio::init();

    // Make window
    // =============================================================================================
    let event_loop = glutin::event_loop::EventLoop::new();
    let display = {
        let context = glutin::ContextBuilder::new().with_vsync(true);
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("James' Mixer")
            .with_resizable(false)
            .with_inner_size(glutin::dpi::PhysicalSize {
                width: 800,
                height: 800,
            });

        glium::Display::new(window_builder, context, &event_loop).unwrap()
    };

    // Evaluates to the value of the window, useful in a few places
    #[rustfmt::skip]
    macro_rules! window { () => { display.gl_window().window() }; }

    // Make imgui
    // =============================================================================================
    let mut imgui = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    let mut renderer = imgui_glium_renderer::Renderer::init(&mut imgui, &display).unwrap();

    // Disable saving imgui data to ini file
    imgui.set_ini_filename(None);

    // Setup enlarged font size
    imgui.fonts().clear();
    imgui.fonts().add_font(&[imgui::FontSource::TtfData {
        data: &std::fs::read("font.ttf").unwrap(),
        size_pixels: 22.,
        config: None,
    }]);
    renderer.reload_font_texture(&mut imgui).unwrap();

    // Attach to window
    platform.attach_window(
        imgui.io_mut(),
        &display.gl_window().window(),
        imgui_winit_support::HiDpiMode::Locked(1.0),
    );

    // Init ui state
    // =============================================================================================
    let mut ui_state = ui::UIState {
        window_size: [0.; 2],
        mic_volume: 10.,
        line_volume: 10.,
        song_volume: 10.,
        selected_song: 0,
        jump_time: [0; 2],
        loaded_song: imgui::ImString::new("Load song below"),
    };

    // Previous frame (pf) start time
    let mut pf_start = Instant::now();

    // Main loop
    // =============================================================================================
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            // FPS limiting
            let min_frame_time = Duration::from_secs_f32(1. / 60.);
            let pf_duration = Instant::now() - pf_start;
            if Instant::now() - pf_start < min_frame_time {
                sleep(min_frame_time - pf_duration);
            }
            pf_start = Instant::now();

            // Set ui state window size
            let size = window!().inner_size();
            ui_state.window_size = [size.width as f32, size.height as f32];

            // Do imgui drawing
            let mut ui = imgui.frame();
            platform.prepare_render(&ui, window!());
            ui::draw_ui(&mut ui, &mut ui_state, &audio);

            // Render imgui ui to window
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.render(&mut target, ui.render()).unwrap();
            target.finish().unwrap();
        }

        // Request a new frame after one is completed
        Event::MainEventsCleared => {
            platform.prepare_frame(imgui.io_mut(), window!()).unwrap();
            window!().request_redraw();
        }

        // Quit on window close request
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }

        ev => platform.handle_event(imgui.io_mut(), window!(), &ev),
    });
}
