use rfd::FileDialog;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source, decoder::LoopedDecoder};
use std::error::Error;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io::BufReader, path::PathBuf};
use std::{process, thread};

slint::include_modules!();

mod bit_iterator;
use bit_iterator::*;

pub fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let logical_size = slint::LogicalSize::new(720.0, 720.0);
    let physical_size = logical_size.to_physical(ui.window().scale_factor());
    ui.window().set_size(physical_size); // don't wait for "Set Size" to be clicked; set the size now!

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).expect("Failed to create sink");
    let sink = Arc::new(Mutex::new(sink));

    let input_audio = Arc::new(Mutex::new(Vec::<i16>::new()));

    // let source : Option<Deconder> = None;
    // Decoder::new(BufReader::new(file)).expect("Failed to decode file");

    ui.on_choose_audio_file({
        let sink = Arc::clone(&sink);
        let input_audio = Arc::clone(&input_audio);
        move || -> (f32, slint::SharedString) {
            if let Some(path) = FileDialog::new().pick_file()
                && let Ok(file) = File::open(&path)
                && let Ok(mut wav_reader) = hound::WavReader::open(&path)
                && let Ok(source) = Decoder::new(BufReader::new(file))
                && let Ok(sink) = sink.lock()
                && let Ok(mut input_audio) = input_audio.lock()
                && let Some(filename) = path.file_name()
            {
                let duration = 0.5 * source.size_hint().0 as f32 / source.sample_rate() as f32;
                sink.append(source);
                *input_audio = wav_reader.samples::<i16>().filter_map(Result::ok).collect();
                return (
                    duration,
                    filename.to_str().unwrap_or("<Filename Error>").into(),
                );
            } else {
                return (0., "".into());
            };
        }
    });

    ui.on_input_play_toggle({
        let sink = Arc::clone(&sink);
        move || -> bool {
            if let Ok(sink) = sink.lock() {
                let is_paused = sink.is_paused();
                match is_paused {
                    true => {
                        sink.play();
                        return true;
                    }
                    false => {
                        sink.pause();
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
    });

    ui.on_input_seek({
        let sink = Arc::clone(&sink);
        move |new_pos: f32| {
            if let Ok(sink) = sink.lock() {
                sink.try_seek(Duration::from_secs_f32(new_pos))
                    .expect("Seek failed");
            }
        }
    });

    ui.on_decode({
        let input_audio = Arc::clone(&input_audio);
        move |bits: i32| -> slint::SharedString {
            let Ok(input_audio) = input_audio.lock() else {
                return String::default().into();
            };

            let bits = bits as u8;

            let mut message_bytes = Vec::<u8>::new();

            let mut bit_iterator = BitIterator {
                bits,
                iter: input_audio.iter().map(|i| *i as u8),
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

            println!("{}", message_bytes.len());

            message_bytes.truncate(100);

            String::from_utf8_lossy(&message_bytes).into_owned().into()
        }
    });

    ui.on_close(move || {
        process::exit(0);
    });

    ui.on_window({
        // doesn't work in hyprland
        let weak_ui = ui.as_weak();
        move || {
            let _ = weak_ui.upgrade_in_event_loop(move |ui| {
                let is_maximized = ui.window().is_maximized();
                ui.window().set_maximized(!is_maximized);
            });
        }
    });

    ui.on_minimize({
        // doesn't work in hyprland
        let weak_ui = ui.as_weak();
        move || {
            let _ = weak_ui.upgrade_in_event_loop(move |ui| ui.window().set_minimized(true));
        }
    });

    thread::spawn({
        let weak_ui = ui.as_weak();
        let sink = Arc::clone(&sink);
        move || {
            loop {
                thread::sleep(Duration::from_millis(100));
                // println!("pos : {pos}, sink pos: {:?}", sink.get_pos());
                let sink = sink.clone();
                let _ = weak_ui.upgrade_in_event_loop(move |ui| {
                    if let Ok(sinked) = sink.lock() {
                        ui.set_input_playback_position(sinked.get_pos().as_secs_f32())
                    }
                });
            }
        }
    });

    ui.run()?;

    Ok(())
}
