use nannou::prelude::*;
use rand::Rng;
use splines::{Interpolation, Key, Spline};

use crate::ui;

#[derive(Debug)]
pub struct Data {
    octaves: Vec<Vec<f32>>,
    octaves_len: usize,
}

impl Data {
    pub fn new(octaves: Vec<Vec<f32>>) -> Self {
        // MUST initiate Data with ::new() to set octaves_len
        let octaves_len = octaves.len();
        Data {
            octaves,
            octaves_len,
        }
    }

    pub fn create_random_data() -> Data {
        // Generates a random Data instance with 4 octaves, each containing 12 random f32 values between 5 and 100
        let mut rng = rand::thread_rng();
        let num_octaves = 4;
        let values_per_octave = 12;
        let min_value = 0.0;
        let max_value = 1.0;

        let octaves = (0..num_octaves)
            .map(|_| {
                (0..values_per_octave)
                    .map(|_| {
                        rng.gen_range(min_value..=max_value) // Generate a random f32 between 0.0 and 1.0
                    })
                    .collect::<Vec<f32>>()
            })
            .collect::<Vec<Vec<f32>>>();

        Data {
            octaves,
            octaves_len: num_octaves,
        }
    }

    /// normalize each amplitude in an octave from MIN to MAX
    fn normalize(octave: Vec<f32>) -> Vec<f32> {
        let min = 0.;
        let max = octave
            .iter()
            .fold(0., |acc, &val| if acc > val { acc } else { val });

        octave
            .into_iter()
            .map(|amp| map_range(amp, min, max, 0., 1.))
            .collect()
    }
}

struct Spline2D {
    x_spline: Spline<f32, f32>,
    y_spline: Spline<f32, f32>,
    length: usize,
}

impl Spline2D {
    fn new(points: Vec<Vec2>) -> Self {
        // will generate Splines for each x values and y values
        // It will include keys = 0 , 1, 2, ... 11 , 0, 1, 2
        // to make the spline curve closed
        // EG, if keys include [0, 1, 2, ... , 10, 11]
        // keys iterpolation would start from 1.0, 1.1, .. (including 1.0)
        // and ends at 9.99, 10.0 (excluding 10.0)

        //validate: points.len() >= 3
        if points.len() < 3 {
            panic!("--points.len() must be greater than or equal to 3");
        }

        let spline_single_axis = |points: &Vec<Vec2>, axis: usize| {
            let mut vals: Vec<f32> = points.iter().map(|p| p[axis]).collect();
            vals.push(points[0][axis]);
            vals.push(points[1][axis]);
            vals.push(points[2][axis]);
            Spline::from_iter(
                vals.iter()
                    .enumerate()
                    .map(|(i, val)| Key::new(i as f32, *val, Interpolation::CatmullRom)),
            )
        };

        let x_spline = spline_single_axis(&points, 0);
        let y_spline = spline_single_axis(&points, 1);

        // testing
        // let sample_x = |val| {
        // println!("--val: {:?}", val);
        // println!(
        //     "--x: {:?}",
        //     x_spline.sample_with_key(val).unwrap_or(SampledWithKey {
        //         value: 99.9,
        //         key: 0
        //     })
        // )
        // };
        // dbg!("--spline_x: {:?}", &x_spline);
        // sample_x(0.0);
        // sample_x(0.1);
        // sample_x(0.9);
        // sample_x(1.);
        // sample_x(2.);
        // sample_x(3.);
        // sample_x(4.5);
        // sample_x(4.6);
        // sample_x(5.5);
        // sample_x(9.9);
        // sample_x(10.0);
        // println!("--y: {}", y_spline.sample(11.5).unwrap_or(99.9));

        Self {
            x_spline,
            y_spline,
            length: points.len(),
        }
    }

    fn generate_samples(&self, num_samples: usize) -> Vec<Vec2> {
        /// returns a list of Vec2 samples from the spline curve
        let mut result: Vec<Vec2> = vec![];

        let start = 1.0; // note that interpolation starts at 1.0 not 0.0 (incl 1.0)
        let end = (self.length + 1) as f32; // interpolation also doesn't include 10.0 onwards (excl 10.0)
        let step = (end - start) / (num_samples as f32 - 1.0);

        for i in 0..num_samples - 1 {
            // 0 to 198
            let t = start + i as f32 * step;
            let x = self.x_spline.sample(t).expect("None at x?");
            let y = self.y_spline.sample(t).expect("None at y?");
            // let x = spline.x_spline.sample(t).unwrap();
            // let y = spline.y_spline.sample(t).unwrap();
            result.push(Vec2::new(x, y));
        }
        // insert last point (end - epslon)
        let x = self.x_spline.sample(end - 0.0001).expect("None at x?");
        let y = self.y_spline.sample(end - 0.0001).expect("None at y?");
        result.push(Vec2::new(x, y));

        // dbg!("--spline x: {:?}", &spline.x_spline);
        // let x = spline.x_spline.sample(5.6).expect("None at x?");
        // println!("--x: {}", x);
        // let y = spline.y_spline.sample(0.5).expect("None at y?");
        // println!("-- y: {}", y);
        // result.push(Vec2::new(x, y));
        result
    }
}

