import numpy as np
from scipy.signal import chirp
from scipy.io.wavfile import write
from pydub import AudioSegment
import os
import shutil

import numpy as np

def generate_sine_wave(frequency, duration, sampling_rate=44100):
    """
    Generate a sine wave for a given frequency.

    Parameters:
    - frequency (float): Frequency of the sine wave in Hz.
    - duration (float): Duration of the sine wave in seconds.
    - sampling_rate (int): Number of samples per second.

    Returns:
    - np.ndarray: Array containing the sine wave.
    """
    # Calculate the total number of samples
    n_samples = int(sampling_rate * duration)
    
    # Generate an array of sample indices
    t = np.linspace(0, duration, n_samples, endpoint=False)
    
    # Calculate the sine wave
    wave = np.sin(2 * np.pi * frequency * t)
    
    return wave


# concatanate multiple sine waves
def concatenate_sine_waves(waves):
    """
    Concatenate multiple sine waves into a single signal.

    Parameters:
    - waves (List[np.ndarray]): List of sine waves to concatenate.

    Returns:
    - np.ndarray: Array containing the concatenated signal.
    """
    return np.concatenate(waves)

# write a wave into a wav file
# there can be multiple channels
# it will require N waves for each of N channels
def write_waves_to_wav(waves, filename, sample_rate=44100):
    """ 
    Write a list of signals to a WAV file. Each signal will be written to a separate channel.

    Parameters:
    - waves (List[np.ndarray]): List of signals to write. Each element is an array representing a channel.
    - filename (str): Name of the WAV file to save.
    - sample_rate (int): Sampling rate (samples per second).

    Returns:
    - None
    """
    # Ensure all arrays have the same length by trimming to the shortest array
    min_length = min(map(len, waves))
    waves = [wave[:min_length] for wave in waves]

    # Convert the list of arrays to a single Numpy array with shape (n_samples, n_channels)
    wave_array = np.stack(waves, axis=-1)

    # Convert float values to a suitable 16-bit integer format
    wave_array = np.int16(wave_array / np.max(np.abs(wave_array)) * 32767)

    # Write the multi-channel data to a WAV file
    write(filename, sample_rate, wave_array)


def generate_scale_frequencies(base_freq=261.63, scale_steps=[0, 2, 4, 5, 7, 9, 11, 12]):
    """
    Generate frequencies for a musical scale based on a starting frequency.

    Parameters:
    - base_freq (float): The frequency of the base note (default is C4 - middle C).
    - scale_steps (List[int]): Steps in the scale from the base note, in semitones.

    Returns:
    - List[float]: List of frequencies for the scale.
    """
    return [base_freq * (2 ** (step / 12.0)) for step in scale_steps]


if __name__ == "__main__":
    wave1 = generate_sine_wave(261.63, 5)  # C3 note, 1 second
    wave2 = generate_sine_wave(440, 5)  # A4 note, 1 second

    # Write these waves as stereo channels to a WAV file
    write_waves_to_wav([wave1, wave2], './src/output_stereo.wav')
