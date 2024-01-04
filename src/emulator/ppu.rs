use once_cell::sync::Lazy;
use crate::emulator::cpu::interrupts::*;
use crate::emulator::ui::UI;
use crate::emulator::address_bus::*;
use super::{lcd::*, cpu::interrupts::request_interrupt};

pub mod fifo;
use fifo::*;

// Bit masks for accessing the OAM flags
const PRIORIT_MASK: u8      = 0x80;
const Y_FLIP_MASK: u8       = 0x40;
const X_FLIP_MAKS: u8       = 0x20;
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
struct OamEntry {
    x: u8,
    y: u8,
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
    pub video_buffer: Box<[u32; (X_RES as u32 * Y_RES as u32) as usize]>,
    oam_ram: [OamEntry; 40],
    vram: [u8; 0x2000],
}


pub static mut PPU_CTX: Lazy<PPU> = Lazy::new(|| PPU {
    curr_frame: 0,
    line_ticks: 0,
    pixel_fifo: PixelFifo::new(),
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
            (8 - unsafe { LCD_CTX.scroll_x } % 8) as i32;
        
        for i in 0..8 {
            let bit = (7 - i) as u8;
            let hi = ((self.pixel_fifo.bgw_fetch_data[1] & (1 << bit) != 0) as u8) << 1;
            let lo = (self.pixel_fifo.bgw_fetch_data[2] & (1 << bit) != 0) as u8;
            let color = unsafe { LCD_CTX.bg_colors[(hi | lo) as usize] };

            if x >= 0 {
                self.pixel_fifo.push(color);
                self.pixel_fifo.fifo_x = self.pixel_fifo.fifo_x.wrapping_add(1);
            }
        }

        return true;
    }

    /**
     * A helper function that fetches a tile based on the
     * current fetch state
     */
    fn pipeline_fetch(&mut self) -> () {
        match self.pixel_fifo.curr_state {
            FetchState::FS_TILE => {
                // Checks if the background window display is enabled
                if unsafe { LCD_CTX.get_lcdc_flag(BGW_ENABLE_MASK) } {
                    let map_area = unsafe { LCD_CTX.get_lcdc_bg_tile_map_area() };
                    let addr = map_area + 
                        (self.pixel_fifo.map_x / 8) as u16 + 
                        ((self.pixel_fifo.map_y / 8) as u16 * 32);
                    let data = bus_read(addr);
                    self.pixel_fifo.bgw_fetch_data[0] = data;
                    if unsafe { LCD_CTX.get_lcdc_bg_tile_data_area() } == 0x8800 {
                        self.pixel_fifo.bgw_fetch_data[0] = 
                            self.pixel_fifo.bgw_fetch_data[0].wrapping_add(128);
                    }
                }
                // Sets the next fetch state
                self.pixel_fifo.curr_state = FetchState::FS_TILE_DATA_LOW;
                self.pixel_fifo.fetch_x += 8;
            },
            FetchState::FS_TILE_DATA_LOW => {
                let data_area = unsafe { LCD_CTX.get_lcdc_bg_tile_data_area() };
                let addr = data_area +
                    (self.pixel_fifo.bgw_fetch_data[0] as u16 * 16) +
                    (self.pixel_fifo.tile_y as u16);
                let data = bus_read(addr);
                self.pixel_fifo.bgw_fetch_data[1] = data;

                // Sets the next fetch state
                self.pixel_fifo.curr_state = FetchState::FS_TILE_DATA_HIGH;
            },
            FetchState::FS_TILE_DATA_HIGH => {
                let data_area = unsafe { LCD_CTX.get_lcdc_bg_tile_data_area() };
                let addr = data_area +
                    (self.pixel_fifo.bgw_fetch_data[0] as u16 * 16) +
                    (self.pixel_fifo.tile_y as u16 + 1);
                let data = bus_read(addr);
                self.pixel_fifo.bgw_fetch_data[2] = data;

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
     * A helper function that increments the LY register
     * and checks if the LY register matches the LYC register
     * and sets the LYC flag accordingly.
     */
    fn increment_ly(&mut self) -> () {
        unsafe { 
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
                let curr_time: u64 = UI::get_ticks();
                let frame_delay = curr_time - unsafe { prev_frame_time };
                if frame_delay < TARGET_FRAME_TIME {
                    UI::delay((TARGET_FRAME_TIME - frame_delay) as u32);
                }

                // Computes the FPS
                if curr_time - unsafe { start_timer } >= 1000 {
                    log::info!(target: "stdout", "FPS: {}", unsafe { frame_counter });
                    unsafe { 
                        frame_counter = 0;
                        start_timer = curr_time;
                    }
                }
                unsafe {
                    frame_counter = frame_counter.checked_add(1).unwrap();
                    prev_frame_time = UI::get_ticks();
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
                }
            }

            self.line_ticks = 0;
        }
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