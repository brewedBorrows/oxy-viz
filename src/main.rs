
use nannou::prelude::*;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{self, Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use minimp3::{Decoder as MiniDecoder, Frame as miniFrame};
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;

mod calculation;
mod render_drawing;


enum Command {
    Play,
    Pause,
    Seek(Duration),
    CalculateFFT,
}

struct Playback {
    is_playing: bool,
    curr_pos: Arc<Mutex<Duration>>
}
struct Model {
    sender: Sender<Command>,
    playback: Playback,
}
pub struct Audio {
    pub sounds: Vec<BufReader<File>>,
    pub volume_sender: std::sync::mpsc::Sender<f32>,
    pub volume: Arc<Mutex<f32>>,
    pub fft_output: Arc<Mutex<Vec<Complex<f32>>>>,
}



fn main() {
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    println!("i am in model");
    app.new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    let fft_output: Arc<Mutex<Vec<Complex<f32>>>> = Arc::new(Mutex::new(vec![]));
    let (volume_sender, _volume_receiver) = channel();
    let volume: Arc<Mutex<f32>> = Arc::new(Mutex::new(0.0));

    let (sender, receiver) = mpsc::channel::<Command>();
    let playback_position = Arc::new(Mutex::new(Duration::from_secs(0)));
    
    let playback_position_clone = Arc::clone(&playback_position);
    let audio_model = Audio  {
        sounds: vec![],
        volume_sender,
        fft_output: Arc::clone(&fft_output),
        volume: Arc::clone(&volume),
    };
   
    println!("this is just before stream play");
    thread::spawn(move || audio_control_thread(receiver, playback_position_clone));
    println!("Audio thread spawned");
    
    Model {
        sender, 
        playback: Playback {
            is_playing: false,
            curr_pos: playback_position,
        },

    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            model.playback.is_playing = !model.playback.is_playing;
            let cmd = if model.playback.is_playing == false {
                Command::Play
            } else {
                Command::Pause
            };
            model.sender.send(cmd).unwrap();
        }
        Key::Q => {
            model.sender.send(Command::CalculateFFT).unwrap();
        }
        Key::Left => {
            println!("Left arrow key pressed");
            let lock = model.playback.curr_pos.lock().unwrap(); //locked the variable here for mutex,
            let new_position = if *lock > Duration::from_secs(5) {
                *lock - Duration::from_secs(5)
            } else {
                Duration::from_secs(0)
            };
            model.sender.send(Command::Seek(new_position)).unwrap(); // passing the timestamp to audio thread
        }
        Key::Right => {
            println!("Right arrow key pressed");
            let lock = model.playback.curr_pos.lock().unwrap(); //locked the variable here for mutex,  and blah blah blah
            let new_position = *lock + Duration::from_secs(5);
            model.sender.send(Command::Seek(new_position)).unwrap(); // passing the timestamp to audio thread
        }
        _ => {}
    }
}
fn display_frequencies(buffer: &[Complex<f32>], sample_rate: usize, fft_size: usize) {
    let target_notes = generate_note_frequencies(4); // Generate frequencies for 4 octaves

    for (i, complex) in buffer.iter().enumerate() {
        let frequency = (i as f32 * sample_rate as f32) / fft_size as f32;
        let magnitude = complex.norm();
        // Check if the frequency is close to any target note frequencies
        if target_notes.iter().any(|&n| (frequency - n).abs() < 1.0) && magnitude > 1.0 {
            println!("Frequency: {:.2} Hz, Magnitude: {:.4}", frequency, magnitude);
        }
    }
}

fn generate_note_frequencies(octaves: usize) -> Vec<f32> {
    let base_notes = [261.63, 277.18, 293.66, 311.13, 329.63, 349.23, 369.99, 392.00, 415.30, 440.00, 466.16, 493.88];
    let mut frequencies = Vec::new();
    for octave in 0..octaves {
        let multiplier = 2.0f32.powi(octave as i32);
        frequencies.extend(base_notes.iter().map(|&f| f * multiplier));
    }
    frequencies
}




fn audio_control_thread(receiver: Receiver<Command>, playback_position: Arc<Mutex<Duration>>) {
    println!("i am in audio_control_thread");

    let file_path = "src/test.mp3";
    let file = File::open(file_path).expect("Failed to open audio file");
    let mut decoder = MiniDecoder::new(BufReader::new(file));

    // Decode entire file into memory
    let mut all_samples = Vec::new();
    while let Ok(miniFrame { data, .. }) = decoder.next_frame() {
        all_samples.extend(data);
    }
    let file = File::open("src/test.mp3").expect("Failed to open audio file");
    let file = BufReader::new(file);
    let decoder = Decoder::new(file).expect("Failed to decode audio file");
    let source: rodio::source::Amplify<rodio::source::SamplesConverter<Decoder<BufReader<File>>, i16>> = decoder.convert_samples::<i16>().amplify(0.25);
    let sample_rate = 44100;
    let channels = 2;
    let window_size = (0.5 * sample_rate as f32) as usize * channels;

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size );
    
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source);

    let last_play_time = Instant::now();

    for command in receiver {
        match command {
            
            Command::CalculateFFT => {
                println!("Calculating FFT");
                let elapsed = last_play_time.elapsed();
                let start_pos =elapsed;
                println!("will this be {:?}",start_pos);
                let samples_offset = (start_pos.as_secs_f32() * sample_rate as f32) as usize;
                println!("{:?}",start_pos);
                if samples_offset + window_size <= all_samples.len() {
                    let mut buffer: Vec<Complex<f32>> = all_samples[samples_offset..samples_offset + window_size]
                        .iter()
                        .map(|&x| Complex::new(x as f32, 0.0))
                        .collect();

                    fft.process(&mut buffer);  // Perform FFT in-place

                    // Display the frequency and magnitude informationqqqqqqq
                    display_frequencies(&buffer, sample_rate, window_size);
                } else {
                    println!("Not enough data available for FFT calculation at the current position.");
                }
            },
            Command::Play => {
                println!("Playing audio");
                sink.play();
                // last_play_time = Instant::now();
            }
            Command::Pause => {
                println!("Pausing audio");
                let elapsed = last_play_time.elapsed();
                *playback_position.lock().unwrap() += elapsed;
                sink.pause();
            }
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

fn view(app: &App, _model: &Model, frame: Frame) {
   

    render_drawing::draw(app, frame, &render_drawing::Data { amp: 34.333 });
}






