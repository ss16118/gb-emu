extern crate sdl2;
use std::sync::{Arc, Mutex};
use sdl2::{pixels::Color, render::Canvas, Sdl, video::Window, 
    rect::Rect, surface::Surface};
use std::time::Duration;
use crate::emulator::address_bus::*;
use crate::emulator::Emulator;

const SCALE: i32 = 4;
const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const FREQ: u32 = 60;

const TILE_COLORS: [Color; 4] = [
    Color::RGB(0xFF, 0xFF, 0xFF),
    Color::RGB(0xAA, 0xAA, 0xAA),
    Color::RGB(0x55, 0x55, 0x55),
    Color::RGB(0, 0, 0)
];

pub struct UI {
    context: Box<Sdl>,
    canvas: Box<Canvas<Window>>,
}

impl UI {
    pub fn new() -> UI {
        let context = sdl2::init().unwrap();
        let video_subsystem = context.video().unwrap();
        let window = video_subsystem.window("GameBoy Emulator", WIDTH, HEIGHT)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();
        UI {
            context: Box::new(context),
            canvas: Box::new(canvas),
        }
    }

    pub fn display_tile(&mut self, start_loc: u16,
            tile_num: u16, x: u32, y: u32) -> () {
        let mut rec = Rect::new(x as i32, y as i32, SCALE as u32, SCALE as u32);
        let canvas = &mut self.canvas;
        for tile_y in (0..16).step_by(2) {
            let b1 = bus_read(start_loc + (tile_num * 16) + tile_y);
            let b2 = bus_read(start_loc + (tile_num * 16) + tile_y + 1);
            for bit in (0..8).rev() {
                let hi = !!(b1 & (1 << bit)) << 1;
                let lo = !!(b2 & (1 << bit));
                let color = hi | lo;

                rec.x = (x as i32 + ((7 - bit) * SCALE)) as i32;
                rec.y = (y as i32 + (tile_y as i32 / 2 * SCALE)) as i32;
                rec.w = SCALE as i32;
                rec.h = SCALE as i32;
                // Draws the rectangle
                canvas.set_draw_color(TILE_COLORS[color as usize]);
                canvas.fill_rect(rec).unwrap();
            }
        }
    }

    fn update(&mut self) -> () {

    }

    /**
     * UI loop, runs until the user closes the window. Handles events,
     * and updates the screen.
     */
    pub fn run(&mut self) -> () {
        let mut event_pump = self.context.event_pump().unwrap();
        loop {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit {..} => std::process::exit(0),
                    _ => {}
                }
            }
            self.update();
            self.canvas.present();
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FREQ));
        }
    }
}