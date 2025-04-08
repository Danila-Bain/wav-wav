use rfd::FileDialog;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::error::Error;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io::BufReader, path::PathBuf};

use playback_rs::{Song, Player};

slint::include_modules!();

// struct MySource {
//     sample_rate: u32,
//     data: Vec<i16>,
//     index: u32,
// }
//
// impl Iterator for MySource {
//     type Item = u32;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.index < self.data.len() {
//             let result = self.data[self.index];
//             self.index+=1;
//             Some(result)
//         } else {
//             None
//         }
//     }
// }
//
// impl Source for MySource {
//     fn channels(&self) -> u16 {
//         return 1
//     }
//
//     fn sample_rate(&self) -> u32 {
//         return self.sample_rate;
//     }
//
//     fn current_frame_len(&self) -> Option<usize> {
//         return self.data.len() - self.index;
//     }
//
//     fn total_duration(&self) -> Option<Duration> {
//         // return self.data.len()
//         return None;
//     }
// }

pub fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let logical_size = slint::LogicalSize::new(800.0, 800.0);
    let physical_size = logical_size.to_physical(ui.window().scale_factor());
    ui.window().set_size(physical_size); // don't wait for "Set Size" to be clicked; set the size now!

    let player = Player::new(None)?; 
    let player = Arc::new(player);

    ui.on_choose_audio_file({
        let player = Arc::clone(&player);
        move || {
            let Some(path) = FileDialog::new().pick_file() else {return};
            let Ok(song) = Song::from_file(path, None) else {return};
            if let Ok(_) = player.play_song_next(&song, None) {
                player.set_playing(true);
            }
        }
    });

    ui.on_play_toggle({ 
        let player = Arc::clone(&player); 
        move || -> bool {
            let is_playing = !player.is_playing();
            player.set_playing(is_playing);
            is_playing
        }
    });

    ui.on_seek(move |new_pos: f32| {
        // let sink = sink_.lock().unwrap();
        // sink.try_seek(Duration::from_secs_f32(8. * new_pos))
        //     .expect("Seek failed");
    });

    ui.run()?;

    Ok(())
}
