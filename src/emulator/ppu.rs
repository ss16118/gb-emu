use std::collections::LinkedList;
use std::sync::Arc;

use once_cell::sync::Lazy;
use crate::emulator::cpu::interrupts::*;
use crate::emulator::ui;
use crate::emulator::address_bus::*;
use crate::emulator::cartridge::CARTRIDGE_CTX;
use super::{lcd::*, cpu::interrupts::request_interrupt};

pub mod fifo;
use fifo::*;

// Bit masks for accessing the OAM flags
const PRIORITY_MASK: u8     = 0x80;
const Y_FLIP_MASK: u8       = 0x40;
const X_FLIP_MASK: u8       = 0x20;
const DMG_PALETTE_MASK: u8  = 0x10;
const BANK_MASK: u8         = 0x08;
const CGB_PALETTE_MASK: u8  = 0x07;

const LINES_PER_FRAME: u32  = 154;
const TICKS_PER_LINE: u32   = 456;
pub const Y_RES: u8             = 144;
pub const X_RES: u8             = 160;

const TARGET_FRAME_TIME: u64 = 1000 / 60;
static mut prev_frame_time: u64 = 0;
static mut start_timer: u64 = 0;
static mut frame_counter: u32 = 0;


// A struct representing a single Object Attribute Memory
// (OAM) entry
#[repr(C)]
#[derive(Copy, Clone)]
pub struct OamEntry {
    pub y: u8,
    pub x: u8,
    tile: u8,
    flags: u8,
}

/**
 * OAM Attributes / Flags:
 * Priority: 0-1 (0=Normal, 1=Priority)
 * Y flip: 0-1 (0=Normal, 1=Mirror vertically)
 * X flip: 0-1 (0=Normal, 1=Mirror horizontally)
 * Palette: 0-1 (0=OBP0, 1=OBP1)
 * Bank: 0-1 [CGB only] (0=Bank 0, 1=Bank 1)
 * CGB palette: 0-3 [CGB only]
 */
impl OamEntry {
    pub fn new () -> OamEntry {
        OamEntry {
            x: 0,
            y: 0,
            tile: 0,
            flags: 0,
        }
    }

    /**
     * Returns the value of the flag given its mask.
     */
    pub fn get_flag(&self, mask: u8) -> u8 {
        return self.flags & mask;
    }

    /**
     * Sets the value of the flag given its mask.
     */
    pub fn set_flag(&mut self, mask: u8, value: u8) -> () {
        self.flags = (self.flags & !mask) | (value & mask);
    }
}


pub struct PPU {
    pub curr_frame: u64,
    line_ticks: u32,
    pixel_fifo: PixelFifo,
    // A list of sprites on the current line
    line_sprites: Vec<*mut OamEntry>,
    fetched_entry_count: u8,
    // Entries fetched during pipeline
    fetched_entries: [*mut OamEntry; 3],
    window_line: u8,

    pub video_buffer: Box<[u32; (X_RES as u32 * Y_RES as u32) as usize]>,
    pub oam_ram: [OamEntry; 40],
    vram: [u8; 0x2000],
}


pub static mut PPU_CTX: Lazy<PPU> = Lazy::new(|| PPU {
    curr_frame: 0,
    line_ticks: 0,
    pixel_fifo: PixelFifo::new(),
    line_sprites: Vec::new(),
    fetched_entry_count: 0,
    fetched_entries: [std::ptr::null_mut(); 3],
    window_line: 0,
    video_buffer: Box::new([0; (X_RES as u32 * Y_RES as u32) as usize]),
    oam_ram: [OamEntry::new(); 40],
    vram: [0; 0x2000],
});


impl PPU {
    /**
     * Writes a byte to the OAM RAM
     */
    pub fn oam_write(&mut self, mut address: u16, value: u8) -> () {
        if address >= 0xFE00 {
            address -= 0xFE00;
        }
        // Converts OAM RAM into a byte array
        let oam_bytes = unsafe {
            std::slice::from_raw_parts_mut(
                self.oam_ram.as_mut_ptr() as *mut u8,
                std::mem::size_of::<OamEntry>() * self.oam_ram.len()
            )
        };
        if (address as usize) < oam_bytes.len() {
            oam_bytes[address as usize] = value;
        } else {
            log::error!("Invalid write to OAM address {:04X}", address);
            std::process::exit(-1);
        }
    }

