use rfd::FileDialog;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::error::Error;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io::BufReader, path::PathBuf};

slint::include_modules!();

struct Audio {
    path: PathBuf,
    pos: f32,
    duration: f32,
    is_paused: bool,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let logical_size = slint::LogicalSize::new(800.0, 800.0);
    let physical_size = logical_size.to_physical(ui.window().scale_factor());
    ui.window().set_size(physical_size); // don't wait for "Set Size" to be clicked; set the size now!


    // sink initialization
    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    std::mem::forget(stream); // Keep stream alive
    let sink = Sink::try_new(&stream_handle).expect("Failed to create sink");
    sink.pause();
    let sink = Arc::new(Mutex::new(sink));

    // let mut input_audio: Option<Audio> = None;
    // let mut output_audio : Option<Audio> = None;

    let sink_ = sink.clone();
    ui.on_choose_audio_file(move || {
        if let Some(path) = FileDialog::new().pick_file() {
            let file = File::open(&path).expect("Failed to open file");
            let source = Decoder::new(BufReader::new(file)).expect("Failed to decode file");
            let duration = source.total_duration().expect("No duration").as_secs_f32();
            // input_audio = Some(Audio {
            //     path,
            //     pos: 0.,
            //     duration,
            //     is_paused: true,
            // });
            let sink = sink_.lock().unwrap();
            sink.stop();
            sink.pause();
            sink.append(source);
        }
    });

    let sink_ = sink.clone();
    ui.on_play_toggle(move || {
        let sink = sink_.lock().unwrap();
        match sink.is_paused() {
            true => sink.play(),
            false => sink.pause(),
        };
    });

    let sink_ = sink.clone();
    ui.on_seek(move |new_pos: f32| {
        let sink = sink_.lock().unwrap();
        sink.try_seek(Duration::from_secs_f32(8. * new_pos))
            .expect("Seek failed");
    });

    ui.run()?;

    Ok(())
}
