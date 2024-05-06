use crate::render_drawing::{DrawConfig, DrawVisual};
use nannou::prelude::*;

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

pub struct SeekLine {
    pub bbox: BBox,
    pub button: Button,
}

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
    fn scale_visual(&mut self, win: Rect) {
    }
}

struct TimeStamp {
    pub time_elapsed: String,
    pub total_time: String,
}

struct SongInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub genre: String,
}

impl SeekLine {
    pub fn new(w: f32) -> Self {
        let scroll_button = Button::new(ButtonName::Play, BBox(0., 0., 15., 15.), || {});
        SeekLine {
            bbox: BBox(0., 0., w, 10.),
            button: scroll_button,
        }
    }
}

impl DrawVisual for SeekLine {
    fn draw_visual(
        &self,
        draw: &nannou::prelude::Draw,
        win: nannou::prelude::Rect,
        config: &DrawConfig,
    ) {
        let (mut x, mut y, w, h) = (self.bbox.0, self.bbox.1, self.bbox.2, self.bbox.3);
        x += w / 2.;
        y += h / 2.;

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
        Self(self.0 + self.2/2., self.1 + self.3/2., self.2, self.3)
    }

    fn to_tuple(&self) -> (f32, f32, f32, f32) {
        (self.0, self.1, self.2, self.3)
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
