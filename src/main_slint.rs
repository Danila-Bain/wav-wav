use rfd::FileDialog;
use rodio::{Decoder, decoder::LoopedDecoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::error::Error;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io::BufReader, path::PathBuf};
use std::thread;

slint::include_modules!();


pub fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let logical_size = slint::LogicalSize::new(800.0, 800.0);
    let physical_size = logical_size.to_physical(ui.window().scale_factor());
    ui.window().set_size(physical_size); // don't wait for "Set Size" to be clicked; set the size now!

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).expect("Failed to create sink");
    let sink = Arc::new(sink);

    // let source : Option<Deconder> = None;
        // Decoder::new(BufReader::new(file)).expect("Failed to decode file");

    ui.on_choose_audio_file({
        let sink = Arc::clone(&sink);
        move || {
            let Some(path) = FileDialog::new().pick_file() else {return};
            let Ok(file) = File::open(path) else {return};
            let Ok(source) = Decoder::new_looped(BufReader::new(file)) else {return};
            sink.append(source);
        }
    });

    ui.on_input_play_toggle({ 
        let sink = Arc::clone(&sink); 
        move || -> bool {
            let is_paused = sink.is_paused();
            match is_paused {
                true => sink.play(),
                false => sink.pause(),
            }
            return !is_paused; 
        }
    });

    ui.on_input_seek({
        let sink = Arc::clone(&sink);
        move |new_pos: f32| {
            sink.try_seek(Duration::from_secs_f32(8. * new_pos))
                .expect("Seek failed");
        }
    });



    thread::spawn({
        let weak_ui = ui.as_weak();
        move || {
            let mut pos = 0.0;
            loop {
                // thread::sleep(Duration::from_millis(100));

                pos += 0.5;
                if pos > 100.0 {
                    pos = 0.0;
                }

                if let Some(strong_ui) = weak_ui.upgrade() {
                    strong_ui.set_input_playback_position(pos);
                } else {
                    break;
                }

            }
        }
    });


    ui.run()?;

    Ok(())
}
