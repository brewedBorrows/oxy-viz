use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{ChannelCount, SampleFormat, SampleRate, SizedSample, Stream, StreamConfig};
use itertools::Itertools;
use rodio::source::UniformSourceIterator;
use rodio::{Decoder, Source};
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};
use universal_audio_decoder::{new_uniform_source_iterator, TrueUniformSourceIterator};
use std::f32::consts;
use hound::{WavWriter, WavSpec};
use std::io::{Read, Write};


pub struct AudioManager {
    pub stream_config: StreamConfig,
    sender_to_audio: Sender<MessageToAudio>,
    drop_sender: Sender<()>,
    playback_position: Arc<Mutex<PlaybackPosition>>,
}

enum PlaybackPosition {
    NotStarted,
    Seeking {
        music_position: f64,
    },
    Paused {
        music_position: f64,
    },
    Playing {
        music_position: f64,
        instant: Instant,
        play_speed: f64,
    },
}

enum MessageToAudio {
    Play,
    Pause,
    Seek(f64),
    LoadMusic(PathBuf),
    SetMusicVolume(f64),
    SetPlaySpeed(f64),
}

type MusicSource = TrueUniformSourceIterator<Decoder<BufReader<File>>>;

struct AudioThreadState {
    stream_config: StreamConfig,

    music: Option<MusicSource>,

    receiver_to_audio: mpsc::Receiver<MessageToAudio>,
    playing: bool,
    played_sample_count: usize,
    skip_sample_count: usize,
    playback_position_ptr: Weak<Mutex<PlaybackPosition>>,
    music_volume: f32,
    play_speed: f64,
}

#[derive(Debug)]
enum Error {
    AudioError(String),
}

impl AudioThreadState {
    pub fn new(
        stream_config: StreamConfig,
        receiver_to_audio: mpsc::Receiver<MessageToAudio>,
        playback_position_ptr: Weak<Mutex<PlaybackPosition>>,
    ) -> Self {
        AudioThreadState {
            stream_config,
            music: None,
            receiver_to_audio,
            playing: false,
            played_sample_count: 0,
            skip_sample_count: 0,
            playback_position_ptr,
            music_volume: 1.0,
            play_speed: 1.0,
        }
    }

