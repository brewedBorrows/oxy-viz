
use std::sync::mpsc::{self, channel, Receiver, Sender};
pub struct Playback {
    is_playing: bool,
    curr_pos: Arc<Mutex<Duration>>,
    fft_output: Arc<Mutex<Vec<Complex<f32>>>>,
}

pub struct Model {
    pub sender: Sender<Command>,
    pub playback: Playback,
    pub data: render_drawing::Data,
    pub temp: u128,
    pub buttons: Vec<ui::Button>,
}