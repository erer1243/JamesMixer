use imgui::*;

pub struct UIState {
    // The size of the actual window, used to update imgui window size
    pub window_size: [f32; 2],

    // The percentage volume (0-100) of the 3 inputs
    pub mic_volume: f32,
    pub phone_volume: f32,
    pub music_volume: f32,

    // Currently selected song
    pub current_song: i32,
}

pub fn draw_ui(ui: &mut imgui::Ui, state: &mut UIState, audio: &crate::audio::Audio) {
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
            // 2 Columns for 2 labels
            ui.columns(2, im_str!("##Labels"), false);
            ui.text("Volume Adjustment:");
            ui.next_column();
            ui.text("Music Controls:");

            ui.separator();

            // 3 columns for 3 input sources
            ui.columns(4, im_str!("##Inputs"), false);

            // Microphone volume column
            ui.set_current_column_width(150.);
            ui.text("Microphone");
            VerticalSlider::new(im_str!("##Mic volume"), [100., 300.])
                .range(0.0..=100.0)
                .display_format(im_str!("%.0f%%"))
                .build(ui, &mut state.mic_volume);

            // Phone line volume column
            ui.next_column();
            ui.set_current_column_width(150.);
            ui.text("Phone");
            VerticalSlider::new(im_str!("##Phone volume"), [100., 300.])
                .range(0.0..=100.0)
                .display_format(im_str!("%.0f%%"))
                .build(ui, &mut state.phone_volume);

            // Music volume column
            ui.next_column();
            ui.set_current_column_width(150.);
            ui.text("Music");
            VerticalSlider::new(im_str!("##Music volume"), [100., 300.])
                .range(0.0..=100.0)
                .display_format(im_str!("%.0f%%"))
                .build(ui, &mut state.music_volume);

            // Music controls column
            ui.next_column();
            ui.button(im_str!("Pause"), [80., 30.]);
            ui.same_line(80. + 3. * ui.clone_style().frame_padding[0]);
            ui.button(im_str!("Play"), [80., 30.]);

            // Music selection box
            ui.columns(1, im_str!("##Selection section"), false);
            ui.separator();
            ui.text("Song Selection");

            ui.button(im_str!("Load"), [80., 30.]);

            ChildWindow::new(0).build(ui, || {
                // Setup width for list box
                let xpad = ui.clone_style().frame_padding[0];
                let width = state.window_size[0] - xpad;
                let width_tok = ui.push_item_width(width);

                let song_list = audio.song_list();
                ui.list_box(
                    im_str!("##Song selector"),
                    &mut state.current_song,
                    song_list.as_slice(),
                    song_list.len() as i32,
                );

                // Clear width
                width_tok.pop(ui);
            })
        });
}