    fn data_callback<S>(mut self) -> impl FnMut(&mut [S], &cpal::OutputCallbackInfo)
    where
        S: SizedSample + cpal::FromSample<f32>,
    {
        move |output, callback_info| {
            for message in self.receiver_to_audio.try_iter() {
                match message {
                    MessageToAudio::Play => {
                        self.playing = true;
                    }
                    MessageToAudio::Pause => {
                        self.playing = false;
                        self.update_pause_state();
                    }
                    MessageToAudio::Seek(time) => {
                        if let Err(e) = if let (Some(music), false) = (&mut self.music, false) {
                            match music
                                .seek(time.max(0.0))
                                .map_err(|e| Error::AudioError("couldn't seek".to_string()))
                            {
                                Ok(sample_count) => {
                                    self.skip_sample_count = (-time.min(0.0)
                                        * self.stream_config.sample_rate.0 as f64
                                        / self.play_speed)
                                        as usize
                                        * (self.stream_config.channels as usize);
                                    self.played_sample_count = sample_count as usize;
                                    self.update_pause_state();
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        } else {
                            Err(Error::AudioError("No music loaded".to_string()))
                        } {
                            println!("Failed to seek: {:?}", e);
                        }
                    }
                    MessageToAudio::LoadMusic(path) => {
                        self.music = Some(self.load_music(path).unwrap())
                    }
                    MessageToAudio::SetMusicVolume(volume) => self.music_volume = volume as f32,
                    MessageToAudio::SetPlaySpeed(speed) => {
                        self.play_speed = speed;
                        if let Some(music) = &mut self.music {
                            music.set_output_sample_rate(
                                self.stream_config.sample_rate.0 as f64 / speed,
                            );
                        };
                    }
                }
            }

            if self.playing {
                let timestamp = callback_info.timestamp();
                let instant = Instant::now()
                    + timestamp
                        .playback
                        .duration_since(&timestamp.callback)
                        .unwrap_or_else(|| Duration::from_nanos(0));

                let playing_sample_count = output.len() / (self.stream_config.channels as usize);

                let music_position_start = self.music_position_start() * self.play_speed;
                let music_position_end = (self.played_sample_count + playing_sample_count) as f64
                    / self.stream_config.sample_rate.0 as f64
                    * self.play_speed;

                if let Some(playback_position) = self.playback_position_ptr.upgrade() {
                    let mut playback_position = playback_position
                        .lock()
                        .map_err(|e| format!("The main thread has been panicked: {}", e))
                        .unwrap(); // Intentionally panic when error
                    *playback_position = PlaybackPosition::Playing {
                        instant,
                        music_position: music_position_start,
                        play_speed: self.play_speed,
                    };
                }

                // TODO: SPAGHETTI CODE!
                self.played_sample_count += output.len().saturating_sub(self.skip_sample_count)
                    / (self.stream_config.channels as usize)
            }

            for out in output.iter_mut() {
                let mut next = match &mut self.music {
                    Some(music) if self.playing => {
                        if self.skip_sample_count > 0 {
                            self.skip_sample_count -= 1;
                            None
                        } else {
                            music.next().map(|a| a * self.music_volume)
                        }
                    }
                    _ => None,
                }
                .unwrap_or(0.0)
                .clamp(-4.0, 4.0); // Prevent too large sound

                *out = S::from_sample(next);
            }
        }
    }

    fn update_pause_state(&self) {
        if let Some(playback_position) = self.playback_position_ptr.upgrade() {
            let mut playback_position = playback_position
                .lock()
                .map_err(|e| Error::AudioError(format!("Error locking playback position: {}", e)))
                .unwrap();
            *playback_position = PlaybackPosition::Paused {
                music_position: self.music_position_start(),
            };
        }
    }

    fn music_position_start(&self) -> f64 {
        let sample_index = self.played_sample_count as isize
            - self.skip_sample_count as isize / self.stream_config.channels as isize;
        sample_index as f64 / self.stream_config.sample_rate.0 as f64
    }

    pub fn load_music(&self, wave: PathBuf) -> Result<MusicSource, Error> {
        let file = std::fs::File::open(wave)
            .map_err(|e| Error::AudioError(format!("Error opening file: {}", e)))?;
        let decoder = rodio::Decoder::new(BufReader::new(file))
            .map_err(|e| Error::AudioError(format!("Error creating decoder: {}", e)))?;
        let ret = new_uniform_source_iterator(decoder, &self.stream_config);
        Ok(ret)
    }
}

impl AudioManager {
    pub fn new() -> Result<Self, Error> {
        let (sender_to_audio, receiver_to_audio) = mpsc::channel();
        let (stream_config_sender, stream_config_receiver) = mpsc::channel();
        let (drop_sender, drop_receiver) = mpsc::channel();
        let playback_position = Arc::new(Mutex::new(PlaybackPosition::NotStarted));

        let playback_position_ptr = Arc::downgrade(&playback_position);

        thread::spawn(move || {
            match stream_thread(receiver_to_audio, playback_position_ptr) {
                Ok((stream_config, _stream)) => {
                    if stream_config_sender.send(Ok(stream_config)).is_err() {
                        eprintln!("Failed to send stream config to main thread");
                    }
                    // Wait for the main thread to drop the AudioManager
                    drop_receiver.recv().ok();
                }
                Err(err) => {
                    if stream_config_sender.send(Err(err)).is_err() {
                        eprintln!("Failed to send error to main thread");
                    }
                }
            }
        });
        let stream_config = stream_config_receiver.recv().map_err(|e| {
            Error::AudioError(format!(
                "audio device initialization thread: Error receiving stream config: {}",
                e
            ))
        })??;

        Ok(AudioManager {
            stream_config,
            sender_to_audio,
            drop_sender,
            playback_position,
        })
    }

    pub fn load_music<P>(&self, path: P) -> Result<(), Error>
    where
        P: Into<PathBuf>,
    {
        self.sender_to_audio
            .send(MessageToAudio::LoadMusic(path.into()))
            .map_err(|_| Error::AudioError("Failed to send load music message".to_string()))
    }

    pub fn play(&self) -> Result<(), Error> {
        self.sender_to_audio
            .send(MessageToAudio::Play)
            .map_err(|e| Error::AudioError("Failed to play music".to_string()))
    }

    pub fn pause(&self) -> Result<(), Error> {
        self.sender_to_audio
            .send(MessageToAudio::Pause)
            .map_err(|e| Error::AudioError("Failed to pause music".to_string()))
    }

    pub fn seek(&self, time: f64) -> Result<(), Error> {
        {
            // TODO there should be a better way
            let mut playback_position = self.playback_position.lock().map_err(|_| {
                Error::AudioError(
                    "Failed to obtain music position; the audio stream has been panicked"
                        .to_string(),
                )
            })?;
            *playback_position = PlaybackPosition::Seeking {
                music_position: time,
            };
        }
        self.sender_to_audio
            .send(MessageToAudio::Seek(time))
            .map_err(|_| {
                Error::AudioError("Failed to seek music, audio stream stopped".to_string())
            })
    }

    pub fn playing(&self) -> Result<bool, Error> {
        let playback_position = self.playback_position.lock().map_err(|_| {
            Error::AudioError(
                "Failed to obtain music position; the audio stream has been panicked".to_string(),
            )
        })?;
        Ok(matches!(
            *playback_position,
            PlaybackPosition::Playing { .. }
        ))
    }

    pub fn set_music_volume(&self, volume: f32) -> Result<(), Error> {
        self.sender_to_audio
            .send(MessageToAudio::SetMusicVolume(volume as f64))
            .map_err(|_| {
                Error::AudioError("Failed to set music volume; audio stream stopped".to_string())
            })
    }

    pub fn set_play_speed(&self, speed: f64) -> Result<(), Error> {
        self.sender_to_audio
            .send(MessageToAudio::SetPlaySpeed(speed))
            .map_err(|_| {
                Error::AudioError(
                    "Failed to set play speed; the audio stream has been stopped".to_string(),
                )
            })
    }

    /// Returns error only if the audio stream has been pannicked.
    pub fn music_position(&self) -> Result<Option<f64>, Error> {
        let playback_position = self.playback_position.lock().map_err(|_| {
            Error::AudioError(
                "Failed to obtain music position; the audio stream has been panicked".to_string(),
            )
        })?;
        use PlaybackPosition::*;
        let res = match *playback_position {
            Playing {
                music_position,
                instant,
                play_speed,
            } => {
                let now = Instant::now();
                let diff = if now > instant {
                    (now - instant).as_secs_f64() * play_speed
                } else {
                    -(instant - now).as_secs_f64() * play_speed
                };
                Some(music_position + diff)
            }
            Paused { music_position } | Seeking { music_position } => Some(music_position),
            NotStarted => None,
        };
        Ok(res)
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        if self.drop_sender.send(()).is_err() {
            eprintln!("Failed to send drop signal to audio thread");
        }
    }
}

fn stream_thread(
    receiver_to_audio: Receiver<MessageToAudio>,
    playback_position_ptr: Weak<Mutex<PlaybackPosition>>,
) -> Result<(StreamConfig, Stream), Error> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| Error::AudioError("No output device found".to_string()))?;
    let mut supported_configs_range = device
        .supported_output_configs()
        .map_err(|e| Error::AudioError(format!("Error getting supported output configs: {}", e)))?;
    let supported_config = supported_configs_range
        .next()
        .ok_or_else(|| Error::AudioError("No supported output config found".to_string()))?
        .with_max_sample_rate();
    dbg!("Supported config: {:?}", supported_config.clone());
    let sample_format = supported_config.sample_format();
    let stream_config: StreamConfig = supported_config.into();
    let state = AudioThreadState::new(
        stream_config.clone(),
        receiver_to_audio,
        playback_position_ptr,
    );
    let error_callback = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let stream = match sample_format {
        SampleFormat::F32 => device.build_output_stream(
            &stream_config,
            state.data_callback::<f32>(),
            error_callback,
            None,
        ),
        SampleFormat::F64 => device.build_output_stream(
            &stream_config,
            state.data_callback::<f32>(),
            error_callback,
            None,
        ),
        SampleFormat::I8 => device.build_output_stream(
            &stream_config,
            state.data_callback::<i8>(),
            error_callback,
            None,
        ),
        SampleFormat::I16 => device.build_output_stream(
            &stream_config,
            state.data_callback::<i16>(),
            error_callback,
            None,
        ),
        SampleFormat::I32 => device.build_output_stream(
            &stream_config,
            state.data_callback::<i32>(),
            error_callback,
            None,
        ),
        SampleFormat::I64 => device.build_output_stream(
            &stream_config,
            state.data_callback::<i64>(),
            error_callback,
            None,
        ),
        SampleFormat::U8 => device.build_output_stream(
            &stream_config,
            state.data_callback::<u8>(),
            error_callback,
            None,
        ),
        SampleFormat::U16 => device.build_output_stream(
            &stream_config,
            state.data_callback::<u16>(),
            error_callback,
            None,
        ),
        SampleFormat::U32 => device.build_output_stream(
            &stream_config,
            state.data_callback::<u32>(),
            error_callback,
            None,
        ),
        SampleFormat::U64 => device.build_output_stream(
            &stream_config,
            state.data_callback::<u64>(),
            error_callback,
            None,
        ),
        _ => {
            // TODO: is this the right way?
            return Err(Error::AudioError("Unsupported sample format".to_string()));
        }
    };
    let stream =
        stream.map_err(|e| Error::AudioError(format!("Error building output stream: {}", e)))?;
    stream
        .play()
        .map_err(|e| Error::AudioError(format!("Error playing output stream: {}", e)))?;
    Ok((stream_config, stream))
}


