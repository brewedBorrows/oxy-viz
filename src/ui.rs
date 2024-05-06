use crate::render_drawing::{DrawConfig, DrawVisual};
use nannou::prelude::*;

#[derive(Debug)]
pub struct Button {
    pub buttonType: ButtonType,
    pub bbox: BBox,
    pub onclick: fn(),
}

impl Button {
    pub fn new(buttonType: ButtonType, bbox: BBox, onclick: fn()) -> Self {
        Button {
            buttonType,
            bbox,
            onclick,
        }
    }
}
#[derive(Debug,Clone,Copy)]
pub enum ButtonType {
    Play,
    FavRecord,
    FavPlay,
}

#[derive(Debug)]
pub struct BBox(f32, f32, f32, f32);

impl BBox {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        BBox(x, y, w, h)
    }
}

impl DrawVisual for Button {
    fn draw_visual(
        &self,
        draw: &nannou::prelude::Draw,
        win: nannou::prelude::Rect,
        config: &DrawConfig,
    ) {
        // println!(" > x: > y: > x+w: > y+h: {:?},{:?},{:?},{:?}",self.bbox.0,self.bbox.1,self.bbox.2,self.bbox.3);
        let (mut x, mut y,w,h) = (self.bbox.0, self.bbox.1, self.bbox.2, self.bbox.3);
        x += w/2.;
        y += h/2.;

        match self.buttonType {
            ButtonType::Play => {
                draw.rect()
                .x_y(x as f32, y as f32)
                .w_h(w, w)
                    .color(RED);
            }
            ButtonType::FavPlay =>{
                draw.rect()
                    .x_y(x as f32, y as f32)
                    .w_h(w, w)
                    .color(RED);
            }
            ButtonType::FavRecord =>{
                draw.rect()
                .x_y(x as f32, y as f32)
                .w_h(w, w)
                    .color(BLUE);
            }    
            _ =>{
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
    buttons: &Vec<Button>,
) {
    for button in buttons {
        button.draw_visual(&draw, win, &config);
    }
}

pub fn check_button_click(x: f32, y: f32, buttons: &Vec<Button>) -> Option<ButtonType> {
    println!("yeh le, {:?}",buttons.len());
    for button in buttons {
        dbg!("ek ek btn: {:?}", button);
    }
    for button in buttons {
        let bbox = &button.bbox;
        let (button_x, button_y, button_w, button_h) = (bbox.0, bbox.1, bbox.2, bbox.3);
        if x > button_x && x < button_x + button_w && y > button_y && y < button_y + button_h {
            println!("----matched! , {:?}", button);
            match button.buttonType {
                ButtonType::Play => return Some(ButtonType::Play),
                ButtonType::FavPlay => return Some(ButtonType::FavPlay),
                ButtonType::FavRecord => return Some(ButtonType::FavRecord),
                _=>return None,
            }
        }
    }
    None
}
