use nannou::prelude::*;

pub struct Data {
    pub amp: f32,
}

pub fn draw(app: &App, frame: Frame, data: &Data) {
    let amp = data.amp;
    let draw = app.draw();
    draw.background().color(CORNFLOWERBLUE);
    let win = app.window_rect();
    draw.ellipse()
        .x_y(0 as f32, 0 as f32)
        .radius(win.w() * amp)
        .color(RED);
    draw.to_frame(app, &frame).unwrap();
}
