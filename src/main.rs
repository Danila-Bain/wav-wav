#![windows_subsystem = "windows"]

use rfd::FileDialog;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use slint::ComponentHandle;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

slint::include_modules!();

mod bit_iterator;
mod prefix_function;

lazy_static::lazy_static!(
    static ref tmp_file: NamedTempFile = NamedTempFile::new().expect("Failed to create a temporary file");
);

struct Player {
    sink: Sink,
    data: Vec<i16>,
    path: Option<PathBuf>,
}

impl Player {
    fn new(stream_handle: &OutputStreamHandle) -> Self {
        let sink = Sink::try_new(&stream_handle).expect("Failed to create sink");
        let data = Vec::new();
        Player {
            sink,
            data,
            path: None,
        }
    }

    fn reload(&mut self) {
        if let Some(path) = &self.path {
            let file = File::open(&path).expect("Failed to open the file for playback");
            let source =
                Decoder::new(BufReader::new(file)).expect("Failed to decode the the file into wav");
            self.sink.clear();
            self.sink.append(source);
            self.sink.pause();
            let _ = self.sink.try_seek(Duration::from_secs_f32(0.));
        }
    }

    fn load(&mut self, path: PathBuf) -> (f32, slint::SharedString, slint::Image) {
        self.path = Some(path.clone());
        self.reload();

        let mut wav_reader =
            hound::WavReader::open(&path).expect("Failed to open the file for data");
        let filename = path
            .file_name()
            .expect("Failed to convert the path to filename to display");

        let duration = wav_reader.duration() as f32 / wav_reader.spec().sample_rate as f32;
        self.data = wav_reader.samples::<i16>().filter_map(Result::ok).collect();

        let width = 2000;
        let height = 160;

        let mut pixel_buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
        {
            let width = width as usize;
            let height = height as usize;

            let bytes = pixel_buffer.make_mut_bytes();

            let max = self.data.iter().map(|y| y.abs()).max().unwrap_or(0) as f32;

            for (xi, chunk) in self
                .data
                .chunks(self.data.len() / width)
                .enumerate()
                .take(width)
            {
                let y = (chunk.iter().map(|y| y.abs()).max().unwrap_or(0) as f32) / (max + 1.);
                let y = y.clamp(0., 1.);
                let y = (1. - y.sqrt() * 0.98) * (height - 1) as f32; // flip and scale
                let y = y as usize;
                // for yi in (y - border_width.min(y))..y {
                //     bytes[yi*width*4 + xi*4 + 3] = 255;
                // }
                for yi in y..height {
                    for channel in 0..4 {
                        bytes[yi * width * 4 + xi * 4 + channel] = 255;
                    }
                }
            }
        }

        let image = slint::Image::from_rgba8(pixel_buffer);

        return (
            duration,
            filename
                .to_str()
                .unwrap_or("< Не удалось отобразить название файла >")
                .into(),
            image,
        );
    }

