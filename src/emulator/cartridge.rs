use phf::{phf_map, Map};

// https://gbdev.io/pandocs/The_Cartridge_Header.html
// A struct that defines the cartridge header
// https://stackoverflow.com/questions/70697768/transmute-struct-into-array-in-rust
#[repr(C)]
struct RomHeader {
    entry_point: [u8; 4],
    nintendo_logo: [u8; 48],
    title: [u8; 16],
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
    rom_header: *const RomHeader,
    rom_size: usize,
    // Actual ROM data
    rom: Vec<u8>,
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

// A static lookup table that maps the RAM size to a string
static RAM_SIZE: Map<u8, &'static str> = phf_map! {
    0x00_u8 => "No RAM",
    0x01_u8 => "Unused",
    0x02_u8 => "8 KiB",
    0x03_u8 => "32 KiB (4 banks of 8 KiB each)",
    0x04_u8 => "128 KiB (16 banks of 8 KiB each)",
    0x05_u8 => "64 KiB (8 banks of 8 KiB each)",
};


// A static lookup table that maps the license code to a string
// https://gbdev.io/pandocs/The_Cartridge_Header.html#01440145--new-licensee-code
static LICENSE_CODE: Map<&'static str, &'static str> = phf_map! {
    "00" => "None",
    "01" => "Nintendo R&D1",
    "08" => "Capcom",
    "13" => "Electronic Arts",
    "18" => "Hudson Soft",
    "19" => "B-AI",
    "20" => "KSS",
    "22" => "POW",
    "24" => "PCM Complete",
    "25" => "San-X",
    "28" => "Kemco Japan",
    "29" => "Seta",
    "30" => "Viacom",
    "31" => "Nintendo",
    "32" => "Bandai",
    "33" => "Ocean/Acclaim",
    "34" => "Konami",
    "35" => "Hector",
    "37" => "Taito",
    "38" => "Hudson",
    "39" => "Banpresto",
    "41" => "Ubisoft",
    "42" => "Atlus",
    "44" => "Malibu",
    "46" => "Angel",
    "47" => "Bullet-Proof",
    "49" => "Irem",
    "50" => "Absolute",
    "51" => "Acclaim",
    "52" => "Activision",
    "53" => "American Sammy",
    "54" => "Konami",
    "55" => "Hi Tech Entertainment",
    "56" => "LJN",
    "57" => "Matchbox",
    "58" => "Mattel",
    "59" => "Milton Bradley",
    "60" => "Titus",
    "61" => "Virgin",
    "64" => "LucasArts",
    "67" => "Ocean",
    "69" => "Electronic Arts",
    "70" => "Infogrames",
    "71" => "Interplay",
    "72" => "Broderbund",
    "73" => "Sculptured",
    "75" => "SCI",
    "78" => "THQ",
    "79" => "Accolade",
    "80" => "Misawa",
    "83" => "Lozc",
    "86" => "Tokuma Shoten Intermedia",
    "87" => "Tsukuda Original",
    "91" => "Chunsoft",
    "92" => "Video System",
    "93" => "Ocean/Acclaim",
    "95" => "Varie",
    "96" => "Yonezawa/S'pal",
    "97" => "Kaneko",
    "99" => "Pack in soft",
    "9F" => "Bottom Up",
    "A4" => "Konami (Yu-Gi-Oh!)",
};



impl Cartridge {
    pub fn new() -> Cartridge {
        log::info!("Initializing cartridge...");
        // Initializes an empty cartridge
        let cartridge = Cartridge {
            filename: String::new(),
            rom_header: std::ptr::null(),
            rom_size: 0,
            rom: Vec::new(),
        };
        log::info!(target: "stdout", "Initialize cartridge: SUCCESS");
        return cartridge;
    }

    /**
     * Parses the ROM header and stores the data in the cartridge
     */
    pub fn load_rom_file(&mut self, rom_file: &str) -> () {
        log::info!("Loading ROM file: {}", rom_file);
        self.filename = rom_file.to_string();
        let rom_data = std::fs::read(rom_file).expect("Unable to read ROM file");
        self.rom_size = rom_data.len();
        self.rom = rom_data;
        // Parses the ROM header by transmuting the memory starting at 0x100
        // and stores the data in the cartridge
        self.rom_header = unsafe {
            std::mem::transmute::<*const u8, *const RomHeader>(&self.rom[0x100])
        };
        // Verifies the ROM header checksum
        if !self.verify_checksum() {
            log::error!(target: "stdout", "Verify ROM header checksum: FAILED");
            std::process::exit(1);
        }
        log::info!(target: "stdout", "Loading ROM file: SUCCESS");
    }

    /**
     * Verifies the ROM header checksum
     * https://gbdev.io/pandocs/The_Cartridge_Header.html#014d--header-checksum
     */
    fn verify_checksum(&self) -> bool {
        log::info!("Verifying ROM header checksum...");
        let mut checksum: u8 = 0;
        for i in 0x134..0x14D {
            // Prints address in hexadecimal
            checksum = checksum.wrapping_sub(self.rom[i]).wrapping_sub(1);
        }
        let result = unsafe {
            checksum == (*self.rom_header).header_checksum
        };
        if result {
            log::info!("Verifying ROM header checksum: SUCCESS");
        }
        return result;
    }

    /**
     * Reads a byte from the ROM
     */
    pub fn read(&self, address: u16) -> u8 {
        // FIXME add support for RAM
        return self.rom[address as usize];
    }

    /**
     * Writes a byte to the ROM. Returns true if the write was successful,
     * false otherwise.
     */
    pub fn write(&mut self, address: u16, data: u8) -> () {
        self.rom[address as usize] = data;
    }

    /**
     * Prints the cartridge information to the log file and/or stdout.
     * @param to_stdout: Whether to print to stdout or not.
     * @return (): Nothing
     */
    pub fn print_info(&self, to_stdout: bool) -> () {
        let print_target = if to_stdout { "stdout" } else { "log_file" };
        log::info!(target: print_target, "======= Cartridge information =======");
        log::info!(target: print_target, "  Filename: {}", self.filename);
        log::info!(target: print_target, "  ROM size: {} bytes", self.rom_size);
        unsafe {
            // Casts the title from a u8 array to a string
            let title = std::str::from_utf8_unchecked(&(*self.rom_header).title);
            // Removes the trailing NULL characters
            let title = title.trim_end_matches(char::from(0));
            log::info!(target: print_target, "  Title: {}", title);
            // Prints the cartridge type
            let cartridge_type = CARTRIDGE_TYPE[&(*self.rom_header).cartridge_type];
            log::info!(target: print_target, "  Cartridge type: {} ({})",
                (*self.rom_header).cartridge_type, cartridge_type);
            // Prints the RAM size
            let ram_size = RAM_SIZE[&(*self.rom_header).ram_size];
            log::info!(target: print_target, "  RAM size: {} ({})", 
                (*self.rom_header).ram_size, ram_size);
            // Prints the license code
            let license_code = 
                std::str::from_utf8_unchecked(&(*self.rom_header).new_license_code);
            let license_code_str: &str;
            // Checks if license code is valid
            if !LICENSE_CODE.contains_key(license_code) {
                log::warn!(target: print_target, "  Invalid license code: {}", license_code);
                license_code_str = "UNKNOWN";
            } else {
                license_code_str = LICENSE_CODE[&license_code];
            }
            log::info!(target: print_target, "  License code: {} ({})",
                license_code, license_code_str);
        }
        log::info!(target: print_target, "=====================================");
    }    
}