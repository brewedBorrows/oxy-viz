use crate::render_drawing::{DrawConfig, DrawVisual};
use nannou::prelude::*;

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

pub enum ButtonType {
    Play,
}

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
        match self.buttonType {
            ButtonType::Play => {
                draw.rect()
                    .x_y(0 as f32, 0 as f32)
                    .w_h(win.w() * 0.05, win.h() * 0.05)
                    .color(RED);
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
    for button in buttons {
        let bbox = &button.bbox;
        let (button_x, button_y, button_w, button_h) = (bbox.0, bbox.1, bbox.2, bbox.3);
        if x > button_x && x < button_x + button_w && y > button_y && y < button_y + button_h {
            match button.buttonType {
                ButtonType::Play => return Some(ButtonType::Play),
                _ => return None,
            }
        } else {
            return None;
        }
    }
    None
}
