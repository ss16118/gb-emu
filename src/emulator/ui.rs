use sdl2_sys::*;
use sdl2_sys::SDL_TextureAccess::*;
use sdl2_sys::SDL_PixelFormatEnum::*;
use sdl2_sys::SDL_EventType::*;
use sdl2_sys::SDL_KeyCode::*;
use sdl2_sys::SDL_WindowEventID::*;

use crate::emulator::address_bus::*;
use crate::emulator::ppu::*;
use crate::emulator::gamepad::*;

const SCALE: i32 = 4;
const WIDTH: i32 = 1024;
const HEIGHT: i32 = 768;
const FREQ: u32 = 60;

const TILE_COLORS: [u32; 4] = [
    0xFFFFFFFF, // White
    0xFFAAAAAA, // Light gray
    0xFF555555, // Dark gray
    0xFF000000 // Black
];

// Jesus christ rust is a pain when
// it comes to converting enums to ints
const KEY_Z: i32 = SDLK_z as i32;
const KEY_X: i32 = SDLK_x as i32;
const KEY_RETURN: i32 = SDLK_RETURN as i32;
const KEY_TAB: i32 = SDLK_TAB as i32;
const KEY_UP: i32 = SDLK_UP as i32;
const KEY_DOWN: i32 = SDLK_DOWN as i32;
const KEY_LEFT: i32 = SDLK_LEFT as i32;
const KEY_RIGHT: i32 = SDLK_RIGHT as i32;

#[allow(non_upper_case_globals)]
static mut main_window: *mut SDL_Window = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
static mut main_renderer: *mut SDL_Renderer = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
static mut main_texture: *mut SDL_Texture = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
static mut main_screen: *mut SDL_Surface = std::ptr::null_mut();

#[allow(non_upper_case_globals)]
static mut debug_window: *mut SDL_Window = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
static mut debug_renderer: *mut SDL_Renderer = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
static mut debug_texture: *mut SDL_Texture = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
static mut debug_screen: *mut SDL_Surface = std::ptr::null_mut();


/**
 * Initializes the main window and debug window
 */
pub fn init() -> () {
    log::info!("Initializing UI...");
    unsafe {
        SDL_Init(SDL_INIT_VIDEO);
        // Creates the main window
        SDL_CreateWindowAndRenderer(WIDTH, HEIGHT, 0, &mut main_window, &mut main_renderer);
        main_screen = SDL_CreateRGBSurface(0, WIDTH, HEIGHT, 32,
            0x00FF0000, 0x0000FF00, 0x000000FF, 0xFF000000);
        main_texture = SDL_CreateTexture(main_renderer, SDL_PIXELFORMAT_ARGB8888 as u32,
            SDL_TEXTUREACCESS_STREAMING as i32, WIDTH, HEIGHT);

        // Creates the debug window
        SDL_CreateWindowAndRenderer(16 * 8 * SCALE, 32 * 8 * SCALE, 0, 
            &mut debug_window, &mut debug_renderer);
        
        debug_screen = SDL_CreateRGBSurface(0, (16 * 8 * SCALE) + (16 * SCALE),
            (32 * 8 * SCALE) + (64 * SCALE), 32,
            0x00FF0000, 0x0000FF00, 0x000000FF, 0xFF000000);
        debug_texture = SDL_CreateTexture(debug_renderer,
                SDL_PIXELFORMAT_ARGB8888 as u32,
                SDL_TEXTUREACCESS_STREAMING as i32,
                (16 * 8 * SCALE) + (16 * SCALE), 
                (32 * 8 * SCALE) + (64 * SCALE));
        
        let mut x = 0;
        let mut y = 0;
        
        // Sets the location of the debug window
        // relative to the main window
        SDL_GetWindowPosition(main_window, &mut x, &mut y);
        SDL_SetWindowPosition(debug_window, x + WIDTH + 10, y);
    }
    log::info!(target: "stdout", "Initialize UI: SUCCESS");

}


pub fn display_tile(surface: *mut SDL_Surface, start_loc: u16, tile_num: u16, x: i32, y: i32) -> () {
    let mut rect: SDL_Rect = SDL_Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0
    };
    for tile_y in (0..16).step_by(2) {
        let b1 = bus_read(start_loc + (tile_num * 16) + tile_y);
        let b2 = bus_read(start_loc + (tile_num * 16) + tile_y + 1);
        for bit in (0..8).rev() {
            let hi = (((b1 & (1 << bit)) > 0) as i8) << 1;
            let lo = ((b2 & (1 << bit)) > 0) as i8;
            let color = hi | lo;

            rect.x = (x + ((7 - bit) * SCALE)) as i32;
            rect.y = (y + (tile_y as i32 / 2 * SCALE)) as i32;
            rect.w = SCALE as i32;
            rect.h = SCALE as i32;
            // Draws the rectangle
            unsafe {
                SDL_FillRect(surface, &rect, TILE_COLORS[color as usize]);
            }
        }
    }
}

/**
 * A helper function that updates the debug window
 */