    /**
     * Reads a byte from the OAM RAM
     */
    pub fn oam_read(&self, mut address: u16) -> u8 {
        if address >= 0xFE00 {
            address -= 0xFE00;
        }
        // Converts OAM RAM into a byte array
        let oam_bytes = unsafe {
            std::slice::from_raw_parts(
                self.oam_ram.as_ptr() as *const u8,
                std::mem::size_of::<OamEntry>() * self.oam_ram.len()
            )
        };
        if (address as usize) < oam_bytes.len() {
            return oam_bytes[address as usize];
        } else {
            log::error!("Invalid read from OAM address {:04X}", address);
            std::process::exit(-1);
        }
    }

    /**
     * Writes a byte to the VRAM
     */
    pub fn vram_write(&mut self, mut address: u16, value: u8) -> () {
        address -= 0x8000;
        if (address as usize) < self.vram.len() {
            self.vram[address as usize] = value;
        } else {
            log::error!("Invalid write to VRAM address {:04X}", address);
            std::process::exit(-1);
        }
    }

    /**
     * Reads a byte from the VRAM
     */
    pub fn vram_read(&self, mut address: u16) -> u8 {
        address -= 0x8000;
        if (address as usize) < self.vram.len() {
            return self.vram[address as usize];
        } else {
            log::error!("Invalid read from VRAM address {:04X}", address);
            std::process::exit(-1);
        }
    }
    /**********************************************************
     * Functions that implement different PPU modes / states
     **********************************************************/

    /**
     * A helper function that fetches the color of a pixel
     */
    fn fetch_sprite_pixels(&mut self, mut bit: i32, mut color: u32, bg_color: u8) -> u32 {
        // Iterates through all the fetched entries
        for i in 0..(self.fetched_entry_count) {
            let fetched_entry = unsafe { *self.fetched_entries[i as usize] };
            let sp_x: i32 = ((fetched_entry.x as i32).wrapping_sub(8)).wrapping_add(unsafe { LCD_CTX.scroll_x % 8 } as i32);
            
            if sp_x.wrapping_add(8) < self.pixel_fifo.fifo_x as i32 {
                // If we have past the sprite, continue
                continue;
            }

            let offset: i32 = (self.pixel_fifo.fifo_x as i32).wrapping_sub(sp_x as i32);

            if offset < 0 || offset > 7 {
                // Out of bounds
                continue;
            }

            bit = (7 - offset) as i32;
            if fetched_entry.get_flag(X_FLIP_MASK) != 0 {
                bit = offset;
            }
            let hi = 
                ((self.pixel_fifo.fetch_entry_data[(i as i32 * 2) as usize] & (1 << bit)) != 0) as u8;
            let lo =
                (((self.pixel_fifo.fetch_entry_data[((i as i32 * 2) + 1) as usize] & (1 << bit)) != 0) as u8) << 1;

            let bg_priority = fetched_entry.get_flag(PRIORITY_MASK) != 0;

            let val = hi | lo;
            // println!("[DEBUG] ly: {}, i: {}, val: {}, bg_priority: {}, bg_color: {}", ly, i, val, bg_priority as u8, bg_color);
            if val == 0 {
                // Transparent pixel
                continue;
            }

            if !bg_priority || bg_color == 0 {
                let palette = fetched_entry.get_flag(DMG_PALETTE_MASK) != 0;
                // println!("[DEBUG] ly: {}, palette: {}", unsafe { LCD_CTX.ly }, palette as u8);
                if palette {
                    color = unsafe { LCD_CTX.sp2_colors[val as usize] };
                } else {
                    color = unsafe { LCD_CTX.sp1_colors[val as usize] };
                }

                if val != 0 {
                    break;
                }
            }
        }

        return color;
    }

