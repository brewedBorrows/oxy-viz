import subprocess
import numpy as np
from scipy.io.wavfile import write
import numpy as np
from scipy.io import wavfile
from scipy.signal import chirp
import shutil
import subprocess
import os


# Constants
sample_rate = 44100  # Sampling rate in Hz
duration = 5  # Duration of each sine wave in seconds

def generate_sine_wave(frequency, sample_rate, duration):
    t = np.linspace(0, duration, int(sample_rate * duration), endpoint=False)
    return 0.5 * np.sin(2 * np.pi * frequency * t)  # Sine wave formula

def concatenate_waves(frequencies, sample_rate, duration):
    # Generate sine waves and concatenate them
    wave = np.concatenate([generate_sine_wave(freq, sample_rate, duration) for freq in frequencies])
    return wave

def create_wave_file(filename, frequencies):
    wave = concatenate_waves(frequencies, sample_rate, duration)
    # Scale to 16-bit integers
    wave_integers = np.int16(wave * 32767)
    # Write to a .wav file
    write(filename, sample_rate, wave_integers)

# Example usage:
frequencies = [261.63, 277.18, 293.66, 311.13, 329.63, 349.23, 369.99, 392.00, 415.30, 440.00, 466.16,
        493.88]  # List of frequencies in Hz
create_wave_file('./src/test.wav', frequencies)

# Run cargo run command
subprocess.run(["cargo", "run"])
