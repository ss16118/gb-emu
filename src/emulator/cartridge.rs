use std::collections::HashMap;
use lazy_static::lazy_static;
use phf::{phf_map, Map};

// https://gbdev.io/pandocs/The_Cartridge_Header.html
// A struct that defines the cartridge header
struct RomHeader {
    entry_point: u32,
    nintendo_logo: [u8; 48],
    title: [u8; 10],
    manufacturer_code: [u8; 4],
    cgb_flag: u8,
    new_license_code: [u8; 2],
    sgb_flag: u8,
    cartridge_type: u8,
    rom_size: u8,
    ram_size: u8,
    destination_code: u8,
    old_license_code: u8,
    mask_rom_version_number: u8,
    header_checksum: u8,
    global_checksum: u16,
}

// A struct that defines the cartridge
// and stores the context of the cartridge
pub struct Cartridge {
    filename: String,
    rom_header: Box<RomHeader>,
    rom_size: usize,
    // Actual ROM data
    rom: Box<Vec<u8>>,
}

// A static lookup table that maps the cartridge type to a string
static CARTRIDGE_TYPE: Map<u8, &'static str> = phf_map! {
    0x00_u8 => "ROM ONLY",
    0x01_u8 => "MBC1",
    0x02_u8 => "MBC1+RAM",
    0x03_u8 => "MBC1+RAM+BATTERY",
    0x05_u8 => "MBC2",
    0x06_u8 => "MBC2+BATTERY",
    0x08_u8 => "ROM+RAM",
    0x09_u8 => "ROM+RAM+BATTERY",
    0x0B_u8 => "MMM01",
    0x0C_u8 => "MMM01+RAM",
    0x0D_u8 => "MMM01+RAM+BATTERY",
    0x0F_u8 => "MBC3+TIMER+BATTERY",
    0x10_u8 => "MBC3+TIMER+RAM+BATTERY",
    0x11_u8 => "MBC3",
    0x12_u8 => "MBC3+RAM",
    0x13_u8 => "MBC3+RAM+BATTERY",
    0x15_u8 => "MBC4",
    0x16_u8 => "MBC4+RAM",
    0x17_u8 => "MBC4+RAM+BATTERY",
    0x19_u8 => "MBC5",
    0x1A_u8 => "MBC5+RAM",
    0x1B_u8 => "MBC5+RAM+BATTERY",
    0x1C_u8 => "MBC5+RUMBLE",
    0x1D_u8 => "MBC5+RUMBLE+RAM",
    0x1E_u8 => "MBC5+RUMBLE+RAM+BATTERY",
    0x20_u8 => "MBC6",
    0x22_u8 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
    0xFC_u8 => "POCKET CAMERA",
    0xFD_u8 => "BANDAI TAMA5",
    0xFE_u8 => "HuC3",
    0xFF_u8 => "HuC1+RAM+BATTERY",
};



impl RomHeader {

    pub fn new() -> RomHeader {
        // Initializes an empty ROM header
        RomHeader {
            entry_point: 0,
            nintendo_logo: [0; 48],
            title: [0; 10],
            manufacturer_code: [0; 4],
            cgb_flag: 0,
            new_license_code: [0; 2],
            sgb_flag: 0,
            cartridge_type: 0,
            rom_size: 0,
            ram_size: 0,
            destination_code: 0,
            old_license_code: 0,
            mask_rom_version_number: 0,
            header_checksum: 0,
            global_checksum: 0,
        }
    }

    pub fn parse_rom_header(&mut self, rom_file: &str) -> () {
        // Parses the ROM header as per
        // https://gbdev.io/pandocs/The_Cartridge_Header.html
        let rom_data = std::fs::read(rom_file).unwrap();
        
    }
}



impl Cartridge {
    pub fn new() -> Cartridge {
        log::info!("Initializing cartridge...");
        // Initializes an empty cartridge
        Cartridge {
            filename: String::new(),
            rom_header: Box::new(RomHeader::new()),
            rom_size: 0,
            rom: Box::new(Vec::new()),
        }
    }

    /**
     * Parses the ROM header and stores the data in the cartridge
     */
    pub fn load_rom_file(&mut self, rom_file: &str) -> () {
        log::info!("Loading ROM file: {}", rom_file);
        self.filename = rom_file.to_string();
        let rom_data = std::fs::read(rom_file).expect("Unable to read ROM file");
        self.rom_size = rom_data.len();
        self.rom = Box::new(rom_data);
        self.rom_header.parse_rom_header(rom_file);
    }

    pub fn print_info(&self) -> () {
        log::info!("ROM file: {}", self.filename);
    }
}