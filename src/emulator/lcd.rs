use std::convert::TryFrom;
use crate::emulator::dma::*;

pub const LCD_START_ADDR: u16 = 0xFF40;
pub const LCD_END_ADDR: u16 = 0xFF4B;

const DEFAULT_COLORS: [u32; 4] = [
    0xFFFFFFFF,
    0xFFAAAAAA,
    0xFF555555,
    0xFF000000,
];
/**
 * A struct that defines the LCD and all
 * the registers associated with it
 * References:
 * https://gbdev.io/pandocs/LCDC.html
 * https://gbdev.io/pandocs/STAT.html
 */
pub struct LCD {
    // LCDC - LCD Control
    lcdc: u8,
    // STAT - LCDC Status
    lcds: u8,
    // SCY - Scroll Y
    pub scroll_y: u8,
    // SCX - Scroll X
    pub scroll_x: u8,
    // LY - LCDC Y-Coordinate
    pub ly: u8,
    // LYC - LY Compare
    pub lyc: u8,
    // DMA - DMA Transfer and Start Address
    dma: u8,
    // BGP - BG Palette Data
    bg_palette: u8,
    // OBP - Object Palette Data
    obj_palette: [u8; 2],
    // WY - Window Y Position
    win_y: u8,
    // WX - Window X Position
    win_x: u8,

    // Other data
    pub bg_colors: [u32; 4],
    sp1_colors: [u32; 4],
    sp2_colors: [u32; 4],
}

#[derive(Copy, Clone, Debug)]
pub enum LCD_MODE {
    MODE_HBLANK = 0,
    MODE_VBLANK = 1,
    MODE_OAM = 2,
    MODE_XFER = 3,
}

impl TryFrom<u8> for LCD_MODE {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(LCD_MODE::MODE_HBLANK),
            1 => Ok(LCD_MODE::MODE_VBLANK),
            2 => Ok(LCD_MODE::MODE_OAM),
            3 => Ok(LCD_MODE::MODE_XFER),
            _ => Err(()),
        }
    }
}

/**
 * LCD Control:
 * LCD & PPU enable: 0 = Off; 1 = On
 * Window tile map area: 0 = 9800–9BFF; 1 = 9C00–9FFF
 * Window enable: 0 = Off; 1 = On
 * BG & Window tile data area: 0 = 8800–97FF; 1 = 8000–8FFF
 * BG tile map area: 0 = 9800–9BFF; 1 = 9C00–9FFF
 * OBJ size: 0 = 8×8; 1 = 8×16
 * OBJ enable: 0 = Off; 1 = On
 * BG & Window enable / priority [Different meaning in CGB Mode]: 0 = Off; 1 = On
 */
/* Bit masks for accessing the LCD Control register */
pub const LCD_ENABLE_MASK: u8 = 0x80;
pub const WIN_TILE_MAP_MASK: u8 = 0x40;
pub const WIN_ENABLE_MASK: u8 = 0x20;
pub const BG_TILE_DATA_MASK: u8 = 0x10;
pub const BG_TILE_MAP_MASK: u8 = 0x08;
pub const OBJ_SIZE_MASK: u8 = 0x04;
pub const OBJ_ENABLE_MASK: u8 = 0x02;
pub const BGW_ENABLE_MASK: u8 = 0x01;

/**
 * LCD Status:
 * LYC int select (Read/Write): If set, selects the LYC == LY condition for the STAT interrupt.
 * Mode 2 int select (Read/Write): If set, selects the Mode 2 condition for the STAT interrupt.
 * Mode 1 int select (Read/Write): If set, selects the Mode 1 condition for the STAT interrupt.
 * Mode 0 int select (Read/Write): If set, selects the Mode 0 condition for the STAT interrupt.
 * LYC == LY (Read-only): Set when LY contains the same value as LYC; it is constantly updated.
 * PPU mode (Read-only): Indicates the PPU’s current status.
 * 
 *
 * LCD Interrupts:
 * A hardware quirk in the monochrome Game Boy makes the LCD interrupt
 * sometimes trigger when writing to STAT (including writing $00) during
 * OAM scan, HBlank, VBlank, or LY=LYC. It behaves as if $FF were written
 * for one cycle, and then the written value were written the next cycle.
 * Because the GBC in DMG mode does not have this quirk, two games that
 * depend on this quirk (Ocean’s Road Rash and Vic Tokai’s Xerd no Densetsu)
 * will not run on a GBC.
 */
/* Bit masks for accessing the LCD Status register */
pub const LYC_INT_MASK: u8 = 0x40;
pub const OAM_INT_MASK: u8 = 0x20;
pub const VBLANK_INT_MASK: u8 = 0x10;
pub const HBLANK_INT_MASK: u8 = 0x08;
const LYC_LY_MASK: u8 = 0x04;
const PPU_MODE_MASK: u8 = 0x03;


