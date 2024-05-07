use crate::render_drawing::{DrawConfig, DrawVisual};
use nannou::prelude::*;

pub const PADDING: f32 = 50.;
pub const BUTTON_W: f32 = 50.;
pub const BUTTON_H: f32 = 50.;

#[derive(Debug)]
pub struct Button {
    pub button_name: ButtonName,
    pub bbox: BBox,
    pub onclick: fn(),
}

impl Button {
    pub fn new(button_name: ButtonName, bbox: BBox, onclick: fn()) -> Self {
        Button {
            button_name,
            bbox,
            onclick,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum ButtonName {
    Play,
    FavRecord,
    FavPlay,
    Seek,
}

#[derive(Debug)]
pub struct SeekLine {
    pub bbox: BBox,
    pub button: Button,
    width: f32,
}

#[derive(Debug)]
pub enum UIElem {
    Button(Button),
    SeekLine(SeekLine),
    SongInfo(SongInfo),
    TimeStamp(TimeStamp),
}

impl DrawVisual for UIElem {
    fn draw_visual(&self, draw: &Draw, win: Rect, config: &DrawConfig) {
        match self {
            UIElem::Button(button) => button.draw_visual(draw, win, config),
            UIElem::SeekLine(seekline) => seekline.draw_visual(draw, win, config),
            UIElem::SongInfo(songinfo) => {}
            UIElem::TimeStamp(timestamp) => {}
        }
    }
    fn scale_visual(&mut self, win: Rect) {}
}

#[derive(Debug)]
struct TimeStamp {
    pub time_elapsed: String,
    pub total_time: String,
}

#[derive(Debug)]
struct SongInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub genre: String,
}

impl SeekLine {
    /// create a new SeekLine with width w and playback position playback_pos (0.0 to 1.0)
    pub fn new(win: Rect) -> Self {
        let line_width = win.w() - 2. * PADDING;

        let line_bbox = BBox::new(0., 0., line_width, 10.)
            .to_bottom_left(win)
            .translate(0., BUTTON_W + PADDING);

        let (x, y, _, _) = line_bbox.translate(0.,0.).to_tuple();

        let scroller_bbox = BBox::new(x, y, 20., 20.);

        SeekLine {
            bbox: line_bbox,
            button: Button::new(ButtonName::Play, scroller_bbox, || {}),
            width: line_width,
        }
    }

    /// set the position of scroller based on playback_pos (0. to 1.) 
    pub fn set_scroller_pos(&mut self, playback_pos: f32) {
      // set x position of scroller, based on start position of seekline and playback_pos
        self.button.bbox.0 = self.bbox.0 + playback_pos * self.width;
    }

    /// get the playback position (0. to 1.) based on mouse x
    pub fn get_playback_pos(&self, x: f32) -> f32 {
      // mouse x  --->  distance from start of seekline
      let x = x - self.bbox.0;
      x / self.width
    }
}

impl DrawVisual for SeekLine {
    fn draw_visual(
        &self,
        draw: &nannou::prelude::Draw,
        win: nannou::prelude::Rect,
        config: &DrawConfig,
    ) {
        let (x, y, w, h) = self.bbox.for_draw_visual().to_tuple();

        draw.rect().x_y(x, y).w_h(w, h).color(BLUE);

        self.button.draw_visual(draw, win, config);
    }
    fn scale_visual(&mut self, win: nannou::prelude::Rect) {}
}

// util functions

#[derive(Debug)]
pub struct BBox(f32, f32, f32, f32);

impl BBox {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        BBox(x, y, w, h)
    }

    /// check if the point (x,y) is inside the bounding box
    pub fn contains(&self, x: f32, y: f32) -> bool {
        let (x_self, y_self, w, h) = (self.0, self.1, self.2, self.3);
        x > x_self && x < x_self + w && y > y_self && y < y_self + h
    }

    /// returns a bound box to draw the rectangle wi, with 0,0 at the bottom left of the Rect
    /// this is required becuase nannou draws the rectangle with 0,0 at the center
    /// but we want to draw the rectangle with 0,0 at the bottom left
    fn for_draw_visual(&self) -> BBox {
        let (x, y, w, h) = self.to_tuple();
        BBox(x + w / 2., y + h / 2., w, h)
    }

    pub fn to_tuple(&self) -> (f32, f32, f32, f32) {
        (self.0, self.1, self.2, self.3)
    }

    /// set x,y to the bottom left of the window
    pub fn to_bottom_left(&self, win: Rect) -> Self {
        let (_, _, w, h) = self.to_tuple();
        let x = -win.w() / 2. + PADDING;
        let y = -win.h() / 2. + PADDING;
        Self(x, y, w, h)
    }

    /// set x,y to bottom center of the window
    pub fn to_bottom_center(&self, win: Rect) -> Self {
        let (_, _, w, h) = self.to_tuple();
        let x = 0.;
        let y = -win.h() / 2. + PADDING;
        Self(x, y, w, h)
    }

    /// set x,y to bottom right of the window
    pub fn to_bottom_right(&self, win: Rect) -> Self {
        let (_, _, w, h) = self.to_tuple();
        let x = win.w() / 2. - PADDING;
        let y = -win.h() / 2. + PADDING;
        Self(x, y, w, h)
    }

    pub fn translate(&self, x: f32, y: f32) -> Self {
        let (x_self, y_self, w, h) = self.to_tuple();
        Self(x_self + x, y_self + y, w, h)
    }
}

impl DrawVisual for Button {

