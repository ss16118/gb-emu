
// Bit masks for accessing the OAM flags
const PRIORIT_MASK: u8      = 0x80;
const Y_FLIP_MASK: u8       = 0x40;
const X_FLIP_MAKS: u8       = 0x20;
const DMG_PALETTE_MASK: u8  = 0x10;
const BANK_MASK: u8         = 0x08;
const CGB_PALETTE_MASK: u8  = 0x07;


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
    oam_ram: [OamEntry; 40],
    vram: [u8; 0x2000],
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            oam_ram: [OamEntry::new(); 40],
            vram: [0; 0x2000],
        }
    }

    /**
     * Writes a byte to the OAM RAM
     */
    pub fn oam_write(&mut self, mut address: u16, value: u8) -> () {
        address -= 0xFE00;
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
        address -= 0xFE00;
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
}