pub static mut LCD_CTX: LCD = LCD {
    lcdc: 0x91,
    lcds: 0,
    scroll_x: 0,
    scroll_y: 0,
    ly: 0,
    lyc: 0,
    dma: 0,
    bg_palette: 0xFC,
    obj_palette: [0xFF; 2],
    win_x: 0,
    win_y: 0,
    bg_colors: [DEFAULT_COLORS[0], DEFAULT_COLORS[1], DEFAULT_COLORS[2], DEFAULT_COLORS[3]],
    sp1_colors: [DEFAULT_COLORS[0], DEFAULT_COLORS[1], DEFAULT_COLORS[2], DEFAULT_COLORS[3]],
    sp2_colors: [DEFAULT_COLORS[0], DEFAULT_COLORS[1], DEFAULT_COLORS[2], DEFAULT_COLORS[3]],
};


impl LCD {
    pub fn init() -> () {
        log::info!("Initializing LCD...");
        unsafe { LCD_CTX.set_lcds_mode(LCD_MODE::MODE_OAM); }
        log::info!(target: "stdout", "Initialize LCD: SUCCESS");
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.lcds,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            DMA_ADDR => self.dma,
            0xFF47 => self.bg_palette,
            0xFF48 => self.obj_palette[0],
            0xFF49 => self.obj_palette[1],
            0xFF4A => self.win_y,
            0xFF4B => self.win_x,
            _ => panic!("Invalid LCD register read: {:#X}", addr),
        }
    }

    fn update_palette(&mut self, palette_data: u8, palette: u8) -> () {
        let colors: &mut [u32; 4];
        match palette {
            0 => {
                colors = &mut self.bg_colors;
            },
            1 => {
                colors = &mut self.sp1_colors;
            },
            2 => {
                colors = &mut self.sp2_colors;
            },
            _ => {
                log::warn!("Invalid palette number: {}", palette);
                std::process::exit(1);
            }
        }

        colors[0] = DEFAULT_COLORS[(palette_data & 0b11) as usize];
        colors[1] = DEFAULT_COLORS[((palette_data >> 2) & 0b11) as usize];
        colors[2] = DEFAULT_COLORS[((palette_data >> 4) & 0b11) as usize];
        colors[3] = DEFAULT_COLORS[((palette_data >> 6) & 0b11) as usize];
    }

    pub fn write(&mut self, addr: u16, value: u8) -> () {
        match addr {
            0xFF40 => self.lcdc = value,
            0xFF41 => self.lcds = value,
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => self.ly = value,
            0xFF45 => self.lyc = value,
            DMA_ADDR => { 
                self.dma = value;
                unsafe { DMA_CTX.start(value) };
            }
            0xFF47 => { 
                self.update_palette(value, 0);
            },
            0xFF48 => {
                // the lower two bits are ignored because color index 0 is transparent for OBJs
                self.update_palette(value & 0b11111100, 1);
            },
            0xFF49 => {
                // the lower two bits are ignored because color index 0 is transparent for OBJs
                self.update_palette(value & 0b11111100, 2);
            },
            0xFF4A => self.win_y = value,
            0xFF4B => self.win_x = value,
            _ => panic!("Invalid LCD register write: {:#X}", addr),
        }
    }

    /* Functions for accessing the LCD Control register */
    pub fn get_lcdc_win_tile_map_area(&self) -> u16 {
        return if self.get_lcdc_flag(WIN_TILE_MAP_MASK) { 0x9C00 } else { 0x9800 };
    }

    pub fn get_lcdc_bg_tile_data_area(&self) -> u16 {
        return if self.get_lcdc_flag(BG_TILE_DATA_MASK) { 0x8000 } else { 0x8800 };
    }

    pub fn get_lcdc_bg_tile_map_area(&self) -> u16 {
        return if self.get_lcdc_flag(BG_TILE_MAP_MASK) { 0x9C00 } else { 0x9800 };
    }

    pub fn get_lcdc_obj_size(&self) -> u8 {
        let obj_size_flag = self.get_lcdc_flag(OBJ_SIZE_MASK);
        return if obj_size_flag { 16 } else { 8 };
    }

    pub fn get_lcdc_flag(&self, mask: u8) -> bool {
        return (self.lcdc & mask) != 0;
    }

    /* Functions for accessing the LCD Status register */

    /**
     * Returns the current mode of the PPU
     */
    pub fn get_lcds_mode(&self) -> LCD_MODE {
        return LCD_MODE::try_from(self.lcds & PPU_MODE_MASK).unwrap();
    }

    /**
     * Sets the current mode of the PPU
     */
    pub fn set_lcds_mode(&mut self, mode: LCD_MODE) -> () {
        self.lcds = (self.lcds & !PPU_MODE_MASK) | (mode as u8);
    }

    /**
     * Returns the value of the LYC flag
     */
    pub fn get_lcds_lyc(&self) -> bool {
        return (self.lcds & LYC_LY_MASK) != 0;
    }

    /**
     * Sets the value of the LYC flag
     */
    pub fn set_lcds_lyc(&mut self, value: bool) -> () {
        self.lcds = (self.lcds & !LYC_LY_MASK) | ((value as u8) << 2);
    }

    /**
     * Returns the value of a LCD Status flag given its mask
     */
    pub fn get_lcds_flag(&self, mask: u8) -> bool {
        return (self.lcds & mask) != 0;
    }
}