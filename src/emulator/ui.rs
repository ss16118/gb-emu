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
    Color::RGB(0xFF, 0xFF, 0xFF), // White
    Color::RGB(0xAA, 0xAA, 0xAA), // Light gray
    Color::RGB(0x55, 0x55, 0x55), // Dark gray
    Color::RGB(0, 0, 0) // Black
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

    /**
     * Creates a new debug window that displays the game tiles
     * that will be used in the actual game window.
     * @param main The main UI window that will be used to run the game
     */
    pub fn create_debug_window(main: &UI) -> UI {
        let context = sdl2::init().unwrap();
        let video_subsystem = context.video().unwrap();
        let (main_x, main_y) = main.canvas.window().position();
        // Creates the debug window to the right of the main window
        let window = 
            video_subsystem.window("Debug Window", 16 * 8 * SCALE as u32, 32 * 8 * SCALE as u32)
            .position(main_x + 100 as i32, main_y)
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        // let texture_creator = canvas.texture_creator();
        // let mut texture = texture_creator.create_texture_streaming(
        //     sdl2::pixels::PixelFormatEnum::ARGB8888,
        //     (16 * 8 * SCALE + 16 * SCALE) as u32,
        //     (32 * 8 * SCALE + 64 * SCALE) as u32
        // ).unwrap();
    
        canvas.clear();
        canvas.present();

        UI {
            context: Box::new(context),
            canvas: Box::new(canvas),
        }
    }


    pub fn display_tile(&mut self, start_loc: u16, tile_num: u16, x: i32, y: i32) -> () {
        let mut rec = Rect::new(x as i32, y as i32, SCALE as u32, SCALE as u32);
        let canvas = &mut self.canvas;
        for tile_y in (0..16).step_by(2) {
            let b1 = bus_read(start_loc + (tile_num * 16) + tile_y);
            let b2 = bus_read(start_loc + (tile_num * 16) + tile_y + 1);
            for bit in (0..8).rev() {
                let hi = (((b1 & (1 << bit)) > 0) as i8) << 1;
                let lo = ((b2 & (1 << bit)) > 0) as i8;
                let color = hi | lo;

                rec.x = (x + ((7 - bit) * SCALE)) as i32;
                rec.y = (y + (tile_y as i32 / 2 * SCALE)) as i32;
                rec.w = SCALE as i32;
                rec.h = SCALE as i32;
                // Draws the rectangle
                canvas.set_draw_color(TILE_COLORS[color as usize]);
                canvas.fill_rect(rec).unwrap();
            }
        }
    }

    fn update_debug_window(debug_window: &mut UI) -> () {
        // Fills the debug window with the color gray
        let rect = Rect::new(0, 0, 16 * 8 * SCALE as u32, 32 * 8 * SCALE as u32);
        debug_window.canvas.set_draw_color(Color::RGB(0x11, 0x11, 0x11));
        debug_window.canvas.fill_rect(rect).unwrap();
        // Draws the tiles
        let addr: u16 = 0x8000;
        let mut x_draw = 0;
        let mut y_draw = 0;
        let mut tile_num: u16 = 0;
        // 384 tiles: 24 * 16
        for y in 0..24 {
            for x in 0..16 {
                debug_window.display_tile(addr, tile_num,
                    x_draw + (x & SCALE), y_draw + (y * SCALE));
                x_draw += 8 * SCALE;
                tile_num += 1;
            }
            y_draw += 8 * SCALE;
            x_draw = 0;
        }
    }
    /**
     * UI loop, runs until the user closes the window. Handles events,
     * and updates the screen.
     */
    pub fn run(main: &mut UI, debug_window: &mut UI) -> () {
        let mut main_event_pump = main.context.event_pump().unwrap();
        'running: loop {
            for event in main_event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit {..} => std::process::exit(0),
                    sdl2::event::Event::Window { 
                        win_event: sdl2::event::WindowEvent::Close, .. 
                    } => std::process::exit(0),
                    _ => {}
                }
            }
            // UI::update(main, debug_window);
            // Updates UI
            UI::update_debug_window(debug_window);
            main.canvas.present();
            debug_window.canvas.present();
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FREQ));
        }
    }
}