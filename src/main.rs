use eframe::egui;
use egui::{Button, RichText};
use std::path::PathBuf;

struct App {
    counter: usize,
    input_audio_path: Option<PathBuf>,
    output_audio_path: Option<PathBuf>,
    message: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            counter: 0,
            input_audio_path: None,
            output_audio_path: None,
            message: String::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("Hello, egui!");

            if ui.button("Click me!").clicked() {
                self.counter += 1;
            }

            if ui
                .add_sized(
                    [120.0, 50.0],
                    Button::new(RichText::new("Click me").size(20.0)),
                )
                .clicked()
            {
                self.counter += 1;
            }
            ui.label(format!("Button clicked: {} times", self.counter));

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .animated(false)
                .show(ui, |ui| {
                    ui.add_sized(
                        [ui.available_width(), 200.0],
                        egui::TextEdit::multiline(&mut self.message),
                    );
                });

            ui.label(format!("Message: {}", self.message));
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    let app = App::default();
    eframe::run_native("wav-wav", options, Box::new(|_cc| Ok(Box::new(app))))
}