    /**
     * A helper function that tries to add a new entry to the FIFO
     * and returns true if the entry was added successfully.
     * Otherwise, i.e., if the FIFO is full, it returns false.
     */
    fn pipeline_fifo_add(&mut self) -> bool {
        if self.pixel_fifo.get_size() > 8 {
            // The FIFO is full
            return false;
        }
        // The FIFO is not full
        // Adds a new entry to the FIFO
        let x: i32 = self.pixel_fifo.fetch_x as i32 -
            (8 - (unsafe { LCD_CTX.scroll_x } % 8)) as i32;
        
        for i in 0..8 {
            let bit: i32 = (7 - i) as i32;
            let hi = (self.pixel_fifo.bgw_fetch_data[1] & (1 << bit) != 0) as u8;
            let lo = ((self.pixel_fifo.bgw_fetch_data[2] & (1 << bit) != 0) as u8) << 1;
            let mut color = unsafe { LCD_CTX.bg_colors[(hi | lo) as usize] };

            // Checks if the background window display is enabled
            if unsafe { !LCD_CTX.get_lcdc_flag(BGW_ENABLE_MASK) } {
                color = unsafe { LCD_CTX.bg_colors[0] };
            }

            // Checks if sprites are enabled
            if unsafe { LCD_CTX.get_lcdc_flag(OBJ_ENABLE_MASK) } {
                color = self.fetch_sprite_pixels(bit, color, hi | lo);
            }
            // println!("[DEBUG] ly: {}, color: {:08X}", unsafe { LCD_CTX.ly }, color);
            if x >= 0 {
                self.pixel_fifo.push(color);
                self.pixel_fifo.fifo_x = self.pixel_fifo.fifo_x.wrapping_add(1);
            }
        }

        return true;
    }


    /**
     * A helper function that loads a sprite tile from memory
     */
    fn pipeline_load_sprite_tile(&mut self) -> () {
        // Iterates through line sprites
        for i in 0..self.line_sprites.len() {
            let entry = self.line_sprites[i];
                let sp_x: i32 = unsafe { (((*entry).x as i32).wrapping_sub(8))
                    .wrapping_add((LCD_CTX.scroll_x % 8) as i32) };
            
            let fetch_x = self.pixel_fifo.fetch_x as i32;
            // The sprite is within the current fetch range
            if (sp_x >= fetch_x && sp_x < fetch_x.wrapping_add(8)) ||
                ((sp_x.wrapping_add(8)) >= fetch_x && 
                (sp_x.wrapping_add(8)) < fetch_x.wrapping_add(8)) {
                self.fetched_entries[self.fetched_entry_count as usize] = entry;
                self.fetched_entry_count = self.fetched_entry_count.wrapping_add(1);
            }

            if self.fetched_entry_count >= 3 {
                // Max 3 sprites per line
                break;
            }
        }
    }


    /**
     * A helper function that loads sprite data from memory
     */
    fn pipeline_load_sprite_data(&mut self, offset: u8) -> () {
        let curr_y: i32 = unsafe { LCD_CTX.ly as i32 };
        let sprite_height = unsafe { LCD_CTX.get_lcdc_obj_size() };
        for i in 0..self.fetched_entry_count {
            let entry = self.fetched_entries[i as usize];
            let mut tile_y: u8 = (curr_y.wrapping_add(16) as i32)
                .wrapping_sub(unsafe { (*entry).y }as i32).wrapping_mul(2) as u8;
            if unsafe { (*entry).get_flag(Y_FLIP_MASK) } != 0 {
                // If flipped upside down
                tile_y =((sprite_height as i32 * 2) - 2).wrapping_sub(tile_y as i32) as u8;
            }
            let mut tile_index: u8 = unsafe { (*entry).tile };
            if sprite_height == 16 {
                // Removes the last bit
                tile_index &= !1;
            }
            let addr = (0x8000 + (tile_index as u16 * 16) as u32 + tile_y as u32) + offset as u32;
            let index = ((i as i32) * 2 + offset as i32) as usize;
            self.pixel_fifo.fetch_entry_data[index] =  bus_read(addr as u16);
        }
    }

    /**
     * A helper function that loads a window tile from memory
     */
    fn pipeline_load_window_tile(&mut self) -> () {
        if !self.window_visible() {
            return;
        }

        let win_y: u16 = unsafe { LCD_CTX.win_y } as u16;
        let win_x: u16 = unsafe { LCD_CTX.win_x } as u16;
        let fetch_x: u16 =  self.pixel_fifo.fetch_x as u16;
        let ly =  unsafe { LCD_CTX.ly } as u16;
        let map_area = unsafe { LCD_CTX.get_lcdc_win_tile_map_area() };
        if fetch_x.wrapping_add(7) >= win_x &&
            fetch_x.wrapping_add(7) < win_x.wrapping_add(Y_RES as u16).wrapping_add(14) {
            if ly >= win_y && ly < win_y.wrapping_add(X_RES as u16) {
                let w_tile_y = self.window_line / 8;
                let addr = map_area + (((fetch_x + 7 - win_x) as u16) / 8) +
                    (w_tile_y as u16 * 32);
                let data = bus_read(addr);
                self.pixel_fifo.bgw_fetch_data[0] = data;

                if unsafe { LCD_CTX.get_lcdc_bg_tile_data_area() } == 0x8800 {
                    self.pixel_fifo.bgw_fetch_data[0] =
                        self.pixel_fifo.bgw_fetch_data[0].wrapping_add(128);
                }
            }
        }

    }
    

