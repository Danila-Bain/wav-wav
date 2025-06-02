use prefix_function::prefix_function;
use rfd::FileDialog;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use slint::ComponentHandle;
use tempfile::NamedTempFile;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

slint::include_modules!();

mod bit_iterator;
mod prefix_function;

struct Player {
    sink: Sink,
    data: Vec<i16>,
    path: Option<PathBuf>,
}

lazy_static::lazy_static!(
    static ref tmp_file: NamedTempFile = NamedTempFile::new().unwrap();
);

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

    fn load(&mut self, path: PathBuf) -> (f32, slint::SharedString) {
        let file = File::open(&path).expect("Failed to open the file for playback");
        let mut wav_reader =
            hound::WavReader::open(&path).expect("Failed to open the file for data");
        let source =
            Decoder::new(BufReader::new(file)).expect("Failed to decode the the file into wav");
        let filename = path
            .file_name()
            .expect("Failed to convert the path to filename to display");

        let duration = 0.5 * source.size_hint().0 as f32 / source.sample_rate() as f32;
        self.sink.clear();
        self.sink.append(source);
        self.sink.pause();
        let _ = self.sink.try_seek(Duration::from_secs_f32(0.));
        self.data = wav_reader.samples::<i16>().filter_map(Result::ok).collect();
        self.path = Some(path.clone());
        return (
            duration,
            filename
                .to_str()
                .unwrap_or("< Filename Display Error >")
                .into(),
        );
    }

    fn seek(&mut self, new_pos: f32) {
        if self.sink.empty()
            && let Some(path) = self.path.clone()
        {
            self.load(path);
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

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let input_player = Arc::new(Mutex::new(Player::new(&stream_handle)));
    let output_player = Arc::new(Mutex::new(Player::new(&stream_handle)));

    ui.on_choose_audio_file({
        let input_player = Arc::clone(&input_player);
        move || -> (f32, slint::SharedString) {
            if let Some(path) = FileDialog::new()
                .add_filter("wav files", &["wav"])
                .pick_file()
                // && let Ok(file) = File::open(&path)
                // && let Ok(mut wav_reader) = hound::WavReader::open(&path)
                // && let Ok(source) = Decoder::new(BufReader::new(file))
                && let Ok(mut input_player) = input_player.lock()
            // && let Some(filename) = path.file_name()
            {
                return input_player.load(path.into());
            } else {
                return (0., "".into());
            };
        }
    });

    ui.on_save_audio_file({
        let output_player = Arc::clone(&output_player);
        move || -> slint::SharedString {
            if let Ok(output_player) = output_player.lock()
                && !output_player.data.is_empty()
                && let Some(path) = FileDialog::new().save_file()
                && let Ok(file) = File::create(&path)
                && let Ok(mut wav_writer) = hound::WavWriter::new(
                    file,
                    hound::WavSpec {
                        channels: 2,
                        sample_rate: 48000,
                        bits_per_sample: 16,
                        sample_format: hound::SampleFormat::Int,
                    },
                )
                && let Some(filename) = path.file_name()
            {
                for sample in output_player.data.iter() {
                    wav_writer.write_sample(*sample).unwrap();
                }
                wav_writer.finalize().unwrap();

                return filename.to_str().unwrap_or("< Filename Error >").into();
            } else {
                return "< Error saving file >".into();
            };
        }
    });

    ui.on_input_play_toggle({
        let input_player = Arc::clone(&input_player);
        move || -> bool {
            if let Ok(mut input_player) = input_player.lock() {
                if input_player.sink.empty() {
                    if let Some(path) = input_player.path.clone() {
                        input_player.load(path);
                        input_player.sink.play();
                        return true;
                    } else {
                        return false;
                    }
                } else {
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
                    if let Some(path) = output_player.path.clone() {
                        output_player.load(path);
                        output_player.sink.play();
                        return true;
                    } else {
                        return false;
                    }
                } else {
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
        move |repeating: bool, bits: i32| -> slint::SharedString {
            let Ok(input_player) = input_player.lock() else {
                return String::default().into();
            };

            let bits = bits as u8;

            let mut message_bytes = Vec::<u8>::new();

            let mut bit_iterator = bit_iterator::BitIterator {
                bits,
                iter: input_player.data.iter().map(|i| *i as u8),
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

            s.into()
        }
    });

    ui.on_encode({
        let input_player = Arc::clone(&input_player);
        let output_player = Arc::clone(&output_player);

        move |repeating: bool, bits: i32, message: slint::SharedString| {
            let Ok(input_player) = input_player.lock() else {
                return;
            };
            let Ok(mut output_player) = output_player.lock() else {
                return;
            };

            output_player.data = input_player.data.clone(); // actually we want to modify output
            //
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

            'outer: for sample in output_player.data.iter_mut() {
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

            let mut wav_writer = hound::WavWriter::new(
                tmp_file.as_file(),
                hound::WavSpec {
                    channels: 2,
                    sample_rate: 48000,
                    bits_per_sample: 16,
                    sample_format: hound::SampleFormat::Int,
                },
            ).expect("Failed to open temporary file for data writing.");

            for sample in output_player.data.iter() {
                wav_writer.write_sample(*sample).unwrap();
            }
            wav_writer.finalize().unwrap();

            // output_player.path = Some(file.path().into());
            //

            let path = tmp_file.path().into();
            // println!("{path:?}");
            // let _ = file.close();
            output_player.load(path);

            // if let Ok(file) = file.reopen()
            //     && let Ok(source) = Decoder::new_wav(BufReader::new(file))
            // {
            //     output_player.sink.append(source);
            // }
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
                    if let Ok(input_player) = input_player.lock() {
                        ui.set_input_playback_position(input_player.sink.get_pos().as_secs_f32());
                        ui.set_input_is_playing(
                            !input_player.sink.empty() && !input_player.sink.is_paused(),
                        );
                    }
                    if let Ok(output_player) = output_player.lock() {
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
