use eframe::egui;
use egui::{Button, Label, Slider, Ui};
use rfd::FileDialog;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io::BufReader, path::PathBuf};

pub fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    let app = App::default();
    eframe::run_native("wav-wav", options, Box::new(|_cc| Ok(Box::new(app))))
}

struct App {
    input_audio_component: InputAudioComponent,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input_audio_component: InputAudioComponent::new(),
        }
    }
}

struct InputAudioComponent {
    audio_path: Option<PathBuf>,
    audio_pos: f32,
    audio_duration: f32,
    sink: Option<Arc<Mutex<Sink>>>,
    is_paused: bool,
}

impl InputAudioComponent {
    fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).expect("Failed to create sink");
        std::mem::forget(stream); // Keep stream alive
        Self {
            audio_path: None,
            audio_pos: 0.,
            audio_duration: 0.001,
            sink: Some(Arc::new(Mutex::new(sink))),
            is_paused: true,
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [40.0, 40.0],
                    Button::new(if self.is_paused { "▶" } else { "⏸" }),
                )
                .clicked()
            {
                if let Some(sink) = self.sink.clone() {
                    let sink = sink.lock().unwrap();
                    if sink.is_paused() {
                        sink.play();
                        self.is_paused = false;
                    } else if sink.empty() {
                        if let Some(path) = self.audio_path.clone() {
                            let file = File::open(path).expect("Failed to open file");
                            let source =
                                Decoder::new(BufReader::new(file)).expect("Failed to decode file");
                            sink.append(source);
                        }
                    } else {
                        sink.pause();
                        self.is_paused = true;
                    }
                }
                // todo : replay if finished
            }

            ui.add_sized(
                [ui.available_width() - 100.0 - 10.0, 40.],
                Label::new(match &self.audio_path {
                    None => "Файл не выбран",
                    Some(path_buf) => path_buf
                        .file_name()
                        .expect("Path buf is None")
                        .to_str()
                        .expect("Path buf string is None"),
                }),
            );

            if ui
                .add_sized([100.0, 40.0], Button::new("Выбрать файл"))
                .clicked()
            {
                if let Some(path) = FileDialog::new().pick_file() {
                    self.audio_path = Some(path);
                    let file = File::open(self.audio_path.clone().expect("No path"))
                        .expect("Failed to open file");
                    let source = Decoder::new(BufReader::new(file)).expect("Failed to decode file");

                    self.audio_duration =
                        source.total_duration().expect("No duration").as_secs_f32();

                    if let Some(sink) = self.sink.clone() {
                        let sink = sink.lock().unwrap();
                        sink.stop();
                        sink.pause();
                        sink.append(source);
                    }
                }
            }
        });

        ui.ctx().request_repaint();
        ui.spacing_mut().slider_width = ui.available_width() - 100.;
        // ui.style_mut().spacing.slider_width = ui.available_width() - 70.0;
        // ui.spacing_mut().slider_rail_height = 100.;
        if ui
            .add(
                Slider::new(&mut self.audio_pos, 0.0..=self.audio_duration)
                    .custom_formatter(|n, _| {
                        let n = n as i32;
                        let mins = (n / 60) % 60;
                        let secs = n % 60;
                        format!("{mins:02}:{secs:02}")
                    })
                    .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 0.5 })
                    .trailing_fill(true),
            )
            .changed()
        {
            if let Some(sink) = self.sink.clone() {
                let sink = sink.lock().unwrap();

                match sink.try_seek(Duration::from_secs_f32(self.audio_pos)) {
                    Ok(()) => ui.label("GOOOOD"),
                    Err(e) => ui.label(format!("{e:?}")),
                    _ => ui.label("WTF"),
                };
            }
        }
        if let Some(sink) = self.sink.clone() {
            let sink = sink.lock().unwrap();
            if !sink.empty() {
                self.audio_pos = sink.get_pos().as_secs_f32();
            } else {
                self.is_paused = true;
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.input_audio_component.show(ui);
        });
        // ui.horizontal(|ui| {
        //     ui.vertical(|ui| {
        //         if ui.button("Выбрать файл").clicked() {
        //             if let Some(path) = FileDialog::new().pick_file() {
        //                 self.input_audio_path = Some(path);
        //             }
        //         };
        //     });
        //
        //     if ui.button("Play/Pause").clicked() {
        //         todo!("Play button");
        //     }
        //     ui.spacing_mut().slider_width = ui.available_width() - 100.;
        //     ui.add(
        //         egui::Slider::new(&mut self.input_audio_pos, 0.0..=120.0).custom_formatter(
        //             |n, _| {
        //                 let n = n as i32;
        //                 let mins = (n / 60) % 60;
        //                 let secs = n % 60;
        //                 format!("{mins:02}:{secs:02}/02:00")
        //             },
        //         ),
        //     );
        // });
        //
        // ui.horizontal(|ui| {
        //     ui.vertical(|ui| {
        //         ui.checkbox(&mut self.decode_cycle, "Искать цикл");
        //
        //         ui.horizontal(|ui| {
        //             if ui.button("<").clicked() {
        //                 if self.decode_bits > 1 {
        //                     self.decode_bits -= 1;
        //                 }
        //             }
        //             ui.label(format!("{} bit", self.decode_bits));
        //             if ui.button(">").clicked() {
        //                 if self.decode_bits < 8 {
        //                     self.decode_bits += 1;
        //                 }
        //             }
        //         });
        //
        //         if ui.button("Расшифровать").clicked() {
        //             todo!("Функция расшифровки, с записью в self.message");
        //         }
        //     });
        //
        //     // let third_height = f32::min(200.0, ui.available_height() / 3.0);
        //     ui.add_sized(
        //         [ui.available_width(), 200.0],
        //         egui::TextEdit::multiline(&mut self.message),
        //     );
        // });
        //
        // ui.horizontal(|ui| {
        //     ui.vertical(|ui| {
        //         if ui.button("Сохранить").clicked() {
        //             if let Some(path) = FileDialog::new().save_file() {
        //                 self.output_audio_path = Some(path);
        //             }
        //         };
        //     });
        //
        //     if ui.button("Play/Pause").clicked() {
        //         todo!("Play button");
        //     }
        //     ui.spacing_mut().slider_width = ui.available_width() - 100.;
        //     ui.add(
        //         egui::Slider::new(&mut self.output_audio_pos, 0.0..=120.0).custom_formatter(
        //             |n, _| {
        //                 let n = n as i32;
        //                 let mins = (n / 60) % 60;
        //                 let secs = n % 60;
        //                 format!("{mins:02}:{secs:02}/02:00")
        //             },
        //         ),
        //     );
        // });
    }
}