    /**
     * A helper function that fetches a tile based on the
     * current fetch state
     */
    fn pipeline_fetch(&mut self) -> () {
        match self.pixel_fifo.curr_state {
            FetchState::FS_TILE => {
                self.fetched_entry_count = 0;
                // Checks if the background window display is enabled
                if unsafe { LCD_CTX.get_lcdc_flag(BGW_ENABLE_MASK) } {
                    let map_area = unsafe { LCD_CTX.get_lcdc_bg_tile_map_area() };
                    let addr: u32 = map_area as u32 + 
                        (self.pixel_fifo.map_x as u32 / 8) + 
                        ((self.pixel_fifo.map_y as u32 / 8) * 32);
                    let data = bus_read(addr as u16);
                    self.pixel_fifo.bgw_fetch_data[0] = data;
                    if unsafe { LCD_CTX.get_lcdc_bg_tile_data_area() } == 0x8800 {
                        self.pixel_fifo.bgw_fetch_data[0] = 
                            self.pixel_fifo.bgw_fetch_data[0].wrapping_add(128);
                    }
                    // println!("[DEBUG] ly: {}, addr: {:04X}, data: {}", unsafe { LCD_CTX.ly }, addr as u16, self.pixel_fifo.bgw_fetch_data[0]);
                    self.pipeline_load_window_tile();
                }
                // If sprites are enabled and there are sprites on the current line
                if unsafe { LCD_CTX.get_lcdc_flag(OBJ_ENABLE_MASK) } && 
                    self.line_sprites.len() > 0 {
                    self.pipeline_load_sprite_tile();
                }

                // Sets the next fetch state
                self.pixel_fifo.curr_state = FetchState::FS_TILE_DATA_LOW;
                self.pixel_fifo.fetch_x = self.pixel_fifo.fetch_x.wrapping_add(8);
            },
            FetchState::FS_TILE_DATA_LOW => {
                let data_area = unsafe { LCD_CTX.get_lcdc_bg_tile_data_area() };
                let addr: u32 = data_area as u32 +
                    (self.pixel_fifo.bgw_fetch_data[0] as u32 * 16) +
                    (self.pixel_fifo.tile_y as u32);
                let data = bus_read(addr as u16);
                self.pixel_fifo.bgw_fetch_data[1] = data;

                self.pipeline_load_sprite_data(0);

                // Sets the next fetch state
                self.pixel_fifo.curr_state = FetchState::FS_TILE_DATA_HIGH;
            },
            FetchState::FS_TILE_DATA_HIGH => {
                let data_area = unsafe { LCD_CTX.get_lcdc_bg_tile_data_area() };
                let addr = data_area as u32 +
                    (self.pixel_fifo.bgw_fetch_data[0] as u32 * 16) +
                    (self.pixel_fifo.tile_y as u32 + 1);
                let data = bus_read(addr as u16);
                self.pixel_fifo.bgw_fetch_data[2] = data;
                self.pipeline_load_sprite_data(1);

                // Sets the next fetch state
                self.pixel_fifo.curr_state = FetchState::FS_IDLE;
            },
            FetchState::FS_IDLE => {
                // Sets the next fetch state
                self.pixel_fifo.curr_state = FetchState::FS_PUSH;
            }
            FetchState::FS_PUSH => {
                // Stays in the same state until the add operation succeeds
                if self.pipeline_fifo_add() {
                    // Sets the next fetch state
                    self.pixel_fifo.curr_state = FetchState::FS_TILE;
                }
            }
        }
    }