enum Visualization {
    CircleWave(CircleWave),
    CircleWaveMultiple(CircleWaveMultiple),
}

pub struct DrawConfig {
    frame_rate: usize,
    color_scheme: String,
    resolution: usize,
    num_samples: usize, // number of samples to generate from spline curve
}

struct PointPolar {
    r: f32,
    theta: f32,
}

impl PointPolar {
    fn to_cartesian(&self) -> Point2 {
        Point2::new(self.r * self.theta.cos(), self.r * self.theta.sin())
    }
}

struct CircleWave {
    points: Vec<PointPolar>,
    radius: f32,
}

struct CircleWaveMultiple {
    circle_waves: Vec<CircleWave>,
}

pub trait DrawVisual {
    // this should manage no of samples etc right
    fn draw_visual(&self, draw: &Draw, win: Rect, config: &DrawConfig);
    fn scale_visual(&mut self, win: Rect);
}

impl CircleWave {
    /// max_amp: max amplitude of the wave from the circumference of the circle
    pub fn new(input_data: &Data, radius: f32, max_amp: f32) -> Self {
        let (octaves, octaves_len) = (&input_data.octaves, input_data.octaves_len);
        if octaves_len > 1 {
            panic!("--CircleWave can only accept 1 octave");
        }

        let norm_octave = Data::normalize(octaves[0].clone());
        // creates a copy.
        // do we want CircleWave to store a normalized copy of Data.octaves[i],
        // or should CircleWave mutate the Data.octaves[i] in place?

        // working in r, theta coordinates
        let num_points = &norm_octave.len();
        let mut theta_list = (0..*num_points).map(|i| (i as f32) * TAU / (*num_points as f32));
        let mut r_list = norm_octave.into_iter().map(|amp| amp * 185.377700 + radius);

        let points: Vec<PointPolar> = (0..*num_points)
            .map(|_| PointPolar {
                r: r_list.next().unwrap(),
                theta: theta_list.next().unwrap(),
            })
            .collect();

        CircleWave { points, radius }
    }
}

impl DrawVisual for CircleWave {
    fn draw_visual(&self, draw: &Draw, win: Rect, config: &DrawConfig) {
        // self.scale_visual(win);
        let points = self.points.iter().map(|p| p.to_cartesian()).collect();

        // feed points to spline_generator
        let spline = Spline2D::new(points);

        // get list of spline_curve_samples from spline_generator
        let spline_samples = spline.generate_samples(config.num_samples);
        // println!("--spline_samples: {:?}", spline_samples);

        // draw polyline from spline_curve_samples
        draw.polyline()
            .color(RED)
            .stroke_weight(5.0)
            .points(spline_samples);

        draw.ellipse()
            .x_y(0 as f32, 0 as f32)
            .radius(win.w() * 0.0125)
            .color(RED);
    }

    fn scale_visual(&mut self, win: Rect) {
        // scale the circle wave
        let scale_factor = 1.5;
        self.points = self
            .points
            .iter_mut()
            .map(|p| PointPolar {
                r: p.r * scale_factor,
                theta: p.theta,
            })
            .collect();
    }
}

impl CircleWaveMultiple {
    pub fn new(input_data: &Data, radii: Vec<f32>, max_amps: Vec<f32>) -> Self {
        let (octaves, octaves_len) = (&input_data.octaves, input_data.octaves_len);

        let circle_waves = (0..octaves_len)
            .map(|i| {
                let norm_octave = Data::normalize(octaves[i].clone());
                let radius = radii[i];
                let max_amp = max_amps[i];
                CircleWave::new(&Data::new(vec![norm_octave]), radius, max_amp)
            })
            .collect();

        CircleWaveMultiple { circle_waves }
    }
}

impl DrawVisual for CircleWaveMultiple {
    fn draw_visual(&self, draw: &Draw, win: Rect, config: &DrawConfig) {
        for circle_wave in &self.circle_waves {
            circle_wave.draw_visual(draw, win, config);
        }
    }

    fn scale_visual(&mut self, win: Rect) {
        for circle_wave in &mut self.circle_waves {
            circle_wave.scale_visual(win);
        }
    }
}

pub fn draw_on_window(app: &App, frame: Frame, data: &Data, ui_elements: &Vec<ui::UIElem>) {
    let draw = app.draw();
    draw.background().color(CORNFLOWERBLUE);
    let win = app.window_rect();

    let radii = data.octaves.iter().enumerate().map(|(i, _)| win.w() / 8.0 + i as f32 * 50.).collect();
    let max_amps = data.octaves.iter().map(|_| win.w() / 16.0).collect();
    let draw_config = DrawConfig {
        frame_rate: 60,
        color_scheme: "red".to_string(),
        resolution: 200,
        num_samples: 400,
    };
    CircleWaveMultiple::new(&data, radii, max_amps).draw_visual(
        &draw,
        win,
        &draw_config,
    );
    ui::render_ui(&draw,
        win,
        &draw_config,
        ui_elements,
    );

    draw.to_frame(app, &frame).unwrap();
}
