use eframe::egui;
use egui::{Button, RichText};

struct App {
    counter: usize,
}

impl Default for App {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, egui!");
            if ui.button("Click me!").clicked() {
                self.counter += 1;
            }

            if ui.add_sized([120.0, 50.0], Button::new(RichText::new("Click me").size(20.0))).clicked() {
                    self.counter += 1;
                }
            ui.label(format!("Button clicked: {} times", self.counter));
        });
    } 
}


fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("My egui app", options, Box::new(|_cc| Ok(Box::new(App::default()))))
}
