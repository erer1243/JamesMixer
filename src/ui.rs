use crate::audio::Audio;
use imgui::*;

pub struct UIState {
    // The size of the actual window, used to update imgui window size
    pub window_size: [f32; 2],

    // The percentage volume (0-100+) of the 3 inputs
    pub mic_volume: f32,
    pub line_volume: f32,
    pub song_volume: f32,

    // Currently selected song list index
    pub selected_song: i32,

    // Jump-to-time target
    pub jump_time: [i32; 2],

    // Currently loaded song name
    pub loaded_song: ImString,
}

pub fn draw_ui(ui: &mut imgui::Ui, state: &mut UIState, audio: &Audio) {
    Window::new(im_str!("main window"))
        // Disable window title, scrollbar etc
        .no_decoration()
        // Always the shape of the actual window
        .size(state.window_size, Condition::Always)
        .position([0., 0.], Condition::Always)
        // Background color is set in main loop with window clear color
        .draw_background(false)
        // Content within ui
        .build(ui, || {
            // Top labels
            // =====================================================================================
            // 2 Columns for 2 labels
            ui.columns(2, im_str!("##Labels"), false);
            ui.text("Volume Adjustment:");
            ui.next_column();
            ui.text("Music Controls:");

            ui.separator();

            // Volume columns
            // =====================================================================================
            // 4 columns for 3 input volume + 1 music control column
            ui.columns(4, im_str!("##Inputs and Controls"), false);

            // Microphone volume column
            // =====================================================================================
            ui.set_current_column_width(150.);
            ui.text("Microphone");
            let changed = VerticalSlider::new(im_str!("##Mic volume"), [100., 300.])
                .range(0.0..=500.0)
                .flags(SliderFlags::LOGARITHMIC)
                .display_format(im_str!("%.0f%%"))
                .build(ui, &mut state.mic_volume);

            if changed {
                audio.set_mic_volume(state.mic_volume);
            }

            // Line in volume column
            // =====================================================================================
            ui.next_column();
            ui.set_current_column_width(150.);
            ui.text("Line in");
            let changed = VerticalSlider::new(im_str!("##Line in volume"), [100., 300.])
                .range(0.0..=500.0)
                .flags(SliderFlags::LOGARITHMIC)
                .display_format(im_str!("%.0f%%"))
                .build(ui, &mut state.line_volume);

            if changed {
                audio.set_line_volume(state.line_volume);
            }

            // Music volume column
            // =====================================================================================
            ui.next_column();
            ui.set_current_column_width(150.);
            ui.text("Music");
            let changed = VerticalSlider::new(im_str!("##Music volume"), [100., 300.])
                .range(0.0..=1000.0)
                .flags(SliderFlags::LOGARITHMIC)
                .display_format(im_str!("%.0f%%"))
                .build(ui, &mut state.song_volume);

            if changed {
                audio.set_song_volume(state.song_volume);
            }

            // Music controls column
            // =====================================================================================
            ui.next_column();
            if ui.button(im_str!("Pause"), [80., 30.]) {
                audio.set_paused(true);
            }

            ui.same_line(80. + 3. * ui.clone_style().frame_padding[0]);
            if ui.button(im_str!("Play"), [80., 30.]) {
                audio.set_paused(false);
            }

            // Draw loaded song
            ui.text("Loaded song:");
            ui.same_line(
                ui.calc_text_size(im_str!("Loaded song:"), false, 0.0)[0]
                    + 3. * ui.clone_style().frame_padding[0],
            );
            ui.text(&state.loaded_song);

            // Draw paused/playing
            ui.text(if audio.get_paused() {
                "Status: Paused"
            } else {
                "Status: Playing"
            });

            // Draw timestamp
            let ((ts_m, ts_s), (mt_m, mt_s)) = audio.music_timestamp();
            ui.text(format!(
                "Timestamp: {:02}:{:02} / {:02}:{:02}",
                ts_m, ts_s, mt_m, mt_s
            ));

            let (samples, max_samples) = audio.music_samples();
            let (mut samples, max_samples) = (samples as u64, max_samples as u64);
            Slider::new(im_str!("##Timestamp slider"))
                .range(0..=max_samples)
                .display_format(im_str!(""))
                .build(ui, &mut samples);

            // Draw jump-to-time
            if ui.button(im_str!("Jump to"), [80., 30.]) && state.jump_time != [0, 0] {
                audio.jump_song(state.jump_time[0] as usize, state.jump_time[1] as usize);
                state.jump_time = [0; 2];
            }

            ui.same_line(80. + 3. * ui.clone_style().frame_padding[0]);

            let width_tok = ui.push_item_width(50.);
            InputInt2::new(ui, im_str!("##Jump time"), &mut state.jump_time).build();
            width_tok.pop(ui);

            // Music selection box
            // =====================================================================================
            let song_list = audio.song_list();
            ui.columns(1, im_str!("##Selection section"), false);
            ui.separator();
            ui.text("Song Selection");

            // Load song button
            if ui.button(im_str!("Load"), [80., 30.]) {
                let song_name = song_list[state.selected_song as usize];

                // Tell audio system to load song
                audio.load_song(song_name);

                // Display loaded song in controls column
                state.loaded_song = song_name.to_owned();
            }

            ChildWindow::new(0).build(ui, || {
                // Setup width for list box
                let xpad = ui.clone_style().frame_padding[0];
                let width = state.window_size[0] - xpad;
                let width_tok = ui.push_item_width(width);

                ui.list_box(
                    im_str!("##Song selector"),
                    &mut state.selected_song,
                    song_list.as_slice(),
                    song_list.len() as i32,
                );

                // Clear width
                width_tok.pop(ui);
            })
        });
}
