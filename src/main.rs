use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use rodio::{Decoder, OutputStream, Sink, Source};
use nannou::prelude::*;

enum Command {
    Play,
    Pause,
    Stop,
}

struct Model {
    sender: Sender<Command>,
    playing: bool,
}

fn main() {
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    app.new_window().key_pressed(key_pressed).view(view).build().unwrap();

    let (sender, receiver) = mpsc::channel::<Command>();
    thread::spawn(|| audio_control_thread(receiver));
    println!("Audio thread spawned");

    Model { sender, playing: false }
}

#[allow(unused_variables)]
fn key_pressed(app: &App, model: &mut Model, key: Key) {
    if key == Key::Space {
        model.playing = !model.playing;
        if model.playing == false {
            println!("Sending Play command");
            model.sender.send(Command::Play).unwrap();
        } else {
            println!("Sending Pause command");
            model.sender.send(Command::Pause).unwrap();
        }
    }
}

fn audio_control_thread(receiver: Receiver<Command>) {
    let file = File::open("src/test.mp3").expect("failed to open audio file");
    let file = BufReader::new(file);
    let decoder = Decoder::new(file).expect("failed to decode audio file");
    let source = decoder.convert_samples::<i16>().amplify(0.25); 
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source);
    for command in receiver {
        match command {
            Command::Play => {
                println!("Playing audio");
                sink.play();
            },
            Command::Pause => {
                println!("Pausing audio");
                sink.pause();
            },
            Command::Stop => {
                println!("Stopping audio");
                sink.stop();
                break;
            },
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(DIMGRAY);
}