    /**
     * A helper function that pushes a pixel to the pipeline
     */
    fn pipeline_push_pixel(&mut self) -> () {
        if self.pixel_fifo.get_size() > 8 {
            // The FIFO is full
            let data = self.pixel_fifo.pop();
            if self.pixel_fifo.line_x >= unsafe { LCD_CTX.scroll_x % 8 } {
                // Pushes the pixel to the video buffer
                let offset: u32 = self.pixel_fifo.pushed_x as u32 + unsafe { LCD_CTX.ly as u32 * X_RES as u32};
                self.video_buffer[offset as usize] = data;
                self.pixel_fifo.pushed_x = self.pixel_fifo.pushed_x.wrapping_add(1);
            }

            self.pixel_fifo.line_x = self.pixel_fifo.line_x.wrapping_add(1);
        }
    }

    /**
     * A helper function that executes all procedures in the
     * pixel processing pipeline
     */
    fn pipeline_process(&mut self) -> () {
        unsafe {
            self.pixel_fifo.map_y = LCD_CTX.ly.wrapping_add(LCD_CTX.scroll_y);
            self.pixel_fifo.map_x = self.pixel_fifo.fetch_x.wrapping_add(LCD_CTX.scroll_x);
            self.pixel_fifo.tile_y = (LCD_CTX.ly.wrapping_add(LCD_CTX.scroll_y) % 8) * 2;
        }
        // If the line is an even number
        if self.line_ticks & 1 == 0 {
            self.pipeline_fetch();
        }
        self.pipeline_push_pixel();
    }


    /**
     * A helper function that checks if the window is visible
     */
    #[inline(always)]
    #[allow(unused_comparisons)]
    fn window_visible(&self) -> bool {
        let win_x = unsafe { LCD_CTX.win_x };
        let win_y = unsafe { LCD_CTX.win_y };
        return unsafe { LCD_CTX.get_lcdc_flag(WIN_ENABLE_MASK) } &&
            win_x >= 0 && win_x <= 166 &&
            win_y >= 0 && win_y < Y_RES;
    }

    /**
     * A helper function that increments the LY register
     * and checks if the LY register matches the LYC register
     * and sets the LYC flag accordingly.
     */
    fn increment_ly(&mut self) -> () {
        unsafe {
            let ly = LCD_CTX.ly;
            if self.window_visible() && ly >= LCD_CTX.win_y && 
                (ly as u16) < ((LCD_CTX.win_y as u16).wrapping_add(Y_RES as u16)) {
                // If we are on the window
                self.window_line = self.window_line.wrapping_add(1);

            }
            LCD_CTX.ly = LCD_CTX.ly.wrapping_add(1);
            if LCD_CTX.ly == LCD_CTX.lyc {
                LCD_CTX.set_lcds_lyc(true);
                if LCD_CTX.get_lcds_flag(LYC_INT_MASK) {
                    request_interrupt(InterruptType::IT_LCD_STAT);
                }
            } else {
                LCD_CTX.set_lcds_lyc(false);
            }
        }
    }



    /**
     * Performs operations under the HBlank mode
     */
    fn mode_hblank(&mut self) -> () {
        if self.line_ticks >= TICKS_PER_LINE {
            self.increment_ly();
            if unsafe { LCD_CTX.ly } >= Y_RES {
                // We are at the end of the line
                // Resets the mode to VBlank
                unsafe { LCD_CTX.set_lcds_mode(LCD_MODE::MODE_VBLANK); }
                request_interrupt(InterruptType::IT_VBLANK);
                // If the VBlank interrupt is enabled, request an interrupt
                if unsafe { LCD_CTX.get_lcds_flag(VBLANK_INT_MASK) } {
                    request_interrupt(InterruptType::IT_LCD_STAT);
                }

                // Increments the frame counter
                self.curr_frame = self.curr_frame.wrapping_add(1);

                // Aims to match the current frame rate
                // with the target frame rate
                let curr_time: u64 = ui::get_ticks();
                let frame_delay = curr_time - unsafe { prev_frame_time };
                if frame_delay < TARGET_FRAME_TIME {
                    ui::delay((TARGET_FRAME_TIME - frame_delay) as u32);
                }

                // Computes the FPS
                if curr_time - unsafe { start_timer } >= 1000 {
                    // log::info!(target: "stdout", "FPS: {}", unsafe { frame_counter });
                    println!("FPS: {}", unsafe { frame_counter });
                    unsafe { 
                        frame_counter = 0;
                        start_timer = curr_time;
                    }
                    unsafe {
                        if CARTRIDGE_CTX.need_save() {
                            CARTRIDGE_CTX.save_battery();
                        }
                    }
                }
                unsafe {
                    frame_counter = frame_counter.checked_add(1).unwrap();
                    prev_frame_time = ui::get_ticks();
                }

            } else {
                // We are still in the middle of the line
                // Resets the mode to OAM
                unsafe { LCD_CTX.set_lcds_mode(LCD_MODE::MODE_OAM); }
            }
            self.line_ticks = 0;
        }

    }