    fn seek(&mut self, new_pos: f32) {
        if self.sink.empty() {
            self.reload();
        }

        self.sink
            .try_seek(Duration::from_secs_f32(new_pos))
            .expect("Seek failed");
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let logical_size = slint::LogicalSize::new(720.0, 720.0);
    let physical_size = logical_size.to_physical(ui.window().scale_factor());
    ui.window().set_size(physical_size); // don't wait for "Set Size" to be clicked; set the size now!

    let (_stream, stream_handle) = OutputStream::try_default().expect("Failed to open an output audio stream");

    let input_player = Arc::new(Mutex::new(Player::new(&stream_handle)));
    let output_player = Arc::new(Mutex::new(Player::new(&stream_handle)));

    ui.on_choose_audio_file({
        let weak_ui = ui.as_weak();
        let input_player = Arc::clone(&input_player);
        move || {
            let input_player = Arc::clone(&input_player);
            let _ = weak_ui.upgrade_in_event_loop(move |ui| {
                if let Some(path) = FileDialog::new()
                    .add_filter("wav files", &["wav"])
                    .pick_file()
                    && let Ok(mut input_player) = input_player.lock()
                {
                    let (duration, filename, image) = input_player.load(path.into());
                    ui.set_input_filename(filename.into());
                    ui.set_input_duration(duration);
                    ui.set_input_waveform(image);
                };
            });
        }
    });

    ui.on_save_audio_file({
        let input_player = Arc::clone(&input_player);
        let output_player = Arc::clone(&output_player);
        let weak_ui = ui.as_weak();
        move || {
            if let Ok(output_player) = output_player.lock()
                && let Ok(input_player) = input_player.lock()
                && !output_player.data.is_empty()
                && let Some(path) = FileDialog::new()
                    .set_file_name(
                        input_player
                            .path
                            .clone()
                            .unwrap_or_default()
                            .file_name()
                            .to_owned()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                    )
                    .save_file()
                && let Ok(_) = std::fs::copy(&*tmp_file, &path)
                && let Some(filename) = path.file_name()
            {
                let filename = filename
                    .to_str()
                    .unwrap_or("< Нечитаемое название файла >")
                    .into();
                let _ = weak_ui.upgrade_in_event_loop(move |ui| {
                    ui.set_output_filename(filename);
                });
            } else {
                let _ = weak_ui.upgrade_in_event_loop(move |ui| {
                    ui.set_output_filename("< Ошибка сохранения файла >".into());
                });
            };
        }
    });

    ui.on_input_play_toggle({
        let input_player = Arc::clone(&input_player);
        move || -> bool {
            if let Ok(mut input_player) = input_player.lock() {
                if input_player.sink.empty() {
                    input_player.reload();
                }
                let is_paused = input_player.sink.is_paused();
                match is_paused {
                    true => {
                        input_player.sink.play();
                        return true;
                    }
                    false => {
                        input_player.sink.pause();
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
    });

    ui.on_output_play_toggle({
        let output_player = Arc::clone(&output_player);
        move || -> bool {
            if let Ok(mut output_player) = output_player.lock() {
                if output_player.sink.empty() {
                    output_player.reload();
                }
                let is_paused = output_player.sink.is_paused();
                match is_paused {
                    true => {
                        output_player.sink.play();
                        return true;
                    }
                    false => {
                        output_player.sink.pause();
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
    });

    ui.on_input_seek({
        let input_player = Arc::clone(&input_player);
        move |new_pos: f32| {
            if let Ok(mut input_player) = input_player.lock() {
                input_player.seek(new_pos);
            }
        }
    });
    ui.on_output_seek({
        let output_player = Arc::clone(&output_player);
        move |new_pos: f32| {
            if let Ok(mut output_player) = output_player.lock() {
                output_player.seek(new_pos);
            }
        }
    });

    ui.on_decode({
        let input_player = Arc::clone(&input_player);
        let weak_ui = ui.as_weak();
        move |repeating: bool, bits: i32| {
            thread::spawn({
                let input_player = Arc::clone(&input_player);
                let weak_ui = weak_ui.clone();
                move || {
                    let _ = weak_ui.upgrade_in_event_loop(move |ui| {
                        ui.set_message_text("< Дешифровка в процессе >".into());
                    });

                    let mut data: Vec<i16> = Vec::new();

                    if let Ok(input_player) = input_player.lock() {
                        data = input_player.data.clone();
                    };

                    let bits = bits as u8;

                    let mut message_bytes = Vec::<u8>::new();

                    let mut bit_iterator = bit_iterator::BitIterator {
                        bits,
                        iter: data.iter().map(|i| *i as u8),
                        curr_bit: bits,
                        curr_item: 0,
                    };

                    'outer: loop {
                        let mut new_byte = 0;
                        for i in 0..8 {
                            match bit_iterator.next() {
                                None => break 'outer,
                                Some(true) => new_byte |= 1 << i,
                                Some(false) => (),
                            }
                        }
                        message_bytes.push(new_byte);
                        if message_bytes.len() >= 10000 {
                            break;
                        }
                    }

                    let mut s: String = String::from_utf8_lossy(&message_bytes).to_owned().into();

                    // tuncate on invalid character
                    for (i, ch) in s.chars().enumerate() {
                        if ch == std::char::REPLACEMENT_CHARACTER
                        // || (ch as u32) < 32
                        {
                            s = s.chars().take(i).collect();
                            break;
                        }
                    }

                    if repeating {
                        let period = prefix_function::period(&s);
                        s = s.chars().take(period).collect();
                    }

                    if s.is_empty() {
                        let _ = weak_ui.upgrade_in_event_loop(move |ui| {
                            ui.set_message_text("< Пусто >".into());
                        });
                    } else {
                        let _ = weak_ui.upgrade_in_event_loop(move |ui| {
                            ui.set_message_text(s.into());
                        });
                    }
                }
            });
        }
    });

    ui.on_encode({
        let weak_ui = ui.as_weak();
        let input_player = Arc::clone(&input_player);
        let output_player = Arc::clone(&output_player);
        move |repeating: bool, bits: i32, message: slint::SharedString| {
            thread::spawn({
                let input_player = Arc::clone(&input_player);
                let output_player = Arc::clone(&output_player);
                let weak_ui = weak_ui.clone();
                move || {
                    let _ = weak_ui.upgrade_in_event_loop(move |ui| {
                        ui.set_output_filename("< Шифровка в процессе >".into());
                    });

                    let mut data: Vec<i16> = Vec::new();
                    if let Ok(input_player) = input_player.lock() {
                        data = input_player.data.clone(); // actually we want to modify output
                    };

                    let bit_iterator = bit_iterator::BitIterator {
                        bits: 8,
                        iter: message.as_bytes().iter().copied(),
                        curr_bit: 8,
                        curr_item: 0,
                    };

                    let mut bit_iterator: Box<dyn Iterator<Item = bool>> = match repeating {
                        true => Box::new(bit_iterator.cycle()),
                        false => Box::new(bit_iterator),
                    };

                    'outer: for sample in data.iter_mut() {
                        for i in 0..bits {
                            let Some(bit) = bit_iterator.next() else {
                                break 'outer;
                            };
                            if bit {
                                *sample |= 1 << i;
                            } else {
                                *sample &= !(1 << i);
                            }
                        }
                    }

                    // let file = tempfile::NamedTempFile::new().expect("Failed to create a temporary file");

                    let mut wav_writer = hound::WavWriter::create(
                        &*tmp_file,
                        hound::WavSpec {
                            channels: 2,
                            sample_rate: 48000,
                            bits_per_sample: 16,
                            sample_format: hound::SampleFormat::Int,
                        },
                    )
                    .expect("Failed to open temporary file for data writing.");

                    for sample in data.iter() {
                        if wav_writer.write_sample(*sample).is_err() {break};
                    }
                    if wav_writer.finalize().is_ok() {
                        let _ = weak_ui.upgrade_in_event_loop(move |ui| {
                            if let Ok(mut output_player) = output_player.lock() {
                                let (duration, _, image) = output_player.load(tmp_file.path().into());
                                ui.set_output_waveform(image);
                                ui.set_output_duration(duration);
                                ui.set_output_filename("< Несохранённое аудио >".into());
                            };
                        });
                    } 
                }
            });
        }
    });

    thread::spawn({
        let weak_ui = ui.as_weak();
        let input_player = Arc::clone(&input_player);
        let output_player = Arc::clone(&output_player);
        move || {
            loop {
                thread::sleep(Duration::from_millis(30));
                let input_player = Arc::clone(&input_player);
                let output_player = Arc::clone(&output_player);
                let _ = weak_ui.upgrade_in_event_loop(move |ui| {
                    if !ui.get_input_dragged()
                        && let Ok(input_player) = input_player.lock()
                    {
                        ui.set_input_playback_position(input_player.sink.get_pos().as_secs_f32());
                        ui.set_input_is_playing(
                            !input_player.sink.empty() && !input_player.sink.is_paused(),
                        );
                    }
                    if !ui.get_output_dragged()
                        && let Ok(output_player) = output_player.lock()
                    {
                        ui.set_output_playback_position(output_player.sink.get_pos().as_secs_f32());
                        ui.set_output_is_playing(
                            !output_player.sink.empty() && !output_player.sink.is_paused(),
                        );
                    }
                });
            }
        }
    });

    ui.run()?;

    Ok(())
}
