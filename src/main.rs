use crate::ui::Button;
use hound::{WavReader, WavSpec};
use minimp3::{Decoder as MiniDecoder, Frame as miniFrame};
use nannou::prelude::*;
use nannou::state::mouse;
use nannou::text::pt_to_scale;
use rodio::{Decoder, OutputStream, Sink, Source, buffer::SamplesBuffer};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{self, channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};


mod calculation;
mod render_drawing;
mod ui;

enum Command {
    Play,
    Pause,
    Seek(Duration),
    CalculateFFT,
}

struct Playback {
    is_playing: bool,
    curr_pos: Arc<Mutex<Duration>>,
    fft_output: Arc<Mutex<Vec<Complex<f32>>>>,
    fav_part: (f32, f32),
}

struct Model {
    sender: Sender<Command>,
    playback: Playback,
    data: render_drawing::Data,
    temp: u128,
    buttons: Vec<ui::Button>,
}
fn main() {
    nannou::app(model).update(update).run();
}

const SRC: &str = "src/test.wav";

fn model(app: &App) -> Model {
    println!("i am in model");
    app.new_window()
        .key_pressed(key_pressed)
        .event(mouse_event)
        .view(view)
        .build()
        .unwrap();

    let fft_output: Arc<Mutex<Vec<Complex<f32>>>> = Arc::new(Mutex::new(vec![]));

    let (sender, receiver) = mpsc::channel::<Command>();
    let playback_position = Arc::new(Mutex::new(Duration::from_secs(0)));

    let playback_position_clone = Arc::clone(&playback_position);
    let fft_output_clone = Arc::clone(&fft_output);

    println!("this is just before stream play");
    thread::spawn(move || {
        audio_control_thread(receiver, playback_position_clone, fft_output_clone)
    });
    println!("Audio thread spawned");

    // gen random data for testing
    let random_data = render_drawing::Data::create_random_data();
    let temp = 0;
    // println!("--data: {:?}", random_data);

    let play_button = ui::Button::new(
        ui::ButtonType::Play,
        ui::BBox::new(0.0, 0.0, 500., 500.),
        || {
            println!("Play button clicked");
        },
    );

    let buttons = vec![play_button];

    Model {
        sender,
        playback: Playback {
            is_playing: false,
            curr_pos: playback_position,
            fft_output,
            fav_part: (0.0, 0.0),
        },
        temp,
        data: random_data,
        buttons,
    }
}