// Define a function to generate a sine wave for a given frequency and duration
fn generate_sine_wave(frequency: f32, duration_secs: f32, sample_rate: u32) -> Vec<i16> {
    let mut samples = Vec::new();
    for n in 0..(duration_secs * sample_rate as f32) as u32 {
        let value = (2.0 * consts::PI * frequency * (n as f32) / sample_rate as f32).sin();
        let amplitude = i16::MAX as f32;
        samples.push((value * amplitude) as i16);
    }
    samples
}

// Function to generate a wav file from a series of frequencies representing musical notes
pub fn generate_scale(frequencies: Vec<f32>, duration_secs: f32, sample_rate: u32) -> Result<PathBuf, std::io::Error> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let path = PathBuf::from("scale.wav");
    let mut writer = WavWriter::create(path.clone(), spec).unwrap();

    for &frequency in &frequencies {
        let samples = generate_sine_wave(frequency, duration_secs, sample_rate);
        for sample in samples {
            writer.write_sample(sample).unwrap();
        }
    }

    writer.finalize().unwrap();
    Ok(path)
}

fn main() {


    let sample_rate = 44100;
    // Frequencies for the C major scale: C, D, E, F, G, A, B, C
    let frequencies = vec![261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88, 523.25];
    
    // Duration of each note in seconds
    let note_duration = Duration::from_secs_f32(0.5);


    // Generate the scale
    let path = generate_scale(frequencies, note_duration.as_secs_f32(), sample_rate).unwrap();

    // run the audio manager
    let audio_manager = AudioManager::new().unwrap();

    // wait for 5 seconds
    std::thread::sleep(Duration::from_secs(6));
}
