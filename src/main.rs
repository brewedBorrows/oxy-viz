use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use rodio::{Decoder, OutputStream, Sink, Source};
use nannou::prelude::*;

enum Command {
    Play,
    Pause,
    Stop, // will use later
    Seek(Duration),
}

struct Model {
    sender: Sender<Command>,
    playing: bool,
    playback_position: Arc<Mutex<Duration>>,
}

fn main() {
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    app.new_window().key_pressed(key_pressed).view(view).build().unwrap();
    let (sender, receiver) = mpsc::channel::<Command>();
    let playback_position = Arc::new(Mutex::new(Duration::from_secs(0)));
    let playback_position_clone = Arc::clone(&playback_position);
    thread::spawn(move || audio_control_thread(receiver, playback_position_clone));
    println!("Audio thread spawned");
    Model { sender, playing: false, playback_position }
}


fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            model.playing = !model.playing;
            let cmd = if model.playing == false { Command::Play } else { Command::Pause };
            model.sender.send(cmd).unwrap();
        },
        Key::Left => {
            println!("Left arrow key pressed");
            let lock = model.playback_position.lock().unwrap();//locked the variable here for mutex, 
            let new_position = if *lock > Duration::from_secs(5) {
                *lock - Duration::from_secs(5)
            } else {
                Duration::from_secs(0)
            };
            model.sender.send(Command::Seek(new_position)).unwrap(); // passing the timestamp to audio thread
        },
        Key::Right => {
            println!("Right arrow key pressed");
            let lock = model.playback_position.lock().unwrap();//locked the variable here for mutex,  and blah blah blah
            let new_position = *lock + Duration::from_secs(5);
            model.sender.send(Command::Seek(new_position)).unwrap(); // passing the timestamp to audio thread
        },
        _ => {}
    }
}


fn audio_control_thread(receiver: Receiver<Command>, playback_position: Arc<Mutex<Duration>>) {
    let file = File::open("src/test.mp3").expect("Failed to open audio file");
    let file = BufReader::new(file);
    let decoder = Decoder::new(file).expect("Failed to decode audio file");
    let source = decoder.convert_samples::<i16>().amplify(0.25);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source);

    let mut last_play_time = Instant::now();

    for command in receiver {
        match command {
            Command::Play => {
                println!("Playing audio");
                sink.play();
                last_play_time = Instant::now();
            },
            Command::Pause => {
                println!("Pausing audio");
                let elapsed = last_play_time.elapsed();
                *playback_position.lock().unwrap() += elapsed;
                sink.pause();
            },
            Command::Stop => {
                println!("Stopping audio");
                sink.stop();
                break;
            },
            Command::Seek(position) => {
                println!("Seeking audio to {:?}", position);
                if let Err(e) = sink.try_seek(position) {
                    eprintln!("Failed to seek: {}", e);
                } else {
                    *playback_position.lock().unwrap() = position;
                    sink.play();
                }
            }
        }
    }
}

fn view(_app: &App, _model: &Model, frame: Frame) {
    frame.clear(DIMGRAY);
}
