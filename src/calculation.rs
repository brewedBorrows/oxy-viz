pub fn calculate(playing: &bool) -> Option<f32> {
    match playing {
        true => Some(0.5),
        false => Some(0.125),
    }
}