fn update_debug_window() -> () {
    // Fills the debug window with the color gray
    let mut rect: SDL_Rect = SDL_Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0
    };
    unsafe {
        rect.w = (*debug_screen).w;
        rect.h = (*debug_screen).h;
        SDL_FillRect(debug_screen, &rect, 0xFF111111);
    }
    // Draws the tiles
    let addr: u16 = 0x8000;
    let mut x_draw = 0;
    let mut y_draw = 0;
    let mut tile_num: u16 = 0;
    // 384 tiles: 24 * 16
    for y in 0..24 {
        for x in 0..16 {
            display_tile(unsafe { debug_screen }, 
                addr, tile_num,
                x_draw + (x & SCALE), y_draw + (y * SCALE));
            x_draw += 8 * SCALE;
            tile_num += 1;
        }
        y_draw += 8 * SCALE;
        x_draw = 0;
    }
    unsafe {
        SDL_UpdateTexture(debug_texture, std::ptr::null(), (*debug_screen).pixels, (*debug_screen).pitch);
        SDL_RenderClear(debug_renderer);
        SDL_RenderCopy(debug_renderer, debug_texture, std::ptr::null(), std::ptr::null());
        SDL_RenderPresent(debug_renderer);
    }
}

/**
 * A helper function that updates the main window
 */
fn update_main_window() -> () {
    let mut rect: SDL_Rect = SDL_Rect {
        x: 0,
        y: 0,
        w: 2048,
        h: 2048
    };

    let video_buffer = unsafe { PPU_CTX.video_buffer.clone() };
    // Loops through each line and each pixel in the line
    for line_num in 0..Y_RES {
        for x in 0..X_RES {
            rect.x = x as i32 * SCALE;
            rect.y = line_num as i32 * SCALE;
            rect.w = SCALE;
            rect.h = SCALE;

            let offset: u32 = (line_num as u32 * X_RES as u32) + x as u32;
            unsafe {
                SDL_FillRect(main_screen, &rect, video_buffer[offset as usize]);
            }
        }
    }
    unsafe {
        SDL_UpdateTexture(main_texture, std::ptr::null(), (*main_screen).pixels, (*main_screen).pitch);
        SDL_RenderClear(main_renderer);
        SDL_RenderCopy(main_renderer, main_texture, std::ptr::null(), std::ptr::null());
        SDL_RenderPresent(main_renderer);
    }
}


/**
 * A helper function that handles key events
 */
fn handle_key_event(down: bool, key_code: i32) -> () {
    match key_code {
        KEY_Z => {
            unsafe { GAMEPAD_CTX.controller.b = down };
        },
        KEY_X => {
            unsafe { GAMEPAD_CTX.controller.a = down };
        },
        KEY_RETURN => {
            unsafe { GAMEPAD_CTX.controller.start = down };
        },
        KEY_TAB => {
            unsafe { GAMEPAD_CTX.controller.select = down };
        },
        KEY_UP => {
            unsafe { GAMEPAD_CTX.controller.up = down };
        },
        KEY_DOWN => {
            unsafe { GAMEPAD_CTX.controller.down = down };
        },
        KEY_LEFT => {
            unsafe { GAMEPAD_CTX.controller.left = down };
        },
        KEY_RIGHT => {
            unsafe { GAMEPAD_CTX.controller.right = down };
        },
        _ => {
            log::warn!("Unsupported key code: {}", key_code);
        }
    }
}


/**
 * UI loop, runs until the user closes the window. Handles events,
 * and updates the screen.
 */
pub fn run() -> () {
    let mut prev_frame: u64 = 0;
    let mut event: SDL_Event = SDL_Event {
        type_: 0,
    };
    
    loop {
        // Event handling
        unsafe {
            while SDL_PollEvent(&mut event) > 0 {
                if event.type_ == SDL_KEYDOWN as u32 {
                    // Down arrow
                    handle_key_event(true, event.key.keysym.sym);
                } else if event.type_ == SDL_KEYUP as u32 {
                    // Up arrow
                    handle_key_event(false, event.key.keysym.sym);
                } else if (event.type_ == SDL_WINDOWEVENT as u32) && 
                   (event.window.event == SDL_WINDOWEVENT_CLOSE as u8) {
                    std::process::exit(0);
                }
            }
        }
        if prev_frame != unsafe { PPU_CTX.curr_frame } {
            update_debug_window();
            update_main_window();
        }
        prev_frame = unsafe { PPU_CTX.curr_frame };
        // main.canvas.present();
        // debug_window.canvas.present();
        // std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FREQ));
    }
}


/**
 * Returns the number of milliseconds since the SDL
 * library was initialized. A wrapper for SDL_GetTicks64.
 */
pub fn get_ticks() -> u64 {
    return unsafe { sdl2_sys::SDL_GetTicks64() };
}

/**
 * Delays the program for the given number of milliseconds.
 * A wrapper for SDL_Delay.
 */
pub fn delay(ms: u32) -> () {
    unsafe { sdl2_sys::SDL_Delay(ms) };
}