fn mouse_event(app: &App, model: &mut Model, event: WindowEvent) {
    // get mousex and mousey
    let pos = app.mouse.position();
    let (x, y) = (pos.x, pos.y);
    for button in &model.buttons {
        match button.buttonType {
            ui::ButtonType::Play => {}
        }
    }
    match event {
        MousePressed(_button) => {
            // println!("Mouse pressed at x: {}, y: {}", x, y);
            let button_clicked_type = ui::check_button_click(x, y, &model.buttons);
            if let Some(button_type) = button_clicked_type {
                match button_type {
                    ui::ButtonType::Play => {
                        println!("Play button clicked");
                        model.playback.is_playing = !model.playback.is_playing;
                        let cmd = if model.playback.is_playing == false {
                            Command::Play
                        } else {
                            Command::Pause
                        };
                        model.sender.send(cmd).unwrap();
                    }
                }
            }
        }
        _ => {}
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

fn display_frequencies(
    buffer: &[Complex<f32>],
    sample_rate: usize,
    fft_size: usize,
    fft_output: Arc<Mutex<Vec<Complex<f32>>>>,
) {
    let target_notes = generate_note_frequencies(4); // Generate frequencies for 4 octaves
    assert!(target_notes.len() == 48, "Expected 48 target notes");

    let mut output = fft_output.lock().expect("Mutex was poisoned").to_vec();

    // Clear previous results
    output.clear();

    for freq in &target_notes {
        let freq_magnitude_complex = Complex::new(*freq, 0.0);
        output.push(freq_magnitude_complex);
    }

    for (i, complex) in buffer.iter().enumerate() {
        let frequency = (i as f32 * sample_rate as f32) / fft_size as f32;
        let magnitude = complex.norm();
        for freq_magnitude in output.iter_mut() {
            if (frequency - freq_magnitude.re).abs() < 1.0 && magnitude > 1.0 {
                // Update the magnitude if the condition is met
                freq_magnitude.im = magnitude;
                break; // Stop checking once the first match is found and updated
            }
        }
    }

    // println!("--output: {:?}", output);

    assert!(
        output.len() == 48,
        "Expected 48 output values after processing"
    );
    *fft_output.lock().unwrap() = output;
}

fn generate_note_frequencies(octaves: usize) -> Vec<f32> {
    let base_notes = [
        261.63, 277.18, 293.66, 311.13, 329.63, 349.23, 369.99, 392.00, 415.30, 440.00, 466.16,
        493.88,
    ];
    let mut frequencies = Vec::new();
    for octave in 0..octaves {
        let multiplier = 2.0f32.powi(octave as i32);
        frequencies.extend(base_notes.iter().map(|&f| f * multiplier));
    }
    frequencies
}

fn audio_control_thread(
    receiver: Receiver<Command>,
    playback_position: Arc<Mutex<Duration>>,
    fft_output: Arc<Mutex<Vec<Complex<f32>>>>,
) {
    println!("i am in audio_control_thread");

    let file_path = SRC;
    let mut reader = WavReader::open(file_path).expect("Failed to open WAV file");
    let spec = reader.spec();

    // Decode entire file into memory
    let all_samples: Vec<i16> = reader.samples::<i16>().filter_map(Result::ok).collect();

    let sample_rate = spec.sample_rate as usize;
    let channels = spec.channels as usize;
    let window_size = ( sample_rate as f32) as usize; 

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    // Convert all_samples to a SamplesBuffer which acts as a Source
    let samples_buffer = SamplesBuffer::new(2, sample_rate as u32, all_samples.clone());
    sink.append(samples_buffer);

    let last_play_time = Instant::now();

    for command in receiver {
        match command {
            Command::CalculateFFT => {
                // FFT calculation logic remains the same
                let start_pos = *playback_position.lock().unwrap();
                let samples_offset = (start_pos.as_secs_f32() * sample_rate as f32) as usize;

                println!("Calculating FFT for sample_offset+window_size: {:?}", samples_offset+window_size);
                println!("Calculating FFT for all_samples: {:?}", all_samples.len());
                if samples_offset + window_size <= all_samples.len() {
                    let mut buffer: Vec<Complex<f32>> = all_samples
                        [samples_offset..samples_offset + window_size]
                        .iter()
                        .map(|&x| Complex::new(x as f32, 0.0))
                        .collect();

                    fft.process(&mut buffer); // Perform FFT in-place
                    println!("FFT calculated for start_pos: {:?}", start_pos);
                    display_frequencies(&buffer, sample_rate, window_size, fft_output.clone());
                    println!("FFT output updated for{:?}",start_pos);
                } else {
                    println!("Reached end of audio samples.");
                    sink.stop();
                    println!(
                        "Not enough data available for FFT calculation at the current position."
                    );
                }
            }
            Command::Play => {
                println!("Playing audio");
                sink.play();
            }
            Command::Pause => {
                println!("Pausing audio");
                // let elapsed = last_play_time.elapsed();
                // *playback_position.lock().unwrap() += elapsed;
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

fn update(app: &App, model: &mut Model, event: Update) {
    
    model.temp += event.since_last.as_millis();
    if model.temp > 100 {
        model.temp = 0;
        // println!("------ event called{:?}", event.since_last.as_secs_f32());
        model.sender.send(Command::CalculateFFT).unwrap();
    }
    // update curr position
    if !model.playback.is_playing {
        let elapsed = event.since_last;
        let mut lock = model.playback.curr_pos.lock().unwrap();
        *lock += elapsed;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // calulations for viz
    // let amp = calculation::calculate(&model.playback.is_playing);

    let x = model
        .playback
        .fft_output
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .to_vec();
    if x.len() < 48 {
        return;
    } else {
        let octaves_flat: Vec<f32> = model
            .playback
            .fft_output
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .to_vec()[x.len() - 48..]
            .iter()
            .map(|x| x.im)
            .collect();

        let octaves: Vec<Vec<f32>> = octaves_flat.chunks(12).map(|x| x.to_vec()).collect();
        // println!("--octaves: {:?}", octaves);
        assert!(octaves.len() == 4, "Expected 4 octaves");
        
        let data = render_drawing::Data::new(octaves);

        // println!("--fft_output: {:?}", octaves_flat);
        if frame.nth()%100 == 0{
            render_drawing::draw_on_window(app, frame, &data, &model.buttons);
        }

           
        

    }
}
