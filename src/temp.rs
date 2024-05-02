// use std::fs::File;
// use std::io::BufReader;
// use rodio::{Decoder, OutputStream, Source};
// use nannou::prelude::*;

// fn main() {
//     nannou::app(model).run();
// }

// struct Model {
//     stream_handle: rodio::OutputStreamHandle,
// }

// fn model(app: &App) -> Model {
//     app.new_window().key_pressed(key_pressed).view(view).build().unwrap();
//     let (_stream, stream_handle) = OutputStream::try_default().expect("failed to create output stream");

//     Model { stream_handle }
// }

// fn key_pressed(_app: &App, model: &mut Model, key: Key) {
//     if key == Key::Space {
//         println!("Spacebar pressed!");
//         // Load a sound from a file, using a path relative to Cargo.toml
//         let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//         let file = File::open("src/test.mp3").expect("failed to open audio file");
//         let file = BufReader::new(file);

//         // Decode that sound file into a source
//         let decoder = Decoder::new(file).expect("failed to decode audio file");

//         // Convert the sample format to f32 and set the volume
//         let source = decoder.convert_samples().amplify(0.25);

//         // Play the sound directly on the device
//         let _ = stream_handle.play_raw(source);
//         std::thread::sleep(std::time::Duration::from_secs(5));
//     }
// }

// fn view(_app: &App, _model: &Model, frame: Frame) {
//     frame.clear(DIMGRAY);
// }
