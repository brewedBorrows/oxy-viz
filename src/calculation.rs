use std::sync::{Arc, Mutex};
use rustfft::num_complex::Complex;

fn process_fft_output(fft_output: &[f32], prev_power_spectrum: &mut Vec<f32>) -> f32 {
    let mut power_spectrum = Vec::new();
    let mut spectral_flux = 0.0;

    for &value in fft_output {
        let power = value * value;
        power_spectrum.push(power);
    }

    if !prev_power_spectrum.is_empty() {
        for i in 0..power_spectrum.len() {
            let flux = power_spectrum[i] - prev_power_spectrum[i];
            if flux > 0.0 {
                spectral_flux += flux;
            }
        }
    }

    *prev_power_spectrum = power_spectrum;

    spectral_flux
}


pub fn calculate(playing: &bool, volume: &Arc<Mutex<f32>>, fft_output: &Arc<Mutex<Vec<Complex<f32>>>>) -> Option<f32> {
     // Early return if not playing.
     if !*playing {
        return None;
    }

    // Lock and read the FFT output.
    let fft_output_guard = fft_output.lock().unwrap();
    let fft_magnitudes: Vec<f32> = fft_output_guard.iter().map(|c| c.norm()).collect();

    // Lock and read the volume.
    let volume_guard = volume.lock().unwrap();
    let amplitude = if *volume_guard > 0.0 {
        (*volume_guard).log(10.0) * 14.5
    } else {
        1.0
    };

    // Calculate the weighted average amplitude from FFT magnitudes.
    let total_magnitude: f32 = fft_magnitudes.iter().sum();
    let average_magnitude = if !fft_magnitudes.is_empty() {
        total_magnitude / fft_magnitudes.len() as f32
    } else {
        0.0
    };

    // Apply the calculated amplitude to the average magnitude.
    let weighted_amplitude = average_magnitude * amplitude;



    match playing {
        true => Some(weighted_amplitude),
        false => Some(0.125),
    }
}