    fn draw_visual(
        &self,
        draw: &nannou::prelude::Draw,
        win: nannou::prelude::Rect,
        config: &DrawConfig,
    ) {
        let (x, y, w, h) = self.bbox.for_draw_visual().to_tuple();

        match self.button_name {
            ButtonName::Play => {
                draw.rect().x_y(x, y).w_h(w, w).color(RED);
            }
            ButtonName::FavPlay => {
                draw.rect().x_y(x, y).w_h(w, w).color(RED);
            }
            ButtonName::FavRecord => {
                draw.rect().x_y(x, y).w_h(w, w).color(BLUE);
            }
            ButtonName::Seek => {
                let (x, y, w, h) = self.bbox.to_tuple();
                println!("----seek button: {:?}", (x, y, w, h));
                draw.rect().x_y(x, y).w_h(w, w).color(GREEN);
            }
            _ => {
                draw.rect()
                    .x_y(0 as f32, -0 as f32)
                    .w_h(win.w() * 0.05, win.h() * 0.05)
                    .color(GREEN);
            }
        }
    }
    fn scale_visual(&mut self, win: nannou::prelude::Rect) {}
}

pub fn render_ui(
    draw: &nannou::prelude::Draw,
    win: nannou::prelude::Rect,
    config: &DrawConfig,
    ui_elements: &Vec<UIElem>,
) {
    for element in ui_elements {
        // dbg!("----drawing element: {:?}", element);
        element.draw_visual(&draw, win, &config);
    }
}

pub fn check_button_click(x: f32, y: f32, buttons: &Vec<Button>) -> Option<ButtonName> {
    println!("yeh le, {:?}", buttons.len());
    for button in buttons {
        dbg!("ek ek btn: {:?}", button);
    }
    for button in buttons {
        let bbox = &button.bbox;
        if bbox.contains(x, y) {
            println!("----matched! , {:?}", button);
            match button.button_name {
                ButtonName::Play => return Some(ButtonName::Play),
                ButtonName::FavPlay => return Some(ButtonName::FavPlay),
                ButtonName::FavRecord => return Some(ButtonName::FavRecord),
                ButtonName::Seek => return Some(ButtonName::Seek),
                _ => return None,
            }
        }
    }
    None
}
