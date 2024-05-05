import numpy as np
from scipy.io import wavfile
from scipy.signal import chirp
from scipy.io.wavfile import write
from pydub import AudioSegment

def generate_sine_wave(duration, frequency, sample_rate=44100):
    """
    Generate a sine wave.

    Args:
        duration (float): Duration of the sine wave in seconds.
        frequency (float): Frequency of the sine wave in Hz.
        sample_rate (int): Sampling rate (samples per second).

    Returns:
        numpy.array: Array containing the generated sine wave.
    """
    t = np.linspace(0, duration, int(sample_rate * duration), endpoint=False)
    sine_wave = np.sin(2 * np.pi * frequency * t)
    return sine_wave

def export_as_mp3(signal, filename, sample_rate=44100):
    """
    Export a signal as an mp3 file.

    Args:
        signal (numpy.array): Signal to be exported.
        filename (str): Name of the mp3 file to be saved.
        sample_rate (int): Sampling rate (samples per second).

    Returns:
        None
    """
    # Convert the signal to the correct format for pydub
    signal = (signal * 32767).astype(np.int16)

    # Write the signal to a wav file
    write(filename + '.wav', sample_rate, signal)

    # Load the wav file using pydub
    sound = AudioSegment.from_wav(filename + '.wav')

    # Export the sound as an mp3 file
    sound.export(filename + '.mp3', format="mp3")

if __name__ == "__main__":
    duration = 5  # Duration of the sine wave in seconds
    frequency = 261.63  # Frequency of the sine wave in Hz
    sample_rate = 44100  # Sampling rate (samples per second)

    sine_wave = generate_sine_wave(duration, frequency, sample_rate)
    export_as_mp3(sine_wave, "sine_wave")
