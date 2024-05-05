use nannou_audio::Buffer;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::BufReader;
use minimp3::{Decoder, Frame};
use std::collections::VecDeque;

 struct Audio {
    pub sounds: Vec<BufReader<File>>,
    pub volume_sender: std::sync::mpsc::Sender<f32>,
    pub volume: Arc<Mutex<f32>>,
    pub fft_output: Arc<Mutex<Vec<Complex<f32>>>>,
}

pub fn render(_audio: &mut Audio, _buffer: &mut Buffer) {
    println!("i am in render");
    let file = File::open("src/test.mp3").unwrap();
    println!("MP3 file opened successfully.");

    let mut decoder = Decoder::new(file);
    println!("Decoder initialized.");

    let sample_rate = 44100;
    let channels = 2;
    let window_size = (0.5 * sample_rate as f32) as usize * channels;

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size / channels);

    let mut samples_buffer = VecDeque::new();
    let mut frame_count = 0;
    let mut chunk_count = 0;

    while let Ok(Frame { data, .. }) = decoder.next_frame() {
        for &sample in &data {
            samples_buffer.push_back(sample);
            if samples_buffer.len() >= window_size {
                let chunk: Vec<i16> = samples_buffer.drain(..window_size).collect();

                let mut buffer: Vec<_> = chunk.iter()
                    .map(|&x| Complex::new(x as f32, 0.0))
                    .collect();

                buffer.resize(window_size / channels, Complex::new(0.0, 0.0));
                fft.process(&mut buffer);

                println!("FFT processed on chunk {}.", chunk_count);
                display_frequencies(&buffer, sample_rate, window_size / channels);

                chunk_count += 1;
            }
        }
        frame_count += 1;
    }

    println!("Total frames processed: {}", frame_count);
    println!("Total chunks processed: {}", chunk_count);
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