    /**
     * Performs operations under the VBlank mode
     */
    fn mode_vblank(&mut self) -> () {
        // VBlank mode lasts for 456 ticks
        // After 456 ticks, the PPU switches to the OAM mode
        if self.line_ticks >= TICKS_PER_LINE {
            self.increment_ly();
            
            if unsafe { LCD_CTX.ly as u32} >= LINES_PER_FRAME {
                unsafe { 
                    LCD_CTX.set_lcds_mode(LCD_MODE::MODE_OAM); 
                    LCD_CTX.ly = 0;
                    self.window_line = 0;
                }
            }

            self.line_ticks = 0;
        }
    }

    /**
     * A helper function that loads sprites on the current line
     */
    fn load_line_sprites(&mut self) -> () {
        let curr_y: i32 = unsafe { LCD_CTX.ly as i32 };
        let sprite_height = unsafe { LCD_CTX.get_lcdc_obj_size() };
        for i in 0..self.oam_ram.len() {
            let entry: *mut OamEntry = &mut self.oam_ram[i];
            if unsafe { (*entry).x == 0 } {
                // If the sprite is not visible
                continue;
            }

            if self.line_sprites.len() >= 10 {
                // Max 10 sprites per line
                break;
            }
            if unsafe { (*entry).y as i32 <= curr_y.wrapping_add(16) } &&
               unsafe { ((*entry).y as i32).wrapping_add(sprite_height as i32) > 
                    curr_y.wrapping_add(16) } {
                // println!("[DEBUG] Added entry x: {}", unsafe { (*entry).x });
                // the sprite is on the current line
                // Adds the sprite to the list of sprites on the current line
                self.line_sprites.push(entry);
            }
        }
        // Sorts the sprites by their x coordinate
        self.line_sprites.sort_by(|a, b| {
            let a_x = (unsafe { *(*a) }).x;
            let b_x = (unsafe { *(*b) }).x;
            a_x.cmp(&b_x)
        });
    }

    /**
     * Performs operations under the OAM mode
     */
    fn mode_oam(&mut self) -> () {
        // OAM mode lasts for 80 ticks
        // After 80 ticks, the PPU switches to the XFER mode
        if self.line_ticks >= 80 {
            unsafe { LCD_CTX.set_lcds_mode(LCD_MODE::MODE_XFER); }
            self.pixel_fifo.reset();
        }

        if self.line_ticks == 1 {
            // Reads OAM on the first tick
            // https://www.youtube.com/watch?v=MLzcci5HL0Y&list=PLVxiWMqQvhg_yk4qy2cSC3457wZJga_e5&index=14
            self.line_sprites.clear();
            self.load_line_sprites();
        }
    }

    /**
     * Performs operations under the XFER mode
     */
    fn mode_xfer(&mut self) -> () {
        self.pipeline_process();
        // XFER mode lasts for 172 ticks
        // After 172 ticks, the PPU switches to the HBlank mode
        if self.pixel_fifo.pushed_x >= X_RES {
            self.pixel_fifo.clear();

            unsafe { LCD_CTX.set_lcds_mode(LCD_MODE::MODE_HBLANK); }
            // Checks if the HBlank interrupt is enabled
            if unsafe { LCD_CTX.get_lcds_flag(HBLANK_INT_MASK) } {
                request_interrupt(InterruptType::IT_LCD_STAT);
            }
        }
    }


    /**
     * Performs a single PPU tick
     */
    pub fn tick(&mut self) -> () {
        self.line_ticks = self.line_ticks.wrapping_add(1);
        
        // During a frame, the Game Boy’s PPU cycles between four modes
        let mode = unsafe { LCD_CTX.get_lcds_mode() };
        match mode {
            LCD_MODE::MODE_HBLANK => {
                self.mode_hblank();
            },
            LCD_MODE::MODE_VBLANK => {
                self.mode_vblank();
            },
            LCD_MODE::MODE_OAM => {
                self.mode_oam();
            },
            LCD_MODE::MODE_XFER => {
                self.mode_xfer();
            }
        }
    }
}