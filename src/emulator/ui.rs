extern crate sdl2;
use sdl2::{pixels::Color, render::Canvas, Sdl, video::Window};
use std::time::Duration;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const FREQ: u32 = 60;

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

    pub fn handle_events(&mut self) -> () {
        let mut event_pump = self.context.event_pump().unwrap();
        loop {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit {..} => std::process::exit(0),
                    _ => {}
                }
            }
            self.canvas.present();
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FREQ));
        }
    }
}