use crate::ui::Button;
use core::time;
use minimp3::{Decoder as MiniDecoder, Frame as miniFrame};
use nannou::prelude::*;
use nannou::state::mouse;
use nannou::text::pt_to_scale;
use rodio::{Decoder, OutputStream, Sink, Source};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::fs::File;
use std::io::{BufReader, Stdout};
use std::path::Path;
use std::sync::mpsc::{self, channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use walkdir::WalkDir;

use tracing::{debug, info, warn, Level};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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
    fft_output: Arc<Mutex<Vec<FreqMagPair>>>,
    fav_part: Duration,
}

struct Model {
    sender: Sender<Command>,
    playback: Playback,
    data: render_drawing::Data,
    temp: u128,
    ui_elements: Vec<ui::UIElem>,
    mp3_files: Vec<std::path::PathBuf>,
    current_track_index: u32,
}

enum LogDestination {
    Stdout,
    File,
}

/// file output is not working
fn setup_logger(destination: LogDestination) {
    if let LogDestination::File = destination {
        let file_appender = rolling::never(".", "output.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        let fmt_layer = fmt::layer()
            .with_timer(fmt::time::ChronoUtc::new(String::from(
                "%Y-%m-%d %H:%M:%S%.6f",
            )))
            .with_writer(move || non_blocking.clone());
        tracing_subscriber::registry()
            .with(EnvFilter::new("audio_vis=info"))
            .with(fmt_layer)
            .init();
    } else {
        let fmt_layer =
            fmt::layer().with_timer(fmt::time::ChronoUtc::new(String::from("%H:%M:%S%.6f")));
        tracing_subscriber::registry()
            .with(EnvFilter::new("audio_vis=info"))
            .with(fmt_layer)
            .init();
    };
}

fn main() {
    setup_logger(LogDestination::Stdout);

    info!("Starting app");

    nannou::app(model).update(update).run();
}

const SRC: &str = "src/c4_maj.wav";

fn find_mp3_files(dir: &str) -> Vec<std::path::PathBuf> {
    WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file() && e.path().extension() == Some("mp3".as_ref()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// A struct to store the frequency and magnitude of a frequency bin.
/// To be used by the visualizer
#[derive(Debug, Clone, Copy)]
struct FreqMagPair {
    freq: f32,
    mag: f32,
}

struct AudioManager {
    sender_to_audio: Sender<Command>,
    playback_position: Arc<Mutex<Duration>>,
    fft_output: Arc<Mutex<Vec<FreqMagPair>>>, // will use this to store the fft output -> 48 values
    last_fft_generated_at: Arc<Mutex<Option<Instant>>>, // for debugging mostly
    mp3_files: Vec<std::path::PathBuf>,
}

fn create_audio_thread() -> AudioManager {
    let directory = "src"; // Change this to the directory you want to search
    let mp3_files = find_mp3_files(directory);

    let fft_output: Arc<Mutex<Vec<FreqMagPair>>> = Arc::new(Mutex::new(vec![]));

    let (sender, receiver) = mpsc::channel::<Command>();
    let playback_position = Arc::new(Mutex::new(Duration::from_secs(0)));

    let playback_position_clone = Arc::clone(&playback_position);
    let fft_output_clone = Arc::clone(&fft_output);

    println!("this is just before stream play");
    thread::spawn(move || {
        audio_control_thread(
            receiver,
            playback_position_clone,
            fft_output_clone,
            Arc::new(Mutex::new(None)),
        )
    });
    println!("Audio thread spawned");
    AudioManager {
        sender_to_audio: sender,
        playback_position,
        fft_output,
        mp3_files,
        last_fft_generated_at: Arc::new(Mutex::new(None)),
    }
}

fn model(app: &App) -> Model {
    println!("i am in model");
    app.new_window()
        .key_pressed(key_pressed)
        .event(mouse_event)
        .view(view)
        .build()
        .unwrap();

    // gen random data for testing
    let random_data = render_drawing::Data::create_random_data();
    let temp = 0;
    // println!("--data: {:?}", random_data);

    let ui_elements = ui::create_ui_elements(app.window_rect());

    // all the audio stuff
    let audio_manager = create_audio_thread();

    let sender = audio_manager.sender_to_audio;
    let playback_position = audio_manager.playback_position;
    let fft_output = audio_manager.fft_output;
    let mp3_files = audio_manager.mp3_files;

    Model {
        sender,
        playback: Playback {
            is_playing: false,
            curr_pos: playback_position,
            fft_output,
            fav_part: Duration::from_secs(0),
        },
        temp,
        data: random_data,
        ui_elements,
        mp3_files,
        current_track_index: 0,
    }
}

fn mouse_event(app: &App, model: &mut Model, event: WindowEvent) {
    // get mousex and mousey
    let pos = app.mouse.position();
    let (x, y) = (pos.x, pos.y);
    match event {
        MousePressed(_button) => {
            println!(" -- x: -- y:{:?} {:?}", x, y);

            for element in &model.ui_elements {
                // if element is a button
                match element {
                    ui::UIElem::Button(button) => {
                        if button.bbox.contains(x, y) {
                            println!("button clicked: {:?}", button.button_name);
                            match button.button_name {
                                ui::ButtonName::Play => {
                                    println!("Play button clicked");
                                    model.playback.is_playing = !model.playback.is_playing;
                                    let cmd = if model.playback.is_playing == false {
                                        Command::Play
                                    } else {
                                        Command::Pause
                                    };
                                    model.sender.send(cmd).unwrap();
                                }
                                ui::ButtonName::FavPlay => {
                                    println!("playing your fav part of the song");
                                    let new_position = model.playback.fav_part;
                                    model.sender.send(Command::Seek(new_position)).unwrap();
                                }
                                ui::ButtonName::FavRecord => {
                                    println!("record the fav part of the song");
                                    let lock = model.playback.curr_pos.lock().unwrap();
                                    model.playback.fav_part = *lock;
                                }
                                ui::ButtonName::Seek => {
                                    // println!("seeking the song");
                                    // Coudln't put seekbutton in the vector because it would have created a copy, or multiple ownerships
                                    // so we'll match seekbutton in ui::UIElem::SeekLine
                                }
                            }
                        }
                    }
                    ui::UIElem::SeekLine(seekline) => {
                        if seekline.bbox.contains(x, y) {
                            println!("seekline clicked");
                            let new_position = seekline.get_playback_pos(x);
                            println!("new position in percent: {:?}%", new_position * 100.);
                        }
                        if seekline.button.bbox.contains(x, y) {
                            println!("seekline button clicked");
                        }
                    }
                    _ => {}
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
        Key::N => {
            println!("looking for new N");
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

/// The buffer is the fft output now contains the frequency and magnitude information for each frequency bin.
/// buffer must of single channel (any one of the channels)
fn display_frequencies(
    buffer: &[Complex<f32>], // contains the actual fft_output
    sample_rate: usize,
    fft_size: usize,
    fft_output: Arc<Mutex<Vec<FreqMagPair>>>, // will use this to store the PROCESSED fft output -> 48 values
) {
    let THRESHOLD = 100.0;

    assert!(
        fft_output.lock().unwrap().len() == 48,
        "Expected 48 target notes"
    );

    let mut fft_output_locked = fft_output.lock().unwrap();

    for freq_magnitude in fft_output_locked.iter_mut() {
        freq_magnitude.mag = 0.0;
    }

    for (i, complex) in buffer.iter().enumerate() {
        // buffer now contains the actual fft output of the audio
        // so for each freq bin:
        let frequency = (i as f32 * sample_rate as f32) / fft_size as f32; // frequency of the fft result at this freq bin
        let magnitude = complex.norm(); // magnitude of the fft result at this freq bin
                                        // iterate over the target notes and update the magnitudes
        for FreqMagPair { freq, mag } in fft_output_locked.iter_mut() {
            // freq is the target note's frequency
            // mag is the magnitude of the target note from previous iteration
            if (frequency - *freq).abs() < 1.0 && magnitude > THRESHOLD {
                // update the target note's magnitude if
                // the frequency in buffer is close to the target note
                // and the magnitude is greater than the threshold
                // info!("FOUND CANDIDATE FREQ: {:?}", frequency);
                *mag = magnitude;
                // THE ABOVE LINE DOES"T SEEM TO BE WORKING, BECAUSE EVEN AFTER CANDIDATE FREQ IS FOUND,
                // AT THE END fft_output HAS ALL ZEROS
                break; // Stop checking once the first match is found and updated
            }
        }
    }

    // is the fft_output stll containing all 0.0 mags? if yes then prolly there's some problem
    let all_zero = fft_output_locked.iter().all(|x| x.mag == 0.0); // RETURNS TRUE
    if all_zero {
        warn!("-- fft output (after) has all zeros");
    }

    // for (i, complex) in buffer.iter().enumerate() {
    //     let frequency = (i as f32 * sample_rate as f32) / fft_size as f32;
    //     let magnitude = complex.norm();
    //     for freq_magnitude in output.iter_mut() {
    //         if (frequency - freq_magnitude.re).abs() < 1.0 && magnitude > 1.0 {
    //             // Update the magnitude if the condition is met
    //             freq_magnitude.im = magnitude;
    //             break; // Stop checking once the first match is found and updated
    //         }
    //     }
    // }

    // println!("--output: {:?}", output);
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

type RodioSource =
    rodio::source::Amplify<rodio::source::SamplesConverter<Decoder<BufReader<File>>, i16>>;
fn load_audio(file_path: &str) -> Result<RodioSource, &str> {
    // if mp3:
    info!("Loading audio file");
    let t = Instant::now();
    let file = File::open(file_path).map_err(|_| "Failed to open audio file")?;
    let file = BufReader::new(file);

    let decoder = match file_path {
        _ if file_path.ends_with(".mp3") => Decoder::new(file),
        _ if file_path.ends_with(".wav") => Decoder::new_wav(file),
        _ => return Err("Unsupported file format"),
    }
    .map_err(|_| "Failed to decode audio file")?;

    let source: rodio::source::Amplify<
        rodio::source::SamplesConverter<Decoder<BufReader<File>>, i16>,
    > = decoder.convert_samples::<i16>().amplify(0.25);
    info!("Time to load audio: {:?}", t.elapsed());
    return Ok(source);
}

fn audio_control_thread(
    receiver: Receiver<Command>,
    playback_position: Arc<Mutex<Duration>>,
    fft_output: Arc<Mutex<Vec<FreqMagPair>>>,
    last_fft_generated_at: Arc<Mutex<Option<Instant>>>,
) {
    println!("i am in audio_control_thread");

    let file_path = SRC;
    let source = load_audio(file_path).unwrap();

    // !!!!! SO USING THE SAME SOURCE SOUNDS GOOD
    // BUT THERE'S A LONG WAIT AT START: UNCOMMENT THIS MAYBE:

    // let file = File::open(file_path).expect("Failed to open audio file");
    // let mut decoder: MiniDecoder<BufReader<File>> = MiniDecoder::new(BufReader::new(file));

    // // Decode entire file into memory
    // let mut all_samples: Vec<i16> = Vec::new();
    // while let Ok(miniFrame { data, .. }) = decoder.next_frame() {
    //     all_samples.extend(data);
    // }

    // we need the same decoded Vec<i16> for fft and playback (OR DO WE?)
    // source.collect() will give that vector, but it will consume source, so we'll recreate it

    let sample_rate = source.sample_rate();
    let channels = source.channels();

    let source_copy = load_audio(file_path).unwrap();
    let t = Instant::now();
    let all_samples: Vec<i16> = source_copy.collect();
    // all_samples.len() must be divisible by channels

    // create list of all_samples_channel_0, all_samples_channel_1, etc
    // all samples is interleaved : [ch0 ch1 ch3 ch0 ch1 ch3 ...]
    let mut all_samples_channels: Vec<Vec<i16>> = vec![Vec::new(); channels as usize];
    info!(
        "all_samples.len() {:?} should be divisibel by : {:?}",
        all_samples.len(),
        channels
    );
    for (i, e) in all_samples.into_iter().enumerate() {
        let channel_index = i % channels as usize;
        all_samples_channels[channel_index].push(e);
    }
    let all_samples_channels = all_samples_channels;

    info!(
        "Audio file loaded into memory for fft, time taken: {:?}",
        t.elapsed()
    );

    // Generate frequencies for 4 octaves
    // this is out C4 C#4 ... etc data.
    // TODO: Actually we need to run this only one time in the whole program,
    // so we can move it to main or during audio setup
    let output: Vec<FreqMagPair> = generate_note_frequencies(4)
        .into_iter()
        .map(|x| FreqMagPair {
            freq: x,
            mag: 0.0,
        })
        .collect();
    
    *fft_output.lock().unwrap() = output;

    let window_size = (0.5 * sample_rate as f32) as usize * channels as usize;
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source);

    *playback_position.lock().unwrap() = Duration::from_secs(0); // important cuz all that writing all_samples takes time
    info!("Audio loaded and ready to play");

    for command in receiver {
        match command {
            Command::CalculateFFT => {
                // println!("Calculating FFT");
                // let elapsed = last_play_time.elapsed();
                let t1 = Instant::now();
                let start_pos = *playback_position.lock().unwrap();
                // println!("will this be {:?}", start_pos);
                let samples_offset = (start_pos.as_secs_f32() * sample_rate as f32) as usize;
                println!(
                    "playback at: (based on duration since start) {:?}",
                    start_pos
                );
                // println!("and time from start: {:?}", Instant::now()-*time_at_start);
                // println!("TIME DIFF: {:?}", Instant::now()-*time_at_start - start_pos);

                // calculate fft for a given channel
                let process_fft_for_channel = |channel_num: usize| {
                    info!("Calculating FFT for channel: {:?}", channel_num);
                    let all_samples_for_ch = &all_samples_channels[channel_num];
                    if samples_offset + window_size <= all_samples_for_ch.len() {
                        let mut buffer: Vec<Complex<f32>> = all_samples_for_ch
                            [samples_offset..samples_offset + window_size]
                            .iter()
                            .map(|&x| Complex::new(x as f32, 0.0))
                            .collect();
                        info!(
                            "-- sample offset {:?} window size {:?}",
                            samples_offset, window_size
                        );
                        // info!("-- buffer len:{:?} {:?}",buffer.len(), buffer);
                        fft.process(&mut buffer); // Perform FFT in-place

                        // Display the frequency and magnitude information
                        display_frequencies(
                            &buffer,
                            sample_rate as usize,
                            window_size,
                            Arc::clone(&fft_output),
                        );
                        let t2 = t1.elapsed() + start_pos;
                        println!(
                            "sampled time: (based on pos in fft buffer): {:?}",
                            t2.as_secs_f32()
                        );
                    } else {
                        println!(
                        "Not enough data available for FFT calculation at the current position."
                    );
                    }
                };

                let want_to_add = false; // DON"T SET TO TRUE!! DOESN"T WORKKKKKK
                let mut res_buffer: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); window_size];

                if !want_to_add {
                    // calculate for only channel 0
                    process_fft_for_channel(0);
                } else {
                    // TODO: use the process_fft_for_channel closure here also (slight modfn needed)
                    // add all fft result into the res_buffer

                    // calc fft for each buffer
                    for i in 0..channels as usize {
                        let all_samples_for_ch = &all_samples_channels[i];
                        info!("Calculating FFT for channel: {:?}", i);
                        if samples_offset + window_size <= all_samples_for_ch.len() {
                            let mut buffer: Vec<Complex<f32>> = all_samples_for_ch
                                [samples_offset..samples_offset + window_size]
                                .iter()
                                .map(|&x| Complex::new(x as f32, 0.0))
                                .collect();
                            fft.process(&mut buffer); // Perform FFT in-place
                            for (j, val) in buffer.iter().enumerate() {
                                res_buffer[j] = res_buffer[j] + val;
                            }
                        } else {
                            println!(
                                "Not enough data available for FFT calculation at the current position."
                            );
                        }
                    }
                    display_frequencies(
                        &res_buffer,
                        sample_rate as usize,
                        window_size,
                        Arc::clone(&fft_output),
                    );
                }

                //     // The data is like [ch0, ch1, ch0, ch1, ch0, ch1, ...]
                //     // So we may need to skip some samples to get the correct channel
                //     let samples_offset =
                //         (start_pos.as_secs_f32() * sample_rate as f32) as usize * channels as usize;
                //     println!("{:?}", start_pos);
                //     if samples_offset + window_size * channels as usize <= all_samples.len() {
                //         // De-interleave the data: collect samples for the first channel (e.g., left channel in a stereo file)
                //         let mut buffer: Vec<Complex<f32>> = (0..window_size)
                //             .filter_map(|i| all_samples.get(samples_offset + i * channels as usize)) // Get every 'channels'-th sample starting from 'samples_offset'
                //             .map(|&x| Complex::new(x as f32, 0.0))
                //             .collect();

                //         if buffer.len() == window_size {
                //             fft.process(&mut buffer); // Perform FFT in-place

                //             // Display the frequency and magnitude information
                //             display_frequencies(
                //                 &buffer,
                //                 sample_rate as usize,
                //                 window_size,
                //                 fft_output.clone(),
                //             );
                //         } else {
                //             println!(
                //                 "Insufficient samples for a complete buffer after de-interleaving."
                //             );
                //         }
                //     } else {
                //         println!(
                //             "Not enough data available for FFT calculation at the current position."
                //         );
                //     }

                info!("FFT Complete: Time taken to complete: {:?}", t1.elapsed());
                let mut lock = last_fft_generated_at.lock().unwrap();
                // if the last fft generated was at Some(instant), then we can calculate the time difference
                if let Some(last_time) = *lock {
                    let time_diff = Instant::now() - last_time;
                    info!("Time since last FFT : {:?} ", time_diff);
                }
                *lock = Some(Instant::now());
            }
            Command::Play => {
                println!("Playing audio");
                sink.play();
            }
            Command::Pause => {
                println!("Pausing audio");
                // HOW does pause work if we're not keeping track of playback_position?
                // actually playback position does stop updating when paused
                // but why?!
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
            .map(|x| x.mag)
            .collect();

        let octaves: Vec<Vec<f32>> = octaves_flat.chunks(12).map(|x| x.to_vec()).collect();
        // println!("--octaves: {:?}", octaves);
        assert!(octaves.len() == 4, "Expected 4 octaves");

        let data = render_drawing::Data::new(octaves);

        // println!("--final data before drawing: {:?}", data);
        // println!("--fft_output: {:?}", octaves_flat);
        render_drawing::draw_on_window(app, frame, &data, &model.ui_elements);
    